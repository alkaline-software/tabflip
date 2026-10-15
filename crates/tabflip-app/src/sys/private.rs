//! The window server's private calls (SkyLight): bringing one window of
//! another app to the front, and switching the system's ⌘Tab off and on.
//!
//! Each is looked up by name when first used, so a macOS that renames one
//! leaves tabflip running with that one thing missing, not unable to
//! start. AltTab, yabai and Hammerspoon use the same calls.

use std::ffi::{c_char, c_void};
use std::sync::OnceLock;

extern "C" {
    fn dlopen(path: *const c_char, mode: i32) -> *mut c_void;
    fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void;
}

const RTLD_LAZY: i32 = 1;

fn skylight() -> *mut c_void {
    static HANDLE: OnceLock<usize> = OnceLock::new();
    *HANDLE.get_or_init(|| unsafe { dlopen(c"/System/Library/PrivateFrameworks/SkyLight.framework/SkyLight".as_ptr(), RTLD_LAZY) as usize }) as *mut c_void
}

fn symbol(name: &std::ffi::CStr) -> Option<*mut c_void> {
    let handle = skylight();
    // RTLD_DEFAULT when SkyLight would not open: CoreGraphics re-exports most.
    let handle = if handle.is_null() { -2isize as *mut c_void } else { handle };
    let f = unsafe { dlsym(handle, name.as_ptr()) };
    (!f.is_null()).then_some(f)
}

#[repr(C)]
#[derive(Default)]
struct Psn {
    high: u32,
    low: u32,
}

fn psn(pid: i32) -> Option<Psn> {
    type GetProcessForPid = unsafe extern "C" fn(i32, *mut Psn) -> i32;
    // Carbon's, long deprecated and still the key the window server wants.
    let f = unsafe { dlsym(-2isize as *mut c_void, c"GetProcessForPID".as_ptr()) };
    if f.is_null() {
        return None;
    }
    let f: GetProcessForPid = unsafe { std::mem::transmute(f) };
    let mut psn = Psn::default();
    (unsafe { f(pid, &mut psn) } == 0).then_some(psn)
}

/// The app `pid` brought to the front with window `wid` as its key
/// window, as a click on that window would. False when the calls are
/// missing, for the public way to be tried instead.
pub fn bring_to_front(pid: i32, wid: u32) -> bool {
    type SetFront = unsafe extern "C" fn(*const Psn, u32, u32) -> i32;
    type PostRecord = unsafe extern "C" fn(*const Psn, *const u8) -> i32;
    const USER_GENERATED: u32 = 0x200;
    let (Some(set_front), Some(post)) = (symbol(c"_SLPSSetFrontProcessWithOptions"), symbol(c"SLPSPostEventRecordTo")) else { return false };
    let Some(psn) = psn(pid) else { return false };
    let set_front: SetFront = unsafe { std::mem::transmute(set_front) };
    let post: PostRecord = unsafe { std::mem::transmute(post) };
    if unsafe { set_front(&psn, wid, USER_GENERATED) } != 0 {
        return false;
    }
    // The window made key: two event records, as the window server sends
    // them for a click, with the window's number in them.
    let mut bytes = [0u8; 0xf8];
    bytes[0x04] = 0xf8;
    bytes[0x3a] = 0x10;
    bytes[0x3c..0x40].copy_from_slice(&wid.to_ne_bytes());
    bytes[0x20..0x30].fill(0xff);
    for kind in [0x01, 0x02] {
        bytes[0x08] = kind;
        unsafe { post(&psn, bytes.as_ptr()) };
    }
    true
}

/// The system's own ⌘Tab and ⌘⇧Tab switched on or off. The setting
/// outlives the app, so it is put back on at quit and at every launch.
/// False when the call is missing.
pub fn set_system_switcher(on: bool) -> bool {
    type SetEnabled = unsafe extern "C" fn(u32, bool) -> i32;
    const COMMAND_TAB: u32 = 1;
    const COMMAND_SHIFT_TAB: u32 = 2;
    let Some(f) = symbol(c"CGSSetSymbolicHotKeyEnabled") else { return false };
    let f: SetEnabled = unsafe { std::mem::transmute(f) };
    unsafe { f(COMMAND_TAB, on) == 0 && f(COMMAND_SHIFT_TAB, on) == 0 }
}
