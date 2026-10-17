//! Settings, tabflip's only window, drawn as Alkaline Software's other
//! apps draw theirs (`docs/design.md`): a rail of sections at the left,
//! under the window's traffic lights, and the chosen section's groups of
//! rows on cards at the right. General is the shortcut, how the app
//! stands on this Mac and Accessibility; Customize is how the list looks;
//! Updates looks for a newer release; About says who makes it. The
//! window's language is chosen from a list at the right of General's
//! title.
//!
//! Opened from the menu bar item, by opening tabflip again, and by itself
//! while Accessibility is missing. tabflip is in the Dock exactly while
//! the window is open (`sys::in_dock`).

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::time::{Duration, Instant};

use gpui::prelude::FluentBuilder as _;
use gpui::*;
use gpui_component::input::{Input, InputEvent, InputState};
use gpui_component::scroll::ScrollableElement as _;
use gpui_component::tooltip::Tooltip;
use gpui_component::{h_flex, v_flex, ActiveTheme as _, Icon, Root, Sizable as _};
use tabflip_core::look::{self, Size};
use tabflip_core::shortcut::Shortcut;
use tabflip_core::update;

use crate::controller::{self, Controller};
use crate::i18n::{self, t, tf};
use crate::sys;
use crate::ui::{float_shadow, key_caps, page_in, pill, small_pill, swallow_click, Press, MENU_IN, MENU_OUT, TURN};

/// The settings' sections, in the rail's order: key, name (the key of
/// its other languages), icon.
pub const SECTIONS: &[(&str, &str, &str)] = &[
    ("general", "General", "icons/settings.svg"),
    ("customize", "Customize", "icons/sparkles.svg"),
    ("updates", "Updates", "icons/redo-2.svg"),
    ("about", "About", "icons/info.svg"),
];

/// The window's key context: Escape closes a list that is open.
pub const KEY_CONTEXT: &str = "Settings";

const WINDOW_W: Pixels = px(780.);
const WINDOW_H: Pixels = px(560.);
const WINDOW_MIN: gpui::Size<Pixels> = gpui::Size { width: px(680.), height: px(440.) };
/// The rail's width, and the page's margin at either side.
const RAIL_W: Pixels = px(190.);
const PAGE_X: Pixels = px(36.);
/// The window's top, clear for the traffic lights and dragged by.
const TOP_H: Pixels = px(52.);
/// What the widest controls at a row's right take, so those on a page
/// start and end together: the shortcut's field and Reset, the size's
/// choice, the highlight's swatches and field, the preview.
const CONTROL_W: Pixels = px(264.);
const FIELD_W: Pixels = px(168.);
/// A permission's state or button, and the update's button.
const PERMIT_W: Pixels = px(150.);
const UPDATE_W: Pixels = px(190.);
/// The language's list at General's top right.
const LANGUAGE_W: Pixels = px(136.);
const CARD_R: Pixels = px(12.);

pub struct Settings {
    controller: Entity<Controller>,
    focus: FocusHandle,
    login: bool,
    login_error: Option<SharedString>,
    /// A highlight colour of one's own, typed as hex.
    hex: Entity<InputState>,
    hex_error: bool,
    scroll: ScrollHandle,
    /// A press on the window's top or the rail that may become a drag.
    win_move: bool,
    /// Something small that is there or is not, by its name: whether it
    /// is, since when, and how many times it changed (`presence`).
    presences: RefCell<HashMap<String, (bool, Instant, u32)>>,
    /// Each choice: the one it came from, the one chosen, when.
    seg_state: RefCell<HashMap<&'static str, (&'static str, &'static str, Option<Instant>)>>,
    /// Where each choice's segment was drawn last, for the plate to slide.
    seg_bounds: Rc<RefCell<HashMap<String, Bounds<Pixels>>>>,
    /// The language's list: open, by the serial of its opening; the one
    /// going, drawn a moment more; where its field was drawn last.
    pick: Option<u32>,
    pick_gone: Option<(u32, Instant)>,
    pick_serial: u32,
    pick_at: Rc<RefCell<Option<Bounds<Pixels>>>>,
    _subscriptions: Vec<Subscription>,
}

impl Settings {
    fn new(controller: Entity<Controller>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let observe = cx.observe(&controller, |_, _, cx| cx.notify());
        // The field shows the colour in use when it is one of one's own.
        let current = controller.read(cx).config.highlight().to_string();
        let typed = if look::preset(&current).is_none() { current } else { String::new() };
        let hex = cx.new(|cx| InputState::new(window, cx).placeholder("#rrggbb").default_value(typed));
        let changes = cx.subscribe(&hex, |this, hex, e: &InputEvent, cx| {
            if !matches!(e, InputEvent::Change) {
                return;
            }
            let text = hex.read(cx).value().to_string();
            // Kept as soon as it is a whole colour; a part-typed one is only
            // marked once it is as long as a whole one.
            let ok = look::parse_hex(&text).is_some() && this.controller.update(cx, |c, cx| c.set_highlight(&text, cx));
            this.hex_error = !ok && !text.trim().is_empty() && text.trim().trim_start_matches('#').len() >= 6;
            cx.notify();
        });
        // Coming back to the window reads again what may have changed in
        // System Settings meanwhile.
        let back = cx.observe_window_activation(window, |this, window, cx| {
            if window.is_window_active() {
                this.login = sys::launches_at_login();
                cx.notify();
            }
        });
        Settings {
            controller,
            focus: cx.focus_handle(),
            login: sys::launches_at_login(),
            login_error: None,
            hex,
            hex_error: false,
            scroll: ScrollHandle::new(),
            win_move: false,
            presences: RefCell::default(),
            seg_state: RefCell::default(),
            seg_bounds: Rc::default(),
            pick: None,
            pick_gone: None,
            pick_serial: 0,
            pick_at: Rc::default(),
            _subscriptions: vec![observe, changes, back],
        }
    }
}

struct Open(WindowHandle<Root>, Entity<Settings>);

impl Global for Open {}

