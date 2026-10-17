//! Everything that talks to macOS. The rest of the app sees plain Rust.

pub mod ax;
pub mod focus;
pub mod status_item;
pub mod private;
pub mod tap;
pub mod windows;

use objc2::MainThreadMarker;
use objc2_app_kit::{NSView, NSWindow};

/// A line in `~/Library/Logs/tabflip-dev.log`, in debug builds only: what
/// the switcher did and how long it took, for a dev build being tried.
pub fn log(line: std::fmt::Arguments) {
    if cfg!(debug_assertions) {
        use std::io::Write as _;
        let Some(dir) = dirs::home_dir() else { return };
        let path = dir.join("Library/Logs/tabflip-dev.log");
        if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(path) {
            let t = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default();
            let _ = writeln!(f, "{}.{:03} {line}", t.as_secs() % 100_000, t.subsec_millis());
        }
    }
}

/// The bundle this process runs from, by its identifier.
pub fn bundle_id() -> Option<String> {
    Some(objc2_foundation::NSBundle::mainBundle().bundleIdentifier()?.to_string())
}

/// The entry the system keeps for this app under Accessibility,
/// forgotten (`tccutil reset`), so the next question makes a fresh one.
/// The system binds a grant to the app's signature: an entry made for a
/// copy signed otherwise (an earlier build) shows as on in System
/// Settings and counts for nothing, and switching it off and on changes
/// nothing; only removing it and granting again does, which is what the
/// person had to do by hand before Allow did this. Nothing to forget is
/// not an error.
pub fn forget_trust() {
    let Some(id) = bundle_id() else { return };
    let out = std::process::Command::new("/usr/bin/tccutil").args(["reset", "Accessibility", &id]).output();
    log(format_args!("forget_trust {id}: {:?}", out.map(|o| String::from_utf8_lossy(&o.stderr).trim().to_string())));
}

/// The settings page in System Settings where Accessibility is granted.
pub const ACCESSIBILITY_SETTINGS_URL: &str = "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility";

fn ns_window(window: &gpui::Window) -> Option<objc2::rc::Retained<NSWindow>> {
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};
    let handle = HasWindowHandle::window_handle(window).ok()?;
    let RawWindowHandle::AppKit(handle) = handle.as_raw() else { return None };
    let view = unsafe { (handle.ns_view.as_ptr() as *const NSView).as_ref() }?;
    view.window()
}

/// A message each time Accessibility's list changes, for any app: the
/// system's own notice, so nothing asks over and over.
pub fn trust_changes() -> futures::channel::mpsc::UnboundedReceiver<()> {
    use objc2_foundation::{NSDistributedNotificationCenter, NSNotification, NSOperationQueue, NSString};
    let (tx, rx) = futures::channel::mpsc::unbounded();
    let block = block2::RcBlock::new(move |_: std::ptr::NonNull<NSNotification>| {
        let _ = tx.unbounded_send(());
    });
    let center = NSDistributedNotificationCenter::defaultCenter();
    let observer = unsafe { center.addObserverForName_object_queue_usingBlock(Some(&NSString::from_str("com.apple.accessibility.api")), None, Some(&NSOperationQueue::mainQueue()), &block) };
    // Kept for the app's life.
    std::mem::forget(observer);
    rx
}

/// A message each time another app comes to the front, for the windows
/// it is remembered to have used.
pub fn activations() -> futures::channel::mpsc::UnboundedReceiver<()> {
    use objc2_app_kit::{NSWorkspace, NSWorkspaceDidActivateApplicationNotification};
    use objc2_foundation::NSNotification;
    let (tx, rx) = futures::channel::mpsc::unbounded();
    let block = block2::RcBlock::new(move |_: std::ptr::NonNull<NSNotification>| {
        let _ = tx.unbounded_send(());
    });
    let center = NSWorkspace::sharedWorkspace().notificationCenter();
    let observer = unsafe { center.addObserverForName_object_queue_usingBlock(Some(NSWorkspaceDidActivateApplicationNotification), None, None, &block) };
    std::mem::forget(observer);
    rx
}

/// The switcher's panel placed: `size` points, centred on the screen
/// under the pointer, and shown over everything without taking the
/// keyboard. It moves with whatever Space is showing, full-screen ones
/// included.
pub fn show_panel(window: &gpui::Window, width: f64, height: f64) {
    use objc2_app_kit::{NSEvent, NSScreen, NSWindowCollectionBehavior};
    use objc2_foundation::{NSPoint, NSRect, NSSize};
    let (Some(native), Some(mtm)) = (ns_window(window), MainThreadMarker::new()) else {
        log(format_args!("show_panel: no native window"));
        return;
    };
    let at = NSEvent::mouseLocation();
    let screens = NSScreen::screens(mtm);
    let inside = |f: NSRect| at.x >= f.origin.x && at.x < f.origin.x + f.size.width && at.y >= f.origin.y && at.y < f.origin.y + f.size.height;
    let Some(screen) = screens.iter().find(|s| inside(s.frame())).or_else(|| screens.firstObject()) else { return };
    let area = screen.visibleFrame();
    let origin = NSPoint::new(area.origin.x + (area.size.width - width) / 2., area.origin.y + (area.size.height - height) / 2. + area.size.height * 0.08);
    native.setFrame_display(NSRect::new(origin, NSSize::new(width, height)), true);
    native.setCollectionBehavior(NSWindowCollectionBehavior::CanJoinAllSpaces | NSWindowCollectionBehavior::FullScreenAuxiliary | NSWindowCollectionBehavior::Transient | NSWindowCollectionBehavior::IgnoresCycle);
    native.setHidesOnDeactivate(false);
    native.setHasShadow(false);
    native.orderFrontRegardless();
}

