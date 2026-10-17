//! tabflip's item in the menu bar: it has no Dock icon and no window most
//! of the time, so this is where Settings and Quit are found.
//!
//! The item is the icon's stack as a template image
//! (`assets/icons/menubar.png`, drawn by `scripts/make-icon.sh`), which
//! the system tints. A left click opens Settings where they were left,
//! and a right click (or Control and a click) shows its menu: the
//! sections, each with the rail's icon, then Quit. General can swap the
//! two clicks (Swap clicks) and hide the item (Show in menu bar); both
//! change at once (`set`).
//!
//! The item has no menu of its own: the button sends both clicks to an
//! Objective-C target, which reads the click from the app's current event
//! and either sends Settings down the channel or sets the menu, clicks
//! the button and takes the menu off again once it closes (`show_menu`).
//! An item with a menu set would show it for either click. The menu's
//! choices call the same target, each known by its tag.
//!
//! The menu's icons are the window's Tabler SVGs, drawn by resvg at one
//! and two pixels a point into a template image, so the system tints them
//! for the light, the dark and the highlighted line. An app built on the
//! macOS 27 SDK has its menus' images hidden unless each item asks for its
//! own (`preferredImageVisibility`); without that the menu shows names
//! alone.

use std::cell::RefCell;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;

use futures::channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
use objc2::rc::Retained;
use objc2::runtime::{AnyObject, NSObjectProtocol as _};
use objc2::{define_class, msg_send, sel, AllocAnyThread as _, MainThreadMarker, MainThreadOnly};
use objc2_app_kit::{NSApplication, NSBitmapImageRep, NSEventMask, NSEventModifierFlags, NSEventType, NSImage, NSMenu, NSMenuItem, NSStatusBar, NSStatusItem, NSVariableStatusItemLength};
use objc2_foundation::{NSData, NSObject, NSSize, NSString};

/// A line of the menu: its key (a section's, or `quit`), its words, the
/// icon before them, its key equivalent, and whether a line stands over it.
pub struct MenuLine {
    pub key: &'static str,
    pub title: &'static str,
    pub icon: &'static [u8],
    pub keys: &'static str,
    pub gap: bool,
}

/// What the item sent: the click that opens Settings, or a line of the
/// menu by its key.
#[derive(Debug, Clone, Copy)]
pub enum MenuPick {
    Settings,
    Line(&'static str),
}

static PICKS: OnceLock<UnboundedSender<MenuPick>> = OnceLock::new();
static KEYS: OnceLock<Vec<&'static str>> = OnceLock::new();
/// A left click shows the menu, and a right click opens Settings.
static MENU_ON_LEFT: AtomicBool = AtomicBool::new(false);

thread_local! {
    /// The item and its menu, kept for the app's life.
    static ITEM: RefCell<Option<(Retained<NSStatusItem>, Retained<NSMenu>)>> = const { RefCell::new(None) };
    /// What the item and the menu call, which they hold weakly.
    static TARGET: RefCell<Option<Retained<MenuTarget>>> = const { RefCell::new(None) };
}

fn send(pick: MenuPick) {
    if let Some(tx) = PICKS.get() {
        let _ = tx.unbounded_send(pick);
    }
}

define_class!(
    // The item and the menu's choices call here, on the main thread, and
    // what was asked goes on to the app as a message.
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "TabflipMenuTarget"]
    struct MenuTarget;