/// A probe's step, for a picture of a state (`TABFLIP_GO`, main.rs):
/// `pick:language` opens the language's list, `language:<code>` sets
/// the language as a press on the list does, `section:<key>` another
/// section, `dark` and `light` the appearance, `untrusted` the
/// permission drawn as not given (and `asked`, as after one Allow),
/// `record` the shortcut's field waiting for keys, `newer` a newer
/// version found.
pub fn go(step: &str, cx: &mut App) {
    let Some(Open(handle, settings)) = cx.try_global::<Open>().map(|o| Open(o.0, o.1.clone())) else { return };
    match step {
        "pick:language" => settings.update(cx, |s, cx| s.toggle_pick(cx)),
        "dark" | "light" => {
            let mode = if step == "dark" { gpui_component::ThemeMode::Dark } else { gpui_component::ThemeMode::Light };
            let _ = handle.update(cx, |_, window, cx| gpui_component::Theme::change(mode, Some(window), cx));
        }
        "untrusted" | "asked" => controller::get(cx).update(cx, |c, cx| {
            c.listening = false;
            c.asked_trust = step == "asked";
            cx.notify();
        }),
        "record" => controller::get(cx).update(cx, |c, cx| {
            c.recording = true;
            cx.notify();
        }),
        "newer" => controller::get(cx).update(cx, |c, cx| {
            c.update.available = Some("9.9.9".into());
            cx.notify();
        }),
        s if s.starts_with("language:") => controller::get(cx).update(cx, |c, cx| c.set_language(&s["language:".len()..], cx)),
        s if s.starts_with("section:") => show(SECTIONS.iter().find(|(k, _, _)| *k == &s["section:".len()..]).map(|(k, _, _)| *k), cx),
        other => eprintln!("tabflip: no such step: {other}"),
    }
}

/// The Settings window brought forward, opened if it is not, on `section`
/// when one is named, else where it was left.
pub fn show(section: Option<&'static str>, cx: &mut App) {
    let controller = controller::get(cx);
    if let Some(s) = section.filter(|s| SECTIONS.iter().any(|(k, _, _)| k == s)) {
        controller.update(cx, |c, cx| {
            c.section = s;
            cx.notify();
        });
    }
    if let Some(Open(handle, _)) = cx.try_global::<Open>() {
        let handle = *handle;
        if handle.update(cx, |_, window, _| window.activate_window()).is_ok() {
            sys::in_dock(true);
            cx.activate(true);
            return;
        }
    }
    // In the Dock before the window opens, so it opens in front.
    sys::in_dock(true);
    let bounds = Bounds::centered(None, size(WINDOW_W, WINDOW_H), cx);
    let made: Rc<RefCell<Option<Entity<Settings>>>> = Rc::default();
    let keep = made.clone();
    let opened = cx.open_window(
        WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            titlebar: Some(TitlebarOptions { title: Some("tabflip".into()), appears_transparent: true, traffic_light_position: Some(point(px(17.), px(19.))) }),
            window_min_size: Some(WINDOW_MIN),
            is_minimizable: true,
            focus: true,
            // The top of the window is the app's to drag by (`drag_region`):
            // left to AppKit, a press on a control there moved the window.
            app_owns_titlebar_drag: true,
            ..Default::default()
        },
        |window, cx| {
            window.observe_window_appearance(|window, cx| gpui_component::Theme::sync_system_appearance(Some(window), cx)).detach();
            gpui_component::Theme::sync_system_appearance(Some(window), cx);
            let view = cx.new(|cx| Settings::new(controller.clone(), window, cx));
            *keep.borrow_mut() = Some(view.clone());
            cx.new(|cx| Root::new(view, window, cx))
        },
    );
    let view = made.borrow_mut().take();
    match (opened, view) {
        (Ok(handle), Some(view)) => {
            cx.set_global(Open(handle, view));
            cx.activate(true);
        }
        (Ok(_), None) => unreachable!("the window's build made the view"),
        (Err(e), _) => {
            eprintln!("tabflip: could not open Settings: {e}");
            sys::in_dock(false);
        }
    }
}

/// What a closed Settings window leaves: out of the Dock, and a recording
/// given up. Heard for every window that closes, by any way (the red
/// button, ⌘W, which never asks `on_window_should_close`), and acted on
/// only for Settings.
pub fn watch_close(cx: &mut App) {
    cx.on_window_closed(|cx, id| {
        let ours = cx.try_global::<Open>().is_some_and(|Open(h, _)| h.window_id() == id);
        if !ours {
            return;
        }
        cx.remove_global::<Open>();
        sys::in_dock(false);
        let c = controller::get(cx);
        c.update(cx, |c, cx| c.stop_recording(cx));
    })
    .detach();
}

/// ⌘W: Settings closed, when they are the window in front.
pub fn close(cx: &mut App) {
    if let Some(Open(handle, _)) = cx.try_global::<Open>() {
        let handle = *handle;
        cx.defer(move |cx| {
            let _ = handle.update(cx, |_, window, _| window.remove_window());
        });
    }
}

/// `TABFLIP_SHOT_SETTINGS`: a picture of the Settings window.
pub fn shoot(path: &std::path::Path, cx: &mut App) {
    if let Some(Open(handle, _)) = cx.try_global::<Open>() {
        let handle = *handle;
        let _ = handle.update(cx, |_, window, _| match sys::shoot_window(window, path) {
            Ok(()) => println!("{}", path.display()),
            Err(e) => eprintln!("tabflip: {e}"),
        });
    }
}