pub fn hide_panel(window: &gpui::Window) {
    if let Some(native) = ns_window(window) {
        native.orderOut(None);
    }
}

/// The height of the screen under the pointer, for the longest the list
/// may be.
pub fn screen_height() -> f64 {
    use objc2_app_kit::{NSEvent, NSScreen};
    let Some(mtm) = MainThreadMarker::new() else { return 800. };
    let at = NSEvent::mouseLocation();
    NSScreen::screens(mtm)
        .iter()
        .find(|s| {
            let f = s.frame();
            at.x >= f.origin.x && at.x < f.origin.x + f.size.width && at.y >= f.origin.y && at.y < f.origin.y + f.size.height
        })
        .map(|s| s.visibleFrame().size.height)
        .unwrap_or(800.)
}

/// Every other tabflip running as an app, the release's or the build's,
/// asked to quit, and made to if it is still there a few seconds on: two
/// would each take the shortcut. The one opened last stays. Whether any
/// was found is returned: a quitting copy gives the system its ⌘Tab back
/// (`on_app_quit`), after this one may have taken it, so the caller
/// applies the shortcut again once they are gone.
pub fn end_other_tabflips() -> bool {
    use objc2_app_kit::NSRunningApplication;
    use objc2_foundation::NSString;
    let me = std::process::id() as i32;
    let mut others = Vec::new();
    for id in ["com.alkalinesoftware.tabflip", "com.alkalinesoftware.tabflip.dev"] {
        for app in NSRunningApplication::runningApplicationsWithBundleIdentifier(&NSString::from_str(id)).iter() {
            let pid = app.processIdentifier();
            if pid != me && pid > 0 {
                app.terminate();
                others.push(pid);
            }
        }
    }
    if others.is_empty() {
        return false;
    }
    log(format_args!("ending other tabflips: {others:?}"));
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_secs(3));
        for pid in others {
            if let Some(app) = NSRunningApplication::runningApplicationWithProcessIdentifier(pid) {
                if !app.isTerminated() {
                    app.forceTerminate();
                }
            }
        }
    });
    true
}

/// No Dock icon and no menu at the screen's top: an app of the menu bar.
/// gpui makes every app a regular one as it starts, over the bundle's
/// `LSUIElement`, so this is said again after it.
pub fn stay_in_menu_bar() {
    in_dock(false);
}

/// In the Dock and the app switcher while Settings are open, as any app
/// with a window is, so the window is found again once another app covers
/// it; back to the menu bar alone when it closes.
pub fn in_dock(on: bool) {
    use objc2_app_kit::{NSApplication, NSApplicationActivationPolicy};
    let Some(mtm) = MainThreadMarker::new() else { return };
    let policy = if on { NSApplicationActivationPolicy::Regular } else { NSApplicationActivationPolicy::Accessory };
    NSApplication::sharedApplication(mtm).setActivationPolicy(policy);
}

/// Whether tabflip opens when the person logs in, as System Settings'
/// Login Items lists it.
pub fn launches_at_login() -> bool {
    use objc2_service_management::{SMAppService, SMAppServiceStatus};
    unsafe { SMAppService::mainAppService().status() == SMAppServiceStatus::Enabled }
}

pub fn set_launch_at_login(on: bool) -> Result<(), String> {
    use objc2_service_management::SMAppService;
    let service = unsafe { SMAppService::mainAppService() };
    let done = unsafe { if on { service.registerAndReturnError() } else { service.unregisterAndReturnError() } };
    done.map_err(|e| e.localizedDescription().to_string())
}

// The window's language. On a Mac an app's language is its own
// `AppleLanguages` in its defaults, which System Settings, Language &
// Region, Applications writes. tabflip's setting is the same one.
mod languages {
    use objc2::runtime::AnyObject;
    use objc2_foundation::{NSArray, NSBundle, NSString, NSUserDefaults};

    const KEY: &str = "AppleLanguages";

    /// `AppleLanguages` as one domain of the defaults has it, alone: not
    /// the search list's, which falls back from the app's to the system's.
    fn list(domain: &str) -> Vec<String> {
        let defaults = NSUserDefaults::standardUserDefaults();
        let Some(dict) = defaults.persistentDomainForName(&NSString::from_str(domain)) else { return Vec::new() };
        let Some(value) = dict.objectForKey(&NSString::from_str(KEY)) else { return Vec::new() };
        let Ok(all) = value.downcast::<NSArray>() else { return Vec::new() };
        all.iter().filter_map(|o| o.downcast::<NSString>().ok().map(|s| s.to_string())).collect()
    }

