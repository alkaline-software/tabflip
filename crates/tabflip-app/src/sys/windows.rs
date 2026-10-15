//! The windows to switch between: every app's windows on this Space, and
//! the ones minimized or hidden with their app.
//!
//! The apps are the ones the Dock shows (a regular activation policy). Each
//! is asked through Accessibility for its windows, their titles and
//! whether they are minimized. Which window is in front of which comes
//! from the window server's list of what is on screen, which gives window
//! numbers and owners without Screen Recording access; the titles are
//! Accessibility's, so that is the one permission tabflip needs.

use objc2_app_kit::{NSApplicationActivationPolicy, NSRunningApplication, NSWorkspace};
use objc2_foundation::{NSArray, NSDictionary, NSNumber, NSString};

use super::ax::Element;

#[derive(Clone)]
pub struct Window {
    pub pid: i32,
    pub id: Option<u32>,
    pub title: String,
    pub app_name: String,
    pub bundle_id: String,
    pub minimized: bool,
    pub app_hidden: bool,
    pub element: Element,
}

/// The apps to ask, read on the main thread.
#[derive(Clone)]
pub struct App {
    pub pid: i32,
    pub name: String,
    pub bundle_id: String,
    pub hidden: bool,
}

pub fn apps() -> Vec<App> {
    let me = std::process::id() as i32;
    let workspace = NSWorkspace::sharedWorkspace();
    workspace
        .runningApplications()
        .iter()
        .filter(|a| a.activationPolicy() == NSApplicationActivationPolicy::Regular && a.processIdentifier() != me && !a.isTerminated())
        .map(|a| App {
            pid: a.processIdentifier(),
            name: a.localizedName().map(|s| s.to_string()).unwrap_or_default(),
            bundle_id: a.bundleIdentifier().map(|s| s.to_string()).unwrap_or_default(),
            hidden: a.isHidden(),
        })
        .collect()
}

/// Every window of `apps` worth switching to, in the order the apps
/// list them. Each app is asked on a thread of its own, so the whole takes
/// as long as the slowest app, and no app longer than the quarter-second
/// limit per question. Off the main thread.
pub fn list(apps: &[App], on_screen: &[u32]) -> Vec<Window> {
    std::thread::scope(|scope| {
        let asked: Vec<_> = apps.iter().map(|app| scope.spawn(move || app_windows(app, on_screen))).collect();
        asked.into_iter().flat_map(|t| t.join().unwrap_or_default()).collect()
    })
}

fn app_windows(app: &App, on_screen: &[u32]) -> Vec<Window> {
    let mut out = Vec::new();
    let Some(el) = Element::app(app.pid) else { return out };
    for w in el.elements("AXWindows") {
        if w.string("AXRole").as_deref() != Some("AXWindow") {
            continue;
        }
        let subrole = w.string("AXSubrole").unwrap_or_default();
        let title = w.string("AXTitle").unwrap_or_default();
        // Documents and their dialogs; not palettes, sheets or popovers.
        let kept = subrole == "AXStandardWindow" || (subrole == "AXDialog" && !title.is_empty());
        if !kept {
            continue;
        }
        let minimized = w.bool("AXMinimized").unwrap_or(false);
        let id = w.window_id();
        // A window neither on screen nor put away is on another Space.
        let here = id.is_some_and(|n| on_screen.contains(&n));
        if !(here || minimized || app.hidden) {
            continue;
        }
        out.push(Window { pid: app.pid, id, title, app_name: app.name.clone(), bundle_id: app.bundle_id.clone(), minimized, app_hidden: app.hidden, element: w });
    }
    out
}

