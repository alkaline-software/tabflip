//! The Accessibility API and the few CoreFoundation calls around it,
//! declared by hand as emaki declares its event tap: the crates that wrap
//! them pull in far more than the dozen functions used here.
//!
//! CoreFoundation strings, arrays and dictionaries are the same objects as
//! Foundation's (toll-free bridged), so a value read here is looked at as
//! an `NSString` or `NSArray` rather than through CF's own getters.

use std::ffi::c_void;

use objc2_foundation::{NSArray, NSNumber, NSString};

pub type Ref = *mut c_void;

#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    fn AXIsProcessTrusted() -> bool;
    fn AXIsProcessTrustedWithOptions(options: Ref) -> bool;
    static kAXTrustedCheckOptionPrompt: Ref;
    fn AXUIElementCreateApplication(pid: i32) -> Ref;
    fn AXUIElementCopyAttributeValue(element: Ref, attribute: Ref, value: *mut Ref) -> i32;
    fn AXUIElementSetAttributeValue(element: Ref, attribute: Ref, value: Ref) -> i32;
    fn AXUIElementPerformAction(element: Ref, action: Ref) -> i32;
    fn AXUIElementSetMessagingTimeout(element: Ref, seconds: f32) -> i32;
    fn _AXUIElementGetWindow(element: Ref, id: *mut u32) -> i32;
}

#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    fn CFRetain(r: Ref) -> Ref;
    fn CFRelease(r: Ref);
    fn CFDictionaryCreate(alloc: Ref, keys: *const Ref, values: *const Ref, n: isize, key_cb: Ref, value_cb: Ref) -> Ref;
    fn CFBooleanGetValue(b: Ref) -> bool;
    fn CFGetTypeID(r: Ref) -> usize;
    fn CFBooleanGetTypeID() -> usize;
    static kCFBooleanTrue: Ref;
    static kCFBooleanFalse: Ref;
    static kCFTypeDictionaryKeyCallBacks: c_void;
    static kCFTypeDictionaryValueCallBacks: c_void;
}

/// Whether the app may read and move other apps' windows and watch the
/// keyboard: Accessibility, in System Settings.
pub fn trusted() -> bool {
    unsafe { AXIsProcessTrusted() }
}

/// The system's own question, which puts the app in Accessibility's list
/// and offers to open it there.
pub fn ask_trust() -> bool {
    unsafe {
        let keys = [kAXTrustedCheckOptionPrompt];
        let values = [kCFBooleanTrue];
        let options = CFDictionaryCreate(std::ptr::null_mut(), keys.as_ptr(), values.as_ptr(), 1, &kCFTypeDictionaryKeyCallBacks as *const c_void as Ref, &kCFTypeDictionaryValueCallBacks as *const c_void as Ref);
        let ok = AXIsProcessTrustedWithOptions(options);
        CFRelease(options);
        ok
    }
}

/// An owned reference to an Accessibility element: an app or one of its
/// windows. CF objects may be used from any thread.
pub struct Element(Ref);

unsafe impl Send for Element {}
unsafe impl Sync for Element {}

impl Clone for Element {
    fn clone(&self) -> Self {
        Element(unsafe { CFRetain(self.0) })
    }
}

impl Drop for Element {
    fn drop(&mut self) {
        unsafe { CFRelease(self.0) }
    }
}

/// A value an attribute holds, released when dropped.
struct Value(Ref);

impl Drop for Value {
    fn drop(&mut self) {
        unsafe { CFRelease(self.0) }
    }
}

fn name(text: &str) -> objc2::rc::Retained<NSString> {
    NSString::from_str(text)
}

fn as_ref(s: &NSString) -> Ref {
    s as *const NSString as Ref
}

impl Element {
    /// The app with this process id. Every question to it gives up after
    /// a quarter of a second, so an app that hangs cannot stall the
    /// switcher.
    pub fn app(pid: i32) -> Option<Element> {
        let r = unsafe { AXUIElementCreateApplication(pid) };
        if r.is_null() {
            return None;
        }
        unsafe { AXUIElementSetMessagingTimeout(r, 0.25) };
        Some(Element(r))
    }

    fn copy(&self, attribute: &str) -> Option<Value> {
        let attr = name(attribute);
        let mut out: Ref = std::ptr::null_mut();
        let err = unsafe { AXUIElementCopyAttributeValue(self.0, as_ref(&attr), &mut out) };
        (err == 0 && !out.is_null()).then(|| Value(out))
    }

    pub fn string(&self, attribute: &str) -> Option<String> {
        let v = self.copy(attribute)?;
        let obj = unsafe { &*(v.0 as *const objc2::runtime::AnyObject) };
        obj.downcast_ref::<NSString>().map(|s| s.to_string())
    }

    pub fn bool(&self, attribute: &str) -> Option<bool> {
        let v = self.copy(attribute)?;
        unsafe {
            if CFGetTypeID(v.0) == CFBooleanGetTypeID() {
                return Some(CFBooleanGetValue(v.0));
            }
        }
        let obj = unsafe { &*(v.0 as *const objc2::runtime::AnyObject) };
        obj.downcast_ref::<NSNumber>().map(|n| n.boolValue())
    }

    pub fn element(&self, attribute: &str) -> Option<Element> {
        let v = self.copy(attribute)?;
        let r = v.0;
        std::mem::forget(v);
        Some(Element(r))
    }

    /// The elements an attribute lists, such as an app's windows.
    pub fn elements(&self, attribute: &str) -> Vec<Element> {
        let Some(v) = self.copy(attribute) else { return Vec::new() };
        let obj = unsafe { &*(v.0 as *const objc2::runtime::AnyObject) };
        let Some(array) = obj.downcast_ref::<NSArray>() else { return Vec::new() };
        array.iter().map(|item| Element(unsafe { CFRetain(objc2::rc::Retained::as_ptr(&item) as Ref) })).collect()
    }

    pub fn set_bool(&self, attribute: &str, value: bool) -> bool {
        let attr = name(attribute);
        let v = unsafe { if value { kCFBooleanTrue } else { kCFBooleanFalse } };
        unsafe { AXUIElementSetAttributeValue(self.0, as_ref(&attr), v) == 0 }
    }

    pub fn set_element(&self, attribute: &str, value: &Element) -> bool {
        let attr = name(attribute);
        unsafe { AXUIElementSetAttributeValue(self.0, as_ref(&attr), value.0) == 0 }
    }

    pub fn perform(&self, action: &str) -> bool {
        let action = name(action);
        unsafe { AXUIElementPerformAction(self.0, as_ref(&action)) == 0 }
    }

    /// The window server's number for a window: what `CGWindowList` and
    /// SkyLight know it by. A private call, but one every window manager
    /// on the Mac uses.
    pub fn window_id(&self) -> Option<u32> {
        let mut id = 0u32;
        let err = unsafe { _AXUIElementGetWindow(self.0, &mut id) };
        (err == 0 && id != 0).then_some(id)
    }
}
