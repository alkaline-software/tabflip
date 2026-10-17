//! The keyboard, seen in every app: an event tap on the main run loop.
//!
//! The tap only matches and forwards; anything slow happens in the app.
//! It keeps from the app in front the shortcut's key (and ` with the
//! shortcut's modifiers, which goes back, as ⇧ does), and while the
//! switcher is open every key, so Tab, the arrows and Esc move through the
//! list and do nothing else. Modifiers always pass, so every app agrees on
//! what is held.

use std::ffi::c_void;
use std::sync::atomic::{AtomicBool, AtomicPtr, AtomicU64, Ordering};

use futures::channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
use tabflip_core::shortcut::{self, Shortcut};

#[derive(Debug, Clone, Copy)]
pub enum Key {
    /// The shortcut, with ⇧ when `back`.
    Trigger { back: bool },
    /// Up or down the list.
    Step(i64),
    /// The held modifiers were let go, or Return pressed: pick.
    Release,
    Escape,
    /// What was pressed while Settings records a shortcut.
    Recorded { flags: u64, key: u16 },
}

type Ref = *mut c_void;
type TapCallback = extern "C" fn(proxy: Ref, kind: u32, event: Ref, info: Ref) -> Ref;

#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    fn CGEventTapCreate(tap: u32, place: u32, options: u32, mask: u64, callback: TapCallback, info: Ref) -> Ref;
    fn CGEventTapEnable(tap: Ref, enable: bool);
    fn CGEventGetFlags(event: Ref) -> u64;
    fn CGEventGetIntegerValueField(event: Ref, field: u32) -> i64;
}
#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    fn CFMachPortCreateRunLoopSource(alloc: Ref, port: Ref, order: isize) -> Ref;
    fn CFRunLoopGetMain() -> Ref;
    fn CFRunLoopAddSource(rl: Ref, source: Ref, mode: Ref);
    fn CFRelease(r: Ref);
    static kCFRunLoopCommonModes: Ref;
}

const SESSION_TAP: u32 = 1;
const HEAD_INSERT: u32 = 0;
const KEY_DOWN: u32 = 10;
const KEY_UP: u32 = 11;
const FLAGS_CHANGED: u32 = 12;
const DISABLED_BY_TIMEOUT: u32 = 0xFFFF_FFFE;
const DISABLED_BY_USER: u32 = 0xFFFF_FFFF;
const KEYCODE_FIELD: u32 = 9;

static PORT: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
/// The shortcut: its held modifiers above bit 16, its key below.
static SHORTCUT: AtomicU64 = AtomicU64::new(0);
/// The switcher is open: from the shortcut's press to its pick or cancel.
static OPEN: AtomicBool = AtomicBool::new(false);
/// Settings is waiting for a shortcut to be pressed.
static RECORDING: AtomicBool = AtomicBool::new(false);

pub fn set_shortcut(s: Shortcut) {
    SHORTCUT.store(s.hold << 16 | s.key as u64, Ordering::Relaxed);
}

pub fn set_recording(on: bool) {
    RECORDING.store(on, Ordering::Relaxed);
}

/// The switcher closed for a reason of the app's (a click), so the tap
/// stops holding keys back.
pub fn closed() {
    OPEN.store(false, Ordering::Relaxed);
}

fn send(info: Ref, key: Key) {
    let tx = unsafe { &*(info as *const UnboundedSender<Key>) };
    let _ = tx.unbounded_send(key);
}

extern "C" fn seen(_proxy: Ref, kind: u32, event: Ref, info: Ref) -> Ref {
    if kind == DISABLED_BY_TIMEOUT || kind == DISABLED_BY_USER {
        let port = PORT.load(Ordering::Relaxed);
        if !port.is_null() {
            unsafe { CGEventTapEnable(port, true) };
        }
        return event;
    }
    let flags = unsafe { CGEventGetFlags(event) };
    let packed = SHORTCUT.load(Ordering::Relaxed);
    let (hold, key) = (packed >> 16, (packed & 0xFFFF) as u16);
    let open = OPEN.load(Ordering::Relaxed);
    match kind {
        FLAGS_CHANGED => {
            if open && flags & hold != hold {
                OPEN.store(false, Ordering::Relaxed);
                send(info, Key::Release);
            }
            event
        }
        KEY_DOWN => {
            let code = unsafe { CGEventGetIntegerValueField(event, KEYCODE_FIELD) } as u16;
            let mods = flags & shortcut::MODS;
            if RECORDING.load(Ordering::Relaxed) {
                send(info, Key::Recorded { flags: mods, key: code });
                return std::ptr::null_mut();
            }
            // ` with the held modifiers is the shortcut backwards, unless
            // the shortcut's own key is `.
            let grave = code == shortcut::GRAVE && key != shortcut::GRAVE;
            let is_trigger = (code == key || grave) && hold != 0 && mods & !shortcut::SHIFT == hold;
            let back = mods & shortcut::SHIFT != 0 || grave;
            // Open with the modifiers no longer held: their release was
            // missed (the tap was off for a moment). Pick, and let the key
            // through, rather than keep every key from every app.
            if open && flags & hold != hold {
                OPEN.store(false, Ordering::Relaxed);
                send(info, Key::Release);
                return event;
            }
            if !open {
                if !is_trigger {
                    return event;
                }
                OPEN.store(true, Ordering::Relaxed);
                send(info, Key::Trigger { back });
                return std::ptr::null_mut();
            }
            let said = match code {
                _ if code == key || grave => Some(Key::Trigger { back }),
                shortcut::ESCAPE => Some(Key::Escape),
                shortcut::RETURN | shortcut::KEYPAD_ENTER => Some(Key::Release),
                shortcut::UP | shortcut::LEFT => Some(Key::Step(-1)),
                shortcut::DOWN | shortcut::RIGHT => Some(Key::Step(1)),
                _ => None,
            };
            if let Some(k) = said {
                if matches!(k, Key::Escape | Key::Release) {
                    OPEN.store(false, Ordering::Relaxed);
                }
                send(info, k);
            }
            std::ptr::null_mut()
        }
        // A key let go while open is the switcher's too, or the app in
        // front would see a release with no press.
        KEY_UP if open || RECORDING.load(Ordering::Relaxed) => std::ptr::null_mut(),
        _ => event,
    }
}

/// The tap, on the main run loop for the app's life, or None when the
/// system refuses it: Accessibility is not granted.
pub fn start() -> Option<UnboundedReceiver<Key>> {
    if !PORT.load(Ordering::Relaxed).is_null() {
        return None;
    }
    let (tx, rx) = unbounded();
    let mask = [KEY_DOWN, KEY_UP, FLAGS_CHANGED].iter().fold(0u64, |m, t| m | 1 << t);
    let info = Box::into_raw(Box::new(tx)) as Ref;
    let port = unsafe { CGEventTapCreate(SESSION_TAP, HEAD_INSERT, 0, mask, seen, info) };
    if port.is_null() {
        drop(unsafe { Box::from_raw(info as *mut UnboundedSender<Key>) });
        return None;
    }
    PORT.store(port, Ordering::Relaxed);
    unsafe {
        let source = CFMachPortCreateRunLoopSource(std::ptr::null_mut(), port, 0);
        CFRunLoopAddSource(CFRunLoopGetMain(), source, kCFRunLoopCommonModes);
        CFRelease(source);
        CGEventTapEnable(port, true);
    }
    Some(rx)
}
