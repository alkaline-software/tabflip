//! One window brought to the front with the keyboard in it.

use objc2_app_kit::{NSApplicationActivationOptions, NSRunningApplication};

use super::ax::Element;
use super::windows::Window;

pub fn focus(w: &Window) {
    let app = NSRunningApplication::runningApplicationWithProcessIdentifier(w.pid);
    if w.app_hidden {
        if let Some(app) = &app {
            app.unhide();
        }
    }
    if w.minimized {
        w.element.set_bool("AXMinimized", false);
    }
    // The window server's way first: it makes this window the key one even
    // though tabflip, a background app, asks. macOS 14's cooperative
    // activation lets an app in front decline the public request.
    let fronted = w.id.is_some_and(|id| super::private::bring_to_front(w.pid, id));
    w.element.perform("AXRaise");
    w.element.set_bool("AXMain", true);
    if let Some(el) = Element::app(w.pid) {
        el.set_element("AXFocusedWindow", &w.element);
    }
    if !fronted {
        if let Some(app) = &app {
            #[allow(deprecated)]
            app.activateWithOptions(NSApplicationActivationOptions::ActivateIgnoringOtherApps);
        }
    }
}