impl Render for Settings {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();
        let (section, lang_serial) = {
            let c = self.controller.read(cx);
            (c.section, c.lang_serial)
        };
        let rows = match section {
            "customize" => self.customize(window, cx),
            "updates" => self.updates(cx),
            "about" => self.about(cx),
            _ => self.general(cx),
        };
        let title = SECTIONS.iter().find(|(k, _, _)| *k == section).map(|(_, name, _)| t(name)).unwrap_or(t("General"));
        let page = h_flex()
            .size_full()
            .items_stretch()
            .child(self.rail(section, cx))
            .child(
                v_flex()
                    .flex_1()
                    .min_w_0()
                    .min_h_0()
                    // The window's top is dragged by, and holds the
                    // section's name, level with the traffic lights, as the
                    // Mac's own settings do. It fades in with the page. At
                    // General's right, the window's language.
                    .child(
                        Self::drag_region(h_flex().h(TOP_H).flex_shrink_0().w_full().px(PAGE_X).items_center(), cx)
                            .child(page_in(format!("title-{section}"), div().text_size(px(15.)).font_weight(FontWeight::SEMIBOLD).child(title)))
                            .child(div().flex_1())
                            .when(section == "general", |d| d.child(page_in("language-pick", div().w(LANGUAGE_W).child(self.language_field(cx))))),
                    )
                    .child(
                        div().relative().flex_1().min_h_0().child(
                            v_flex()
                                .id("settings-body")
                                .size_full()
                                .overflow_y_scroll()
                                .track_scroll(&self.scroll)
                                .px(PAGE_X)
                                .pt(px(2.))
                                .pb(px(20.))
                                .child(page_in(format!("page-{section}"), v_flex().gap(px(18.)).children(rows))),
                        ),
                    )
                    .vertical_scrollbar(&self.scroll),
            );
        // A new language: the window fades into it, once.
        let page = match lang_serial {
            0 => page.into_any_element(),
            n => page.with_animation(ElementId::Name(format!("language-{n}").into()), Animation::new(Duration::from_millis(260)).with_easing(ease_out_quint()), |d, t| d.opacity(0.3 + 0.7 * t)).into_any_element(),
        };
        div()
            .id("settings")
            .key_context(KEY_CONTEXT)
            .track_focus(&self.focus)
            .relative()
            .size_full()
            .bg(theme.popover)
            .text_color(theme.foreground)
            .on_action(cx.listener(|this, _: &crate::Escape, _, cx| this.close_pick(cx)))
            .child(page)
            .when_some(self.pick_gone.filter(|(_, at)| at.elapsed() < MENU_OUT), |d, (serial, _)| d.child(self.render_pick(serial, true, window, cx)))
            .when_some(self.pick, |d, serial| d.child(self.render_pick(serial, false, window, cx)))
    }
}