    impl MenuTarget {
        /// A click on the item, either button. A left click with Control
        /// held is a right one, as everywhere on a Mac.
        #[unsafe(method(clicked:))]
        fn clicked(&self, _sender: Option<&AnyObject>) {
            let mtm = self.mtm();
            let right = NSApplication::sharedApplication(mtm).currentEvent().is_some_and(|e| e.r#type() == NSEventType::RightMouseUp || e.modifierFlags().contains(NSEventModifierFlags::Control));
            if right == MENU_ON_LEFT.load(Ordering::Relaxed) {
                send(MenuPick::Settings);
                return;
            }
            show_menu(mtm);
        }

        #[unsafe(method(pick:))]
        fn pick(&self, sender: Option<&NSMenuItem>) {
            let key = sender.and_then(|s| KEYS.get()?.get(usize::try_from(s.tag()).ok()?).copied());
            if let Some(key) = key {
                send(MenuPick::Line(key));
            }
        }
    }
);

/// The menu, under the item. It is the item's only while it is shown, so
/// the click that opens Settings reaches the target; the click below
/// returns once the menu has closed.
fn show_menu(mtm: MainThreadMarker) {
    let Some((status, menu)) = ITEM.with(|i| i.borrow().clone()) else { return };
    status.setMenu(Some(&menu));
    if let Some(button) = status.button(mtm) {
        unsafe { button.performClick(None) };
    }
    status.setMenu(None);
}

/// An icon for a menu's line from its SVG, at the menu's 16 points, drawn
/// for one and two pixels a point, as a template the system tints.
fn line_image(svg: &[u8]) -> Option<Retained<NSImage>> {
    let tree = resvg::usvg::Tree::from_data(svg, &resvg::usvg::Options::default()).ok()?;
    let image = NSImage::initWithSize(NSImage::alloc(), NSSize::new(16., 16.));
    for scale in [1u32, 2] {
        let side = 16 * scale;
        let mut pixmap = resvg::tiny_skia::Pixmap::new(side, side)?;
        let k = side as f32 / tree.size().width();
        resvg::render(&tree, resvg::tiny_skia::Transform::from_scale(k, k), &mut pixmap.as_mut());
        let png = pixmap.encode_png().ok()?;
        let rep = NSBitmapImageRep::imageRepWithData(&NSData::with_bytes(&png))?;
        rep.setSize(NSSize::new(16., 16.));
        image.addRepresentation(&rep);
    }
    image.setTemplate(true);
    Some(image)
}

/// The item in the menu bar with its menu of `lines`, shown or not, with
/// its clicks swapped or not, and what is chosen from it.
pub fn install(lines: &[MenuLine], menu_on_left: bool, shown: bool) -> Option<UnboundedReceiver<MenuPick>> {
    let mtm = MainThreadMarker::new()?;
    let (tx, rx) = unbounded();
    let _ = PICKS.set(tx);
    let _ = KEYS.set(lines.iter().map(|l| l.key).collect());
    MENU_ON_LEFT.store(menu_on_left, Ordering::Relaxed);

    let target: Retained<MenuTarget> = unsafe { msg_send![MenuTarget::alloc(mtm), init] };
    let menu = NSMenu::new(mtm);
    for (i, line) in lines.iter().enumerate() {
        if line.gap {
            menu.addItem(&NSMenuItem::separatorItem(mtm));
        }
        let it = unsafe { NSMenuItem::initWithTitle_action_keyEquivalent(NSMenuItem::alloc(mtm), &NSString::from_str(line.title), Some(sel!(pick:)), &NSString::from_str(line.keys)) };
        unsafe { it.setTarget(Some(&target)) };
        it.setTag(i as isize);
        it.setImage(line_image(line.icon).as_deref());
        // An app built on the macOS 27 SDK has its menus' images hidden
        // unless each item asks for its own (`preferredImageVisibility`,
        // `.visible` = 1); before 27 there is no such choice and they are
        // always drawn.
        if it.respondsToSelector(sel!(setPreferredImageVisibility:)) {
            let _: () = unsafe { msg_send![&*it, setPreferredImageVisibility: 1isize] };
        }
        menu.addItem(&it);
    }
    let status = NSStatusBar::systemStatusBar().statusItemWithLength(NSVariableStatusItemLength);
    if let Some(button) = status.button(mtm) {
        let data = NSData::with_bytes(include_bytes!("../../assets/icons/menubar.png"));
        if let Some(image) = NSImage::initWithData(NSImage::alloc(), &data) {
            image.setSize(NSSize::new(18., 18.));
            image.setTemplate(true);
            button.setImage(Some(&image));
        }
        button.setToolTip(Some(&NSString::from_str("tabflip")));
        unsafe {
            button.setTarget(Some(&target));
            button.setAction(Some(sel!(clicked:)));
        }
        button.sendActionOn(NSEventMask::LeftMouseUp | NSEventMask::RightMouseUp);
    }
    status.setVisible(shown);
    ITEM.with(|i| *i.borrow_mut() = Some((status, menu)));
    TARGET.with(|t| *t.borrow_mut() = Some(target));
    Some(rx)
}

/// Which click shows the menu, and whether the item is in the bar at
/// all: both change at once. Hidden, Settings are reached by opening
/// tabflip again (`on_reopen` in main.rs).
pub fn set(menu_on_left: bool, shown: bool) {
    MENU_ON_LEFT.store(menu_on_left, Ordering::Relaxed);
    ITEM.with(|i| {
        if let Some((status, _)) = i.borrow().as_ref() {
            status.setVisible(shown);
        }
    });
}

/// Each choice of the menu named again, by its tag: the language changed.
pub fn retitle(lines: &[MenuLine]) {
    ITEM.with(|i| {
        if let Some((_, menu)) = i.borrow().as_ref() {
            for (tag, line) in lines.iter().enumerate() {
                if let Some(item) = menu.itemWithTag(tag as isize) {
                    item.setTitle(&NSString::from_str(line.title));
                }
            }
        }
    });
}
