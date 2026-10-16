//! tabflip's item in the menu bar: it has no Dock icon and no window most
//! of the time, so this is where Settings and Quit are found. Settings can
//! hide it (General, Show in menu bar).

use std::cell::RefCell;

use futures::channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2::{define_class, msg_send, sel, MainThreadMarker, MainThreadOnly};
use objc2_app_kit::{NSImage, NSMenu, NSMenuItem, NSStatusBar, NSStatusItem};
use objc2_foundation::{NSObject, NSString};

#[derive(Debug, Clone, Copy)]
pub enum Command {
    Settings,
    Quit,
}

thread_local! {
    static SENDER: RefCell<Option<UnboundedSender<Command>>> = const { RefCell::new(None) };
    /// The item and its menu's target, kept for the app's life.
    static KEPT: RefCell<Option<(Retained<NSStatusItem>, Retained<MenuTarget>)>> = const { RefCell::new(None) };
}

fn send(c: Command) {
    SENDER.with(|s| {
        if let Some(tx) = s.borrow().as_ref() {
            let _ = tx.unbounded_send(c);
        }
    });
}

define_class!(
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "TabflipMenuTarget"]
    struct MenuTarget;

    impl MenuTarget {
        #[unsafe(method(openSettings:))]
        fn open_settings(&self, _sender: Option<&AnyObject>) {
            send(Command::Settings);
        }

        #[unsafe(method(quit:))]
        fn quit(&self, _sender: Option<&AnyObject>) {
            send(Command::Quit);
        }
    }
);

/// The item in the menu bar, shown or not, and what is chosen from its menu.
pub fn install(shown: bool) -> Option<UnboundedReceiver<Command>> {
    let mtm = MainThreadMarker::new()?;
    let (tx, rx) = unbounded();
    SENDER.with(|s| *s.borrow_mut() = Some(tx));

    let target: Retained<MenuTarget> = unsafe { msg_send![MenuTarget::alloc(mtm), init] };
    let item = NSStatusBar::systemStatusBar().statusItemWithLength(-1.0);
    if let Some(button) = item.button(mtm) {
        let symbol = NSImage::imageWithSystemSymbolName_accessibilityDescription(&NSString::from_str("rectangle.stack"), Some(&NSString::from_str("tabflip")));
        match symbol {
            Some(image) => {
                image.setTemplate(true);
                button.setImage(Some(&image));
            }
            None => button.setTitle(&NSString::from_str("⇥")),
        }
    }

    let menu = NSMenu::new(mtm);
    let entry = |title: &str, action, key: &str| {
        let it = unsafe { NSMenuItem::initWithTitle_action_keyEquivalent(NSMenuItem::alloc(mtm), &NSString::from_str(title), Some(action), &NSString::from_str(key)) };
        unsafe { it.setTarget(Some(&target)) };
        it
    };
    menu.addItem(&entry("Settings…", sel!(openSettings:), ","));
    menu.addItem(&NSMenuItem::separatorItem(mtm));
    menu.addItem(&entry("Quit tabflip", sel!(quit:), "q"));
    item.setMenu(Some(&menu));
    item.setVisible(shown);

    KEPT.with(|k| *k.borrow_mut() = Some((item, target)));
    Some(rx)
}

/// Show or hide the item. Hidden, Settings are reached by opening
/// tabflip again (`on_reopen` in main.rs).
pub fn set_visible(shown: bool) {
    KEPT.with(|k| {
        if let Some((item, _)) = k.borrow().as_ref() {
            item.setVisible(shown);
        }
    });
}