/// The numbers of the ordinary windows on screen, front to back.
pub fn on_screen() -> Vec<u32> {
    type Ref = *mut std::ffi::c_void;
    #[link(name = "CoreGraphics", kind = "framework")]
    extern "C" {
        fn CGWindowListCopyWindowInfo(option: u32, relative_to: u32) -> Ref;
    }
    #[link(name = "CoreFoundation", kind = "framework")]
    extern "C" {
        fn CFRelease(r: Ref);
    }
    const ON_SCREEN_ONLY: u32 = 1 << 0;
    const EXCLUDE_DESKTOP: u32 = 1 << 4;
    let list = unsafe { CGWindowListCopyWindowInfo(ON_SCREEN_ONLY | EXCLUDE_DESKTOP, 0) };
    if list.is_null() {
        return Vec::new();
    }
    let array = unsafe { &*(list as *const NSArray<NSDictionary<NSString, NSNumber>>) };
    let (number, layer) = (NSString::from_str("kCGWindowNumber"), NSString::from_str("kCGWindowLayer"));
    let mut out = Vec::new();
    for info in array.iter() {
        // Layer 0 holds documents; the menu bar, the Dock and panels like
        // ours sit above it.
        let on_layer_0 = info.objectForKey(&layer).is_some_and(|l| l.intValue() == 0);
        if let (true, Some(n)) = (on_layer_0, info.objectForKey(&number)) {
            out.push(n.unsignedIntValue());
        }
    }
    unsafe { CFRelease(list) };
    out
}

/// The focused window's number in the app in front: what the switcher
/// remembers as just used.
pub fn focused() -> Option<u32> {
    let app = NSWorkspace::sharedWorkspace().frontmostApplication()?;
    let el = Element::app(app.processIdentifier())?;
    el.element("AXFocusedWindow")?.window_id()
}

/// The app's icon, as a PNG drawn at `px` pixels square.
pub fn icon_png(pid: i32, px: f64) -> Option<Vec<u8>> {
    let app = NSRunningApplication::runningApplicationWithProcessIdentifier(pid)?;
    let icon = app.icon()?;
    png(&icon, px)
}

/// An installed app's icon by its bundle id, running or not.
pub fn bundle_icon_png(bundle_id: &str, px: f64) -> Option<Vec<u8>> {
    let workspace = NSWorkspace::sharedWorkspace();
    let url = workspace.URLForApplicationWithBundleIdentifier(&NSString::from_str(bundle_id))?;
    let path = url.path()?;
    png(&workspace.iconForFile(&path), px)
}

fn png(image: &objc2_app_kit::NSImage, px: f64) -> Option<Vec<u8>> {
    use objc2::AllocAnyThread as _;
    use objc2_app_kit::{NSBitmapImageFileType, NSBitmapImageRep, NSCompositingOperation, NSDeviceRGBColorSpace, NSGraphicsContext};
    use objc2_foundation::{NSDictionary, NSPoint, NSRect, NSSize};

    let side = px as isize;
    let rep = unsafe {
        NSBitmapImageRep::initWithBitmapDataPlanes_pixelsWide_pixelsHigh_bitsPerSample_samplesPerPixel_hasAlpha_isPlanar_colorSpaceName_bytesPerRow_bitsPerPixel(
            NSBitmapImageRep::alloc(),
            std::ptr::null_mut(),
            side,
            side,
            8,
            4,
            true,
            false,
            NSDeviceRGBColorSpace,
            0,
            0,
        )
    }?;
    let context = NSGraphicsContext::graphicsContextWithBitmapImageRep(&rep)?;
    NSGraphicsContext::saveGraphicsState_class();
    NSGraphicsContext::setCurrentContext(Some(&context));
    let rect = NSRect::new(NSPoint::new(0., 0.), NSSize::new(px, px));
    image.drawInRect_fromRect_operation_fraction(rect, NSRect::ZERO, NSCompositingOperation::SourceOver, 1.0);
    context.flushGraphics();
    NSGraphicsContext::restoreGraphicsState_class();
    let data = unsafe { rep.representationUsingType_properties(NSBitmapImageFileType::PNG, &NSDictionary::new()) }?;
    Some(data.to_vec())
}