impl Settings {
    /// The rail of sections at the window's left, under its traffic
    /// lights: an icon and a name per section, the chosen one on a plate.
    /// General wears a dot while Accessibility is missing, and Updates
    /// while a newer tabflip waits; each comes and goes in motion.
    fn rail(&self, current: &'static str, cx: &mut Context<Self>) -> AnyElement {
        let theme = cx.theme().clone();
        let c = self.controller.read(cx);
        let (wanting, waiting) = (!c.listening, c.update.available.is_some());
        Self::drag_region(
            v_flex()
                .w(RAIL_W)
                .h_full()
                .flex_shrink_0()
                .pt(TOP_H)
                .px(px(10.))
                .gap(px(2.))
                .bg(if theme.mode.is_dark() { theme.sidebar } else { theme.muted.opacity(0.5) })
                .border_r_1()
                .border_color(theme.border),
            cx,
        )
        .children(SECTIONS.iter().map(|&(key, label, icon)| {
            let active = key == current;
            let dot = (key == "general" && wanting) || (key == "updates" && waiting);
            let dot = self.fade(&format!("dot-{key}"), dot, div().size(px(7.)).flex_shrink_0().rounded_full().bg(theme.primary), cx);
            h_flex()
                .id(SharedString::from(format!("rail-{key}")))
                .h(px(30.))
                .px(px(10.))
                .gap(px(9.))
                .items_center()
                .rounded(px(8.))
                .cursor_pointer()
                .when(active, |d| d.bg(theme.sidebar_accent))
                .when(!active, |d| d.hover(|s| s.bg(theme.sidebar_accent.opacity(0.6))))
                .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.close_pick(cx);
                    this.controller.update(cx, |c, cx| {
                        c.section = key;
                        cx.notify();
                    });
                    this.scroll.set_offset(point(px(0.), px(0.)));
                    cx.notify();
                }))
                .child(Icon::default().path(icon).with_size(px(15.)).text_color(if active { theme.foreground } else { theme.muted_foreground }))
                .child(div().flex_1().text_size(px(13.)).when(active, |d| d.font_weight(FontWeight::MEDIUM)).text_color(if active { theme.foreground } else { theme.sidebar_foreground }).child(t(label)))
                .child(dot)
        }))
        .into_any_element()
    }

    /// General: the shortcut, how the app stands on this Mac, and the one
    /// permission.
    fn general(&mut self, cx: &mut Context<Self>) -> Vec<AnyElement> {
        let theme = cx.theme().clone();
        let c = self.controller.read(cx);
        let (listening, recording) = (c.listening, c.recording);
        let shortcut = c.config.shortcut();
        let (shown, swapped, asked) = (c.config.menu_bar_icon, c.config.menu_bar_swap, c.asked_trust);
        let (note, wrong): (&'static str, bool) = if let Some(e) = &c.record_error {
            (t(e.said()), true)
        } else if recording {
            (t("Press the keys, or Esc to keep the old one."), false)
        } else if !listening {
            (t("Allow Accessibility below first."), false)
        } else if c.system_switcher_stuck {
            (t("macOS kept its own ⌘Tab; try ⌥Tab."), true)
        } else if shortcut.is_command_tab() {
            (t("Replaces the Mac's own ⌘Tab while tabflip runs."), false)
        } else {
            (t("Hold it, press again to move down, let go to switch."), false)
        };

        let field = div()
            .id("shortcut")
            .w(FIELD_W)
            .h(px(28.))
            .px(px(10.))
            .flex()
            .items_center()
            .justify_center()
            .rounded(px(8.))
            .bg(theme.muted)
            .border_1()
            .border_color(if recording { theme.primary } else { gpui::transparent_black() })
            .text_size(px(12.5))
            .map(|d| if recording { d.text_color(theme.muted_foreground).child(t("Press keys…")) } else { d.child(key_caps(&shortcut.to_string())) })
            .map(|d| {
                if listening {
                    d.cursor_pointer().on_click(cx.listener(|this, _, window, cx| {
                        swallow_click(window, cx);
                        this.controller.update(cx, |c, cx| if c.recording { c.stop_recording(cx) } else { c.start_recording(cx) });
                    }))
                } else {
                    d.opacity(0.5)
                }
            });
        let unchanged = shortcut == Shortcut::default() || recording;
        let reset = pill("reset", "icons/rotate.svg", t("Reset"), Press::Quiet, unchanged, &theme).flex_1().when(!unchanged, |d| {
            d.on_click(cx.listener(|this, _, window, cx| {
                swallow_click(window, cx);
                this.controller.update(cx, |c, cx| {
                    c.set_shortcut(Shortcut::default());
                    cx.notify();
                });
            }))
        });
        let shortcut_control = h_flex().w(CONTROL_W).gap(px(8.)).child(field).child(reset).into_any_element();
        let shortcut_note = div().when(wrong, |d| d.text_color(theme.danger)).child(note).into_any_element();

        let login_note = self.login_error.clone().map(|e| div().text_color(theme.danger).child(e).into_any_element());
        let login = self.switch("login", self.login, Rc::new(|this, on, cx| {
            match sys::set_launch_at_login(on) {
                Ok(()) => this.login_error = None,
                Err(e) => this.login_error = Some(e.into()),
            }
            this.login = sys::launches_at_login();
            cx.notify();
        }), cx);
        let menu_bar = self.switch("menu-bar", shown, Rc::new(|this, on, cx| this.controller.update(cx, |c, cx| c.set_menu_bar_icon(on, cx))), cx);
        let swap = self.switch("menu-swap", swapped, Rc::new(|this, on, cx| this.controller.update(cx, |c, cx| c.set_menu_bar_swap(on, cx))), cx);

        // Given: said so. Not given: Allow, the system's own question,
        // once; then the way to System Settings. The change plays once.
        let access = if listening {
            h_flex().h(px(28.)).w_full().gap(px(6.)).items_center().justify_center().rounded(px(8.)).bg(theme.success.opacity(0.1)).text_size(px(12.5)).text_color(theme.success).child(Icon::default().path("icons/check.svg").with_size(px(14.))).child(t("Allowed")).into_any_element()
        } else {
            let (icon, word) = if asked { ("icons/external-link.svg", t("Open Settings")) } else { ("icons/shield.svg", t("Allow")) };
            pill("allow", icon, word, Press::Primary, false, &theme)
                .w_full()
                .on_click(cx.listener(|this, _, window, cx| {
                    swallow_click(window, cx);
                    this.controller.update(cx, |c, cx| c.ask_trust(cx));
                }))
                .into_any_element()
        };
        let access = self.arrive("access", listening, div().w(PERMIT_W).child(access), cx);
        let access_note: &'static str = if listening { t("To see every window and switch to it.") } else { t("Turn tabflip on in the list; it starts by itself.") };

        vec![
            group(Some(t("Switcher")), vec![line(t("Shortcut"), Some(shortcut_note), shortcut_control, &theme)], &theme),
            group(
                Some(t("App")),
                vec![
                    line(t("Launch at login"), login_note, login, &theme),
                    line(t("Show in menu bar"), Some(div().child(t("When hidden, open tabflip again to come back here.")).into_any_element()), menu_bar, &theme),
                    line(t("Swap clicks"), Some(div().child(t("The menu on a left click, Settings on a right one.")).into_any_element()), swap, &theme),
                ],
                &theme,
            ),
            group(Some(t("Permissions")), vec![line(t("Accessibility"), Some(div().child(access_note).into_any_element()), access, &theme)], &theme),
        ]
    }

    /// Customize: the list's size and the selected row's colour, with a
    /// row as the list will draw it.
    fn customize(&mut self, window: &mut Window, cx: &mut Context<Self>) -> Vec<AnyElement> {
        let theme = cx.theme().clone();
        let c = self.controller.read(cx);
        let size_now = c.config.size();
        let highlight = c.config.highlight().to_string();

        let sizes = div()
            .w(CONTROL_W)
            .child(self.segmented(
                "size",
                Size::ALL.iter().map(|s| (s.name(), t(s.name()))).collect(),
                size_now.name(),
                Rc::new(|this, key, cx| {
                    let size = Size::from_config(key);
                    this.controller.update(cx, |c, cx| c.set_size(size, cx));
                }),
                cx,
            ))
            .into_any_element();

        // The swatches in the appearance the list will be drawn in, each
        // named on a hover.
        let dark = matches!(window.appearance(), WindowAppearance::Dark | WindowAppearance::VibrantDark);
        let swatches = h_flex()
            .w(CONTROL_W)
            .justify_between()
            .children(look::PRESETS.iter().map(|p| {
                let on = p.name.eq_ignore_ascii_case(&highlight);
                let name = p.name;
                div()
                    .id(name)
                    .size(px(26.))
                    .p(px(2.))
                    .rounded_full()
                    .border_2()
                    .border_color(if on { theme.foreground.opacity(0.7) } else { gpui::transparent_black() })
                    .cursor_pointer()
                    .tooltip(move |window, cx| Tooltip::new(t(name)).build(window, cx))
                    .on_click(cx.listener(move |this, _, window, cx| {
                        swallow_click(window, cx);
                        this.controller.update(cx, |c, cx| c.set_highlight(name, cx));
                        this.hex.update(cx, |h, cx| h.set_value("", window, cx));
                        this.hex_error = false;
                    }))
                    .child(div().size_full().rounded_full().bg(rgb(if dark { p.dark } else { p.light })))
            }))
            .into_any_element();

        let own = look::preset(&highlight).is_none();
        let hex = h_flex()
            .w(CONTROL_W)
            .h(px(28.))
            .px(px(10.))
            .gap(px(8.))
            .items_center()
            .rounded(px(8.))
            .bg(theme.muted)
            .border_1()
            .border_color(if self.hex_error { theme.danger } else if own { theme.primary } else { gpui::transparent_black() })
            .text_size(px(12.5))
            .child(div().size(px(14.)).flex_shrink_0().rounded_full().bg(rgb(look::highlight(&highlight, dark))).when(!own, |d| d.opacity(0.)))
            .child(div().flex_1().min_w_0().cursor_text().child(Input::new(&self.hex).appearance(false).bordered(false).small()))
            .into_any_element();
        let hex_note = if self.hex_error { Some(div().text_color(theme.danger).child(t("Not a colour: #rrggbb or #rgb.")).into_any_element()) } else { Some(div().child(t("Any colour, as hex.")).into_any_element()) };

        // A row as the list draws it, at the chosen size and colour.
        let preview = {
            let k = size_now.scale();
            let pick = look::highlight(&highlight, dark);
            let text = if look::dark_text_on(pick) { rgb(0x1d1d1f).into() } else { gpui::white() };
            h_flex()
                .w(CONTROL_W)
                .h(px(30. * k))
                .px(px(9. * k))
                .gap(px(8. * k))
                .rounded(px(7. * k))
                .bg(rgb(pick))
                .text_color(text)
                .child(div().size(px(16. * k)).flex_shrink_0().rounded(px(4. * k)).bg(text.opacity(0.35)))
                .child(div().text_size(px(13. * k)).child("Gmail"))
                .child(div().ml_auto().text_size(px(11. * k)).opacity(0.75).child("Google Chrome"))
                .into_any_element()
        };

        vec![group(
            Some(t("The list")),
            vec![
                line(t("Size"), None, sizes, &theme),
                line(t("Highlight"), None, swatches, &theme),
                line(t("Your own colour"), hex_note, hex, &theme),
                line(t("Preview"), None, preview, &theme),
            ],
            &theme,
        )]
    }

    /// Updates: the version, with what a check found and its button, and
    /// whether to look once a day. tabflip does not install anything
    /// itself: a newer version's installer is offered to download.
    fn updates(&mut self, cx: &mut Context<Self>) -> Vec<AnyElement> {
        let theme = cx.theme().clone();
        let c = self.controller.read(cx);
        let auto_on = c.config.check_updates;
        let u = &c.update;
        let (checking, failed, answered, available) = (u.checking, u.failed, u.answered, u.available.clone());
        let current = update::current_version();
        let said = if checking {
            ""
        } else if available.is_some() {
            t("A newer version is out.")
        } else if failed {
            t("Could not reach GitHub.")
        } else if answered {
            t("Up to date.")
        } else {
            ""
        };
        let version_line: SharedString = if said.is_empty() { format!("tabflip {current}") } else { format!("tabflip {current} · {said}") }.into();
        let link = |id: &'static str, label: &'static str, url: String| {
            let (ink, hover) = (theme.muted_foreground, theme.primary);
            div()
                .id(id)
                .self_start()
                .cursor_pointer()
                .text_color(ink)
                .text_decoration_1()
                .text_decoration_color(ink.opacity(0.6))
                .hover(move |s| s.text_color(hover).text_decoration_color(hover))
                .on_click(move |_, _, _| sys::open_url(&url))
                .child(label)
        };
        let version_note = v_flex().gap(px(1.)).child(div().child(version_line)).when_some(available.clone(), |d, v| d.child(link("update-notes", t("Release notes"), update::release_page(&v)))).into_any_element();
        let button = match available.clone() {
            Some(v) => pill("update-get", "icons/download.svg", tf("Download {v}", &[("v", &v)]), Press::Primary, false, &theme).w(UPDATE_W).on_click(move |_, window, cx| {
                swallow_click(window, cx);
                sys::open_url(&update::asset_url(&v));
            }),
            None => pill("update-check", "icons/redo-2.svg", if checking { t("Checking…") } else { t("Check for updates") }, Press::Quiet, checking, &theme).w(UPDATE_W).when(!checking, |d| {
                d.on_click(cx.listener(|this, _, window, cx| {
                    swallow_click(window, cx);
                    this.controller.update(cx, |c, cx| c.check_updates(false, cx));
                }))
            }),
        }
        .into_any_element();
        let auto = self.switch("update-auto", auto_on, Rc::new(|this, on, cx| this.controller.update(cx, |c, cx| c.set_check_updates(on, cx))), cx);
        let downloaded = available.is_some().then(|| div().px(px(14.)).text_size(px(11.5)).text_color(theme.muted_foreground).child(t("Open the downloaded disk image and drag tabflip to Applications, over the old one.")).into_any_element());
        let mut out = vec![group(
            None,
            vec![
                line(t("Version"), Some(version_note), button, &theme),
                line(t("Check automatically"), Some(div().child(t("Once a day. Nothing is installed without you.")).into_any_element()), auto, &theme),
            ],
            &theme,
        )];
        out.extend(downloaded);
        out
    }

    /// About: the title card, set as a film's first one is, and who makes
    /// tabflip: Alkaline Software.
    fn about(&mut self, cx: &mut Context<Self>) -> Vec<AnyElement> {
        let theme = cx.theme().clone();
        let current = update::current_version();
        let hover = theme.primary;
        let link = |id: &'static str, name: &'static str, url: &'static str| {
            div()
                .id(id)
                .cursor_pointer()
                .text_color(theme.foreground)
                .text_decoration_1()
                .text_decoration_color(theme.foreground.opacity(0.35))
                .hover(move |s| s.text_color(hover).text_decoration_color(hover))
                .on_click(move |_, _, _| sys::open_url(url))
                .child(name)
        };
        let prose = || h_flex().flex_wrap().justify_center().text_size(px(13.5)).line_height(px(22.)).text_color(theme.muted_foreground);
        let go = |id: &'static str, icon: &'static str, label: &'static str, url: String| small_pill(id, icon, label, Press::Quiet, false, &theme).on_click(move |_, _, _| sys::open_url(&url));
        let site = update::repo_page();
        let open_site = {
            let site = site.clone();
            move |_: &ClickEvent, _: &mut Window, _: &mut App| sys::open_url(&site)
        };
        vec![
            // Who presents, the app, its version.
            v_flex()
                .w_full()
                .items_center()
                .pt(px(30.))
                .pb(px(28.))
                .rounded(CARD_R)
                .border_1()
                .border_color(theme.border)
                .bg(card_ground(&theme))
                .child(weave(t("{studio} presents"), vec![("studio", link("about-studio", "Alkaline Software", "https://github.com/alkaline-software").into_any_element())], prose().text_size(px(12.5))))
                .child(div().id("about-icon").mt(px(18.)).cursor_pointer().hover(|s| s.opacity(0.82)).on_click(open_site.clone()).child(img("icon/app-large.png").size(px(88.)).flex_shrink_0()))
                .child(div().id("about-name").mt(px(6.)).cursor_pointer().hover(|s| s.opacity(0.82)).on_click(open_site).text_size(px(42.)).line_height(px(48.)).text_color(theme.primary).font_family(wordmark_face(cx)).child("tabflip"))
                .child(div().mt(px(4.)).text_size(px(12.5)).text_color(theme.muted_foreground).child(tf("Switch between windows, not apps · Version {v}", &[("v", &current)])))
                .into_any_element(),
            v_flex()
                .w_full()
                .items_center()
                .gap(px(16.))
                .pt(px(8.))
                .child(weave(
                    t("A studio by {jp} and {pingfan}."),
                    vec![("jp", link("about-jp", "Dr. John Paul Helveston", "https://www.jhelvy.com").into_any_element()), ("pingfan", link("about-pingfan", "Dr. Pingfan Hu", "https://pingfan.org").into_any_element())],
                    prose(),
                ))
                .child(h_flex().gap(px(8.)).child(go("about-source", "icons/github.svg", "GitHub", site.clone())).child(go("about-notes", "icons/file-text.svg", t("Release Notes"), update::release_page(current))))
                .into_any_element(),
        ]
    }

    /// How something small stands at this draw, asked at every draw
    /// whether it is there or not, so a change is seen as one: whether it
    /// is coming, and the number of the change, while it moves; nothing
    /// at rest. What was never drawn before is not played. One going is
    /// drawn until its time is up, and once more then as nothing.
    fn presence(&self, id: &str, on: bool, cx: &mut Context<Self>) -> Option<(bool, u32)> {
        let mut all = self.presences.borrow_mut();
        let now = Instant::now();
        let p = all.entry(id.to_string()).or_insert_with(|| (on, now.checked_sub(TURN * 2).unwrap_or(now), 0));
        if p.0 != on {
            *p = (on, now, p.2 + 1);
            if !on {
                cx.spawn(async move |this, cx| {
                    cx.background_executor().timer(TURN + Duration::from_millis(20)).await;
                    let _ = this.update(cx, |_, cx| cx.notify());
                })
                .detach();
            }
        }
        (p.2 > 0 && p.1.elapsed() < TURN).then_some((on, p.2))
    }

    /// Something that is always there but changes face with `state`
    /// (Allow becoming Allowed): the new face fades in, once, when the
    /// state changes (`presence`); the first sight plays nothing.
    fn arrive(&self, id: &str, state: bool, el: Div, cx: &mut Context<Self>) -> AnyElement {
        match self.presence(id, state, cx) {
            Some((_, n)) => el.with_animation(ElementId::Name(format!("{id}-{n}").into()), Animation::new(TURN).with_easing(ease_out_quint()), |d, t| d.opacity(t)).into_any_element(),
            None => el.into_any_element(),
        }
    }

    /// Something whose room stays, there or not: it runs only its
    /// opacity, over a moment, when that changes (`presence`).
    fn fade(&self, id: &str, on: bool, el: Div, cx: &mut Context<Self>) -> AnyElement {
        match self.presence(id, on, cx) {
            Some((coming, n)) => el.with_animation(ElementId::Name(format!("{id}-{n}").into()), Animation::new(TURN).with_easing(ease_out_quint()), move |d, t| d.opacity(if coming { t } else { 1. - t })).into_any_element(),
            None => el.opacity(if on { 1. } else { 0. }).into_any_element(),
        }
    }

    /// A switch, for what is on or off: a track that fills with the accent
    /// and a knob that slides to its end, both over a moment as it turns.
    /// A press anywhere on it turns it.
    #[allow(clippy::type_complexity)]
    fn switch(&self, id: &'static str, on: bool, turn: Rc<dyn Fn(&mut Self, bool, &mut Context<Self>)>, cx: &mut Context<Self>) -> AnyElement {
        const W: f32 = 36.;
        const H: f32 = 20.;
        const KNOB: f32 = 16.;
        const INSET: f32 = 2.;
        let travel = W - KNOB - INSET * 2.;
        let theme = cx.theme().clone();
        let moving = self.presence(&format!("switch-{id}"), on, cx);
        let at = move |t: f32| px(INSET + travel * t);
        let fill = div().absolute().inset_0().rounded_full().bg(theme.primary);
        let knob = div().absolute().top(px(INSET)).size(px(KNOB)).rounded_full().bg(gpui::white()).shadow_sm();
        let (fill, knob) = match moving {
            Some((coming, n)) => (
                fill.with_animation(ElementId::Name(format!("switch-{id}-fill-{n}").into()), Animation::new(TURN).with_easing(ease_out_quint()), move |d, t| d.opacity(if coming { t } else { 1. - t })).into_any_element(),
                knob.with_animation(ElementId::Name(format!("switch-{id}-knob-{n}").into()), Animation::new(TURN).with_easing(ease_out_quint()), move |d, t| d.left(at(if coming { t } else { 1. - t }))).into_any_element(),
            ),
            None => (fill.opacity(if on { 1. } else { 0. }).into_any_element(), knob.left(at(if on { 1. } else { 0. })).into_any_element()),
        };
        div()
            .id(SharedString::from(format!("switch-{id}")))
            .relative()
            .flex_shrink_0()
            .w(px(W))
            .h(px(H))
            .rounded_full()
            .bg(theme.switch)
            .cursor_pointer()
            .on_click(cx.listener(move |this, _, window, cx| {
                swallow_click(window, cx);
                turn(this, !on, cx)
            }))
            .child(fill)
            .child(knob)
            .into_any_element()
    }

    /// A choice among a few: the choices in a row on a muted track, the
    /// chosen one on a raised plate that slides from the old choice to the
    /// new one. The plate is placed from where each segment was drawn last
    /// (`seg_bounds`); on the very first draw, before any segment has been
    /// measured, the chosen segment paints its own plate, so nothing
    /// flashes.
    #[allow(clippy::type_complexity)]
    fn segmented(&self, control: &'static str, options: Vec<(&'static str, &'static str)>, current: &'static str, on: Rc<dyn Fn(&mut Self, &'static str, &mut Context<Self>)>, cx: &mut Context<Self>) -> AnyElement {
        const SLIDE: Duration = Duration::from_millis(220);
        let (pad, seg_h, round) = (px(3.), px(22.), px(6.));
        let theme = cx.theme().clone();
        let dark = theme.mode.is_dark();
        let track_bg = if dark { theme.tab_bar_segmented } else { theme.muted };
        let plate_bg = if dark { theme.secondary_active } else { theme.popover };
        let (from, to) = {
            let mut st = self.seg_state.borrow_mut();
            let e = st.entry(control).or_insert((current, current, None));
            if e.1 != current {
                e.0 = e.1;
                e.1 = current;
                e.2 = Some(Instant::now());
            }
            let sliding = e.2.is_some_and(|at| at.elapsed() < SLIDE + Duration::from_millis(80));
            (if sliding { e.0 } else { e.1 }, e.1)
        };
        let (b_from, b_to) = {
            let b = self.seg_bounds.borrow();
            (b.get(&format!("{control}-{from}")).copied(), b.get(&format!("{control}-{to}")).copied())
        };
        let plate = match (b_from, b_to) {
            (Some(a), Some(b)) => Some(div().absolute().rounded(round).bg(plate_bg).shadow_sm().with_animation(ElementId::Name(format!("{control}-plate-{to}").into()), Animation::new(SLIDE).with_easing(ease_out_quint()), move |d, t| {
                let x = a.origin.x + (b.origin.x - a.origin.x) * t;
                let w = a.size.width + (b.size.width - a.size.width) * t;
                d.left(x).top(b.origin.y).w(w).h(b.size.height)
            })),
            _ => None,
        };
        let measured = plate.is_some();
        let ids: Vec<String> = options.iter().map(|(k, _)| format!("{control}-{k}")).collect();
        let seg_bounds = self.seg_bounds.clone();
        let entity = cx.entity().downgrade();
        let row = h_flex()
            .gap(px(2.))
            .flex_1()
            .min_w_0()
            // Where each segment landed, relative to the track. A change is
            // noted and the next draw places the plate from it.
            .on_children_prepainted(move |bounds, _, cx| {
                let Some(first) = bounds.first() else { return };
                let mut changed = false;
                let mut map = seg_bounds.borrow_mut();
                for (id, b) in ids.iter().zip(bounds.iter()) {
                    let rel = Bounds { origin: point(b.origin.x - first.origin.x + pad, pad), size: b.size };
                    if map.get(id) != Some(&rel) {
                        map.insert(id.clone(), rel);
                        changed = true;
                    }
                }
                drop(map);
                if changed {
                    let _ = entity.update(cx, |_, cx| cx.notify());
                }
            })
            .children(options.into_iter().map(|(key, label)| {
                let active = key == current;
                let on = on.clone();
                h_flex()
                    .id(SharedString::from(format!("{control}-{key}")))
                    .h(seg_h)
                    .px(px(11.))
                    .flex_1()
                    .items_center()
                    .justify_center()
                    .rounded(round)
                    .cursor_pointer()
                    .text_size(px(12.5))
                    .when(active && !measured, |d| d.bg(plate_bg).shadow_sm())
                    .when(active, |d| d.font_weight(FontWeight::MEDIUM).text_color(theme.foreground))
                    .when(!active, |d| d.text_color(theme.muted_foreground).hover(|s| s.text_color(theme.foreground)))
                    .on_click(cx.listener(move |this, _, window, cx| {
                        swallow_click(window, cx);
                        on(this, key, cx)
                    }))
                    .child(label)
            }));
        h_flex().id(SharedString::from(format!("{control}-track"))).relative().w_full().p(pad).rounded(round + px(2.)).bg(track_bg).children(plate).child(row).into_any_element()
    }

    /// The field at General's title's right that opens the language's
    /// list: the language by the system's, or by its flag and letters,
    /// in the middle, and the mark of a list at the right. A press on it
    /// is not a drag of the top.
    fn language_field(&self, cx: &mut Context<Self>) -> AnyElement {
        let theme = cx.theme().clone();
        let language = self.controller.read(cx).config.language.clone();
        let (icon, label) = match i18n::CHOICES.iter().find(|(c, _, _)| *c == language) {
            Some((_, flag, letters)) => (None, format!("{flag}  {letters}")),
            None => (Some("icons/globe.svg"), t("System").to_string()),
        };
        let open = self.pick.is_some();
        let over = theme.muted_foreground.opacity(0.16);
        let at = self.pick_at.clone();
        div()
            .relative()
            .w_full()
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .child(
                h_flex()
                    .id("language-trigger")
                    .h(px(28.))
                    .px(px(10.))
                    .gap(px(8.))
                    .items_center()
                    .rounded(px(8.))
                    .bg(theme.muted)
                    .cursor_pointer()
                    .text_size(px(12.5))
                    .when(open, |d| d.bg(over))
                    .hover(move |s| s.bg(over))
                    .on_click(cx.listener(|this, _, window, cx| {
                        swallow_click(window, cx);
                        this.toggle_pick(cx);
                    }))
                    .child(
                        // As far from both ends as the list's mark is from
                        // its own, so the words sit in the middle.
                        h_flex().flex_1().min_w_0().px(px(18.)).gap(px(7.)).items_center().justify_center().children(icon.map(|icon| Icon::default().path(icon).with_size(px(14.)).text_color(theme.muted_foreground))).child(div().min_w_0().truncate().child(label)),
                    )
                    .child(div().absolute().right(px(10.)).top_0().bottom_0().flex().items_center().child(Icon::default().path("icons/chevrons-up-down.svg").with_size(px(13.)).text_color(theme.muted_foreground))),
            )
            // Where the field is drawn, for the list to hang off it.
            .child(canvas(move |b, _, _| *at.borrow_mut() = Some(b), |_, _, _, _| {}).absolute().inset_0())
            .into_any_element()
    }

    /// A press on the language's field: the list opens, or, open, closes.
    fn toggle_pick(&mut self, cx: &mut Context<Self>) {
        if self.pick.is_some() {
            return self.close_pick(cx);
        }
        self.pick_serial += 1;
        self.pick_gone = None;
        self.pick = Some(self.pick_serial);
        cx.notify();
    }

    /// The list is put away: it fades where it was, taking no click.
    fn close_pick(&mut self, cx: &mut Context<Self>) {
        if let Some(serial) = self.pick.take() {
            self.pick_gone = Some((serial, Instant::now()));
            cx.spawn(async move |this, cx| {
                cx.background_executor().timer(MENU_OUT + Duration::from_millis(20)).await;
                let _ = this.update(cx, |this, cx| {
                    if this.pick_gone.is_some_and(|(_, at)| at.elapsed() >= MENU_OUT) {
                        this.pick_gone = None;
                    }
                    cx.notify();
                });
            })
            .detach();
        }
        cx.notify();
    }

    /// The language's list, hanging under its field with its right edge
    /// on the field's: System first, then each language by its flag and
    /// letters, the one in use ticked. It comes in over a moment, rising
    /// into place, and goes out fading, over a clear sheet any click puts
    /// away.
    fn render_pick(&self, serial: u32, gone: bool, window: &Window, cx: &mut Context<Self>) -> AnyElement {
        let theme = cx.theme().clone();
        let Some(by) = *self.pick_at.borrow() else { return div().into_any_element() };
        let view = window.viewport_size();
        let current = self.controller.read(cx).config.language.clone();
        let dark = theme.mode.is_dark();
        let lit = if dark { theme.foreground.opacity(0.11) } else { theme.list_hover };
        let chosen = if dark { theme.foreground.opacity(0.06) } else { theme.muted };
        let choices: Vec<(&'static str, String, bool)> = std::iter::once(("system", t("System").to_string(), false)).chain(i18n::CHOICES.iter().enumerate().map(|(ix, (code, flag, letters))| (*code, format!("{flag}  {letters}"), ix == 0))).collect();
        let mut rows = v_flex().px(px(5.)).gap(px(2.));
        for (ix, (code, name, rule)) in choices.into_iter().enumerate() {
            let active = code == current;
            if rule {
                rows = rows.child(div().flex_shrink_0().h(px(1.)).mx(px(6.)).my(px(3.)).bg(theme.border));
            }
            rows = rows.child(
                h_flex()
                    .id(("language-row", ix))
                    .flex_shrink_0()
                    .px(px(10.))
                    .py(px(6.))
                    .gap(px(6.))
                    .items_center()
                    .rounded(px(8.))
                    .cursor_pointer()
                    .when(active, |d| d.bg(chosen))
                    .hover(move |s| s.bg(lit))
                    .when(!gone, |d| {
                        d.on_click(cx.listener(move |this, _, _, cx| {
                            this.close_pick(cx);
                            if !active {
                                this.controller.update(cx, |c, cx| c.set_language(code, cx));
                            }
                        }))
                    })
                    .child(div().text_size(px(13.)).font_weight(FontWeight::MEDIUM).child(name))
                    .when(active, |d| d.child(div().flex_1()).child(Icon::default().path("icons/check.svg").with_size(px(12.)).text_color(theme.primary))),
            );
        }
        let rest = by.bottom() + px(6.);
        let card = v_flex()
            .id(("language-card", serial as usize))
            .absolute()
            .right(view.width - by.right())
            .min_w(by.size.width)
            .py(px(5.))
            .rounded(px(12.))
            .bg(theme.popover)
            .border_1()
            .border_color(theme.border)
            .shadow(float_shadow(&theme))
            .on_mouse_down(MouseButton::Left, |_, window, cx| swallow_click(window, cx))
            .child(rows);
        if gone {
            let card = card.with_animation(ElementId::Name(format!("language-out-{serial}").into()), Animation::new(MENU_OUT), move |d, t| d.top(rest).opacity(1. - t));
            return div().absolute().inset_0().child(card).into_any_element();
        }
        let card = card.with_animation(ElementId::Name(format!("language-in-{serial}").into()), Animation::new(MENU_IN).with_easing(ease_out_quint()), move |d, t| d.top(rest - px(6.) * (1. - t)).opacity(t));
        let shut = |this: &mut Self, _: &MouseDownEvent, _: &mut Window, cx: &mut Context<Self>| this.close_pick(cx);
        div().id("language-sheet").absolute().inset_0().occlude().on_mouse_down(MouseButton::Left, cx.listener(shut)).on_mouse_down(MouseButton::Right, cx.listener(shut)).child(card).into_any_element()
    }

    /// A part of the window it can be dragged by, as a title bar is: the
    /// window's top and the rail. A double click does what the system's
    /// title bar does on one.
    fn drag_region(el: Div, cx: &mut Context<Self>) -> Div {
        el.on_mouse_down(
            MouseButton::Left,
            cx.listener(|this, ev: &MouseDownEvent, window, _| {
                this.win_move = ev.click_count != 2;
                if ev.click_count == 2 {
                    window.titlebar_double_click();
                }
            }),
        )
        .on_mouse_move(cx.listener(|this, ev: &MouseMoveEvent, window, _| {
            if !std::mem::take(&mut this.win_move) {
                return;
            }
            if ev.pressed_button == Some(MouseButton::Left) {
                window.start_window_move();
            }
        }))
    }
}

/// A group of the settings: its name over it, small and quiet, in line
/// with the rows' names, and its rows on one card.
fn group(title: Option<&'static str>, rows: Vec<AnyElement>, theme: &gpui_component::Theme) -> AnyElement {
    v_flex()
        .gap(px(6.))
        .when_some(title, |d, t| d.child(div().min_h(px(20.)).px(px(14.)).flex().items_center().text_size(px(12.)).font_weight(FontWeight::SEMIBOLD).text_color(theme.muted_foreground).child(t)))
        .child(
            v_flex()
                .w_full()
                .rounded(CARD_R)
                .border_1()
                .border_color(theme.border)
                .bg(card_ground(theme))
                .children(rows.into_iter().enumerate().map(|(i, row)| div().when(i > 0, |d| d.border_t_1().border_color(theme.border)).child(row))),
        )
        .into_any_element()
}

/// A card's ground: a shade off the page.
fn card_ground(theme: &gpui_component::Theme) -> Hsla {
    if theme.mode.is_dark() {
        theme.sidebar
    } else {
        theme.muted.opacity(0.4)
    }
}

/// A row of a card: its name, and under it a line where the name does not
/// say it all; its control at the right.
fn line(title: &'static str, detail: Option<AnyElement>, control: AnyElement, theme: &gpui_component::Theme) -> AnyElement {
    h_flex()
        .min_h(px(46.))
        .px(px(14.))
        .py(px(8.))
        .gap(px(16.))
        .items_center()
        .child(v_flex().flex_1().min_w_0().gap(px(1.)).child(div().text_size(px(13.)).child(title)).children(detail.map(|d| div().text_size(px(11.5)).line_height(px(15.)).text_color(theme.muted_foreground).child(d))))
        .child(div().flex_shrink_0().child(control))
        .into_any_element()
}

/// A sentence with things that are not words in it (links), in the
/// language in use: `{name}` in the translated sentence is where each
/// goes, so each language puts them where its order wants them.
fn weave(sentence: &str, mut things: Vec<(&str, AnyElement)>, row: Div) -> Div {
    let mut row = row;
    let mut rest = sentence;
    while let Some(open) = rest.find('{') {
        let Some(close) = rest[open..].find('}').map(|c| open + c) else { break };
        if open > 0 {
            row = row.child(rest[..open].to_string());
        }
        let name = &rest[open + 1..close];
        if let Some(i) = things.iter().position(|(n, _)| *n == name) {
            row = row.child(things.remove(i).1);
        }
        rest = &rest[close + 1..];
    }
    if !rest.is_empty() {
        row = row.child(rest.to_string());
    }
    row
}

/// The face the app's name is set in on About: a light, elegant sans with
/// flared strokes, Optima, else Avenir Next, both the Mac's own.
fn wordmark_face(cx: &App) -> SharedString {
    let names = cx.text_system().all_font_names();
    ["Optima", "Avenir Next"].into_iter().find(|f| names.iter().any(|n| n == f)).unwrap_or(".SystemUIFont").into()
}