    pub fn system() -> Vec<String> {
        list("NSGlobalDomain")
    }

    pub fn own() -> Option<String> {
        let id = NSBundle::mainBundle().bundleIdentifier()?;
        list(&id.to_string()).into_iter().next()
    }

    pub fn set_own(code: Option<&str>) {
        let defaults = NSUserDefaults::standardUserDefaults();
        let key = NSString::from_str(KEY);
        match code {
            Some(code) => {
                let one = NSArray::from_retained_slice(&[NSString::from_str(code)]);
                let one: &AnyObject = &one;
                unsafe { defaults.setObject_forKey(Some(one), &key) };
            }
            None => defaults.removeObjectForKey(&key),
        }
    }
}

/// The system's preferred languages, in its order, apart from any the
/// app was given of its own; the locale's when the defaults name none.
pub fn system_languages() -> Vec<String> {
    let all = languages::system();
    if !all.is_empty() {
        return all;
    }
    objc2_foundation::NSLocale::preferredLanguages().iter().map(|s| s.to_string()).collect()
}

/// The language the app was given of its own, if any: by tabflip's
/// setting or by System Settings.
pub fn own_language() -> Option<String> {
    languages::own()
}

/// The app given a language of its own (a bundle's code, `zh-Hans`), or
/// none, to follow the system's.
pub fn set_own_language(code: Option<&str>) {
    languages::set_own(code);
}

/// Open a URL in the app the system gives it to.
pub fn open_url(url: &str) {
    use objc2_app_kit::NSWorkspace;
    use objc2_foundation::{NSString, NSURL};
    if let Some(url) = NSURL::URLWithString(&NSString::from_str(url)) {
        NSWorkspace::sharedWorkspace().openURL(&url);
    }
}

/// A picture of one of the app's own windows, written to `path` as a PNG,
/// for looking at the panel from a script (`TABFLIP_SHOT`). A process may
/// capture its own windows without Screen Recording access.
/// `CGWindowListCreateImage` is looked up by name: newer SDKs mark it
/// unavailable, while the function is still there.
pub fn shoot_window(window: &gpui::Window, path: &std::path::Path) -> Result<(), String> {
    use std::ffi::{c_char, c_void, CString};
    use std::os::unix::ffi::OsStrExt;

    #[repr(C)]
    struct Rect {
        x: f64,
        y: f64,
        w: f64,
        h: f64,
    }
    type Capture = unsafe extern "C" fn(Rect, u32, u32, u32) -> *mut c_void;
    #[link(name = "ImageIO", kind = "framework")]
    #[link(name = "CoreFoundation", kind = "framework")]
    extern "C" {
        fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void;
        fn CFURLCreateFromFileSystemRepresentation(alloc: *const c_void, path: *const u8, len: isize, is_dir: u8) -> *mut c_void;
        fn CFStringCreateWithCString(alloc: *const c_void, text: *const c_char, encoding: u32) -> *mut c_void;
        fn CGImageDestinationCreateWithURL(url: *mut c_void, kind: *mut c_void, count: usize, options: *const c_void) -> *mut c_void;
        fn CGImageDestinationAddImage(dest: *mut c_void, image: *mut c_void, properties: *const c_void);
        fn CGImageDestinationFinalize(dest: *mut c_void) -> u8;
        fn CFRelease(obj: *mut c_void);
    }

    let number = ns_window(window).ok_or("no window")?.windowNumber();
    let bytes = path.as_os_str().as_bytes();
    unsafe {
        let capture = dlsym(-2isize as *mut c_void, c"CGWindowListCreateImage".as_ptr());
        if capture.is_null() {
            return Err("this system has no CGWindowListCreateImage".into());
        }
        let capture: Capture = std::mem::transmute(capture);
        let null = Rect { x: f64::INFINITY, y: f64::INFINITY, w: 0.0, h: 0.0 };
        // Only this window, its own bounds, without the system's shadow.
        let image = capture(null, 1 << 3, number as u32, 1);
        if image.is_null() {
            return Err("the window could not be captured".into());
        }
        let url = CFURLCreateFromFileSystemRepresentation(std::ptr::null(), bytes.as_ptr(), bytes.len() as isize, 0);
        let png = CString::new("public.png").unwrap();
        let kind = CFStringCreateWithCString(std::ptr::null(), png.as_ptr(), 0x0800_0100);
        let dest = CGImageDestinationCreateWithURL(url, kind, 1, std::ptr::null());
        let written = if dest.is_null() {
            false
        } else {
            CGImageDestinationAddImage(dest, image, std::ptr::null());
            let ok = CGImageDestinationFinalize(dest) != 0;
            CFRelease(dest);
            ok
        };
        CFRelease(kind);
        CFRelease(url);
        CFRelease(image);
        if written { Ok(()) } else { Err(format!("could not write {}", path.display())) }
    }
}
