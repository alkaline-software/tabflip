//! Settings, tabflip's only window, drawn as Alkaline Software's other
//! apps draw theirs: a rail of sections at the left, under the window's
//! traffic lights, and the chosen section's groups of rows on cards at
//! the right. General is the shortcut, how the app stands on this Mac
//! and Accessibility; Customize is how the list looks; Updates looks for
//! a newer release; About says who makes it.
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
use crate::sys;
use crate::ui::{key_caps, page_in, pill, swallow_click, Press, TURN};

/// The settings' sections, in the rail's order: key, name, icon.
pub const SECTIONS: &[(&str, &str, &str)] = &[
    ("general", "General", "icons/settings.svg"),
    ("customize", "Customize", "icons/sparkles.svg"),
    ("updates", "Updates", "icons/redo-2.svg"),
    ("about", "About", "icons/info.svg"),
];

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
const CARD_R: Pixels = px(12.);

pub struct Settings {
    controller: Entity<Controller>,
    login: bool,
    login_error: Option<SharedString>,
    /// A highlight colour of one's own, typed as hex.
    hex: Entity<InputState>,
    hex_error: bool,
    scroll: ScrollHandle,
    /// A press on the window's top or the rail that may become a drag.
    win_move: bool,
    /// Each switch: whether it is on, when it last turned, how many times.
    turns: RefCell<HashMap<&'static str, (bool, Instant, u32)>>,
    /// Each choice: the one it came from, the one chosen, when.
    seg_state: RefCell<HashMap<&'static str, (&'static str, &'static str, Option<Instant>)>>,
    /// Where each choice's segment was drawn last, for the plate to slide.
    seg_bounds: Rc<RefCell<HashMap<String, Bounds<Pixels>>>>,
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
            login: sys::launches_at_login(),
            login_error: None,
            hex,
            hex_error: false,
            scroll: ScrollHandle::new(),
            win_move: false,
            turns: RefCell::default(),
            seg_state: RefCell::default(),
            seg_bounds: Rc::default(),
            _subscriptions: vec![observe, changes, back],
        }
    }
}

struct Open(WindowHandle<Root>);

impl Global for Open {}

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
    if let Some(Open(handle)) = cx.try_global::<Open>() {
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
            cx.new(|cx| Root::new(view, window, cx))
        },
    );
    match opened {
        Ok(handle) => {
            cx.set_global(Open(handle));
            cx.activate(true);
        }
        Err(e) => {
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
        let ours = cx.try_global::<Open>().is_some_and(|Open(h)| h.window_id() == id);
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
    if let Some(Open(handle)) = cx.try_global::<Open>() {
        let handle = *handle;
        cx.defer(move |cx| {
            let _ = handle.update(cx, |_, window, _| window.remove_window());
        });
    }
}

/// `TABFLIP_SHOT_SETTINGS`: a picture of the Settings window.
pub fn shoot(path: &std::path::Path, cx: &mut App) {
    if let Some(Open(handle)) = cx.try_global::<Open>() {
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
        let section = self.controller.read(cx).section;
        let rows = match section {
            "customize" => self.customize(window, cx),
            "updates" => self.updates(cx),
            "about" => self.about(cx),
            _ => self.general(cx),
        };
        let title = SECTIONS.iter().find(|(k, _, _)| *k == section).map(|(_, name, _)| *name).unwrap_or("General");
        h_flex()
            .size_full()
            .items_stretch()
            .bg(theme.popover)
            .text_color(theme.foreground)
            .child(self.rail(section, cx))
            .child(
                v_flex()
                    .flex_1()
                    .min_w_0()
                    .min_h_0()
                    // The window's top is dragged by, and holds the
                    // section's name, level with the traffic lights, as the
                    // Mac's own settings do. It fades in with the page.
                    .child(Self::drag_region(h_flex().h(TOP_H).flex_shrink_0().w_full().px(PAGE_X).items_center(), cx).child(page_in(format!("title-{section}"), div().text_size(px(15.)).font_weight(FontWeight::SEMIBOLD).child(title))))
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
            )
    }
}

impl Settings {
    /// The rail of sections at the window's left, under its traffic
    /// lights: an icon and a name per section, the chosen one on a plate.
    /// General wears a dot while Accessibility is missing, and Updates
    /// while a newer tabflip waits.
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
                    this.controller.update(cx, |c, cx| {
                        c.section = key;
                        cx.notify();
                    });
                    this.scroll.set_offset(point(px(0.), px(0.)));
                    cx.notify();
                }))
                .child(Icon::default().path(icon).with_size(px(15.)).text_color(if active { theme.foreground } else { theme.muted_foreground }))
                .child(div().flex_1().text_size(px(13.)).when(active, |d| d.font_weight(FontWeight::MEDIUM)).text_color(if active { theme.foreground } else { theme.sidebar_foreground }).child(label))
                .when(dot, |d| d.child(div().size(px(7.)).rounded_full().bg(theme.primary)))
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
        let shown = c.config.menu_bar_icon;
        let (note, wrong): (SharedString, bool) = if let Some(e) = &c.record_error {
            (e.to_string().into(), true)
        } else if recording {
            ("Press the keys, or Esc to keep the old one.".into(), false)
        } else if !listening {
            ("Allow Accessibility below first.".into(), false)
        } else if c.system_switcher_stuck {
            ("macOS kept its own ⌘Tab; try ⌥Tab.".into(), true)
        } else if shortcut.is_command_tab() {
            ("Replaces the Mac's own ⌘Tab while tabflip runs.".into(), false)
        } else {
            ("Hold it, press again to move down, let go to switch.".into(), false)
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
            .map(|d| if recording { d.text_color(theme.muted_foreground).child("Press keys…") } else { d.child(key_caps(&shortcut.to_string())) })
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
        let reset = pill("reset", "icons/rotate.svg", "Reset", Press::Quiet, unchanged, &theme).flex_1().when(!unchanged, |d| {
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

        let access = if listening {
            h_flex().h(px(28.)).w_full().gap(px(6.)).items_center().justify_center().rounded(px(8.)).bg(theme.success.opacity(0.1)).text_size(px(12.5)).text_color(theme.success).child(Icon::default().path("icons/check.svg").with_size(px(14.))).child("Allowed").into_any_element()
        } else {
            pill("allow", "icons/external-link.svg", "Open Settings", Press::Primary, false, &theme)
                .w_full()
                .on_click(|_, window, cx| {
                    swallow_click(window, cx);
                    sys::open_url(sys::ACCESSIBILITY_SETTINGS_URL);
                })
                .into_any_element()
        };
        let access_note: &'static str = if listening { "To see every window and switch to it." } else { "Turn tabflip on in the list; it starts by itself." };

        vec![
            group(Some("Switcher"), vec![line("Shortcut", Some(shortcut_note), shortcut_control, &theme)], &theme),
            group(
                Some("App"),
                vec![
                    line("Launch at login", login_note, login, &theme),
                    line("Show in menu bar", Some(div().child("When hidden, open tabflip again to come back here.").into_any_element()), menu_bar, &theme),
                ],
                &theme,
            ),
            group(Some("Permissions"), vec![line("Accessibility", Some(div().child(access_note).into_any_element()), div().w(PERMIT_W).child(access).into_any_element(), &theme)], &theme),
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
                Size::ALL.iter().map(|s| (s.name(), s.name())).collect(),
                size_now.name(),
                Rc::new(|this, key, cx| {
                    let size = Size::from_config(key);
                    this.controller.update(cx, |c, cx| c.set_size(size, cx));
                }),
                cx,
            ))
            .into_any_element();

        // The swatches in the appearance the list will be drawn in.
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
                    .tooltip(move |window, cx| Tooltip::new(name).build(window, cx))
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
        let hex_note = if self.hex_error { Some(div().text_color(theme.danger).child("Not a colour: #rrggbb or #rgb.").into_any_element()) } else { Some(div().child("Any colour, as hex.").into_any_element()) };

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
            Some("The list"),
            vec![
                line("Size", None, sizes, &theme),
                line("Highlight", None, swatches, &theme),
                line("Your own colour", hex_note, hex, &theme),
                line("Preview", None, preview, &theme),
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
            "a newer version is out."
        } else if failed {
            "could not reach GitHub."
        } else if answered {
            "up to date."
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
        let version_note = v_flex().gap(px(1.)).child(div().child(version_line)).when_some(available.clone(), |d, v| d.child(link("update-notes", "Release notes", update::release_page(&v)))).into_any_element();
        let button = match available.clone() {
            Some(v) => pill("update-get", "icons/download.svg", format!("Download {v}"), Press::Primary, false, &theme).w(UPDATE_W).on_click(move |_, window, cx| {
                swallow_click(window, cx);
                sys::open_url(&update::asset_url(&v));
            }),
            None => pill("update-check", "icons/redo-2.svg", if checking { "Checking…" } else { "Check for updates" }, Press::Quiet, checking, &theme).w(UPDATE_W).when(!checking, |d| {
                d.on_click(cx.listener(|this, _, window, cx| {
                    swallow_click(window, cx);
                    this.controller.update(cx, |c, cx| c.check_updates(false, cx));
                }))
            }),
        }
        .into_any_element();
        let auto = self.switch("update-auto", auto_on, Rc::new(|this, on, cx| this.controller.update(cx, |c, cx| c.set_check_updates(on, cx))), cx);
        let downloaded = available.is_some().then(|| div().px(px(14.)).text_size(px(11.5)).text_color(theme.muted_foreground).child("Open the downloaded disk image and drag tabflip to Applications, over the old one.").into_any_element());
        let mut out = vec![group(
            None,
            vec![
                line("Version", Some(version_note), button, &theme),
                line("Check automatically", Some(div().child("Once a day. Nothing is installed without you.").into_any_element()), auto, &theme),
            ],
            &theme,
        )];
        out.extend(downloaded);
        out
    }

    /// About: a title card, and who makes tabflip: Alkaline Software.
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
        let prose = || h_flex().flex_wrap().justify_center().gap(px(4.)).text_size(px(13.5)).line_height(px(22.)).text_color(theme.muted_foreground);
        let go = |id: &'static str, icon: &'static str, label: &'static str, url: String| pill(id, icon, label, Press::Quiet, false, &theme).h(px(24.)).px(px(10.)).text_size(px(12.)).on_click(move |_, _, _| sys::open_url(&url));
        let site = update::repo_page();
        let open_site = {
            let site = site.clone();
            move |_: &ClickEvent, _: &mut Window, _: &mut App| sys::open_url(&site)
        };
        vec![
            // A title card, set in the middle: the app and its version.
            v_flex()
                .w_full()
                .items_center()
                .pt(px(30.))
                .pb(px(28.))
                .rounded(CARD_R)
                .border_1()
                .border_color(theme.border)
                .bg(card_ground(&theme))
                .child(div().id("about-icon").cursor_pointer().hover(|s| s.opacity(0.82)).on_click(open_site.clone()).child(img("icon/app-large.png").size(px(88.)).flex_shrink_0()))
                .child(div().id("about-name").mt(px(6.)).cursor_pointer().hover(|s| s.opacity(0.82)).on_click(open_site).text_size(px(42.)).line_height(px(48.)).text_color(theme.primary).font_family(wordmark_face(cx)).child("tabflip"))
                .child(div().mt(px(4.)).text_size(px(12.5)).text_color(theme.muted_foreground).child(format!("Switch between windows, not apps · Version {current}")))
                .into_any_element(),
            v_flex()
                .w_full()
                .items_center()
                .gap(px(16.))
                .pt(px(8.))
                .child(prose().child("An open source software package by").child(link("about-studio", "Alkaline Software", "https://github.com/alkaline-software")))
                .child(h_flex().gap(px(8.)).child(go("about-source", "icons/github.svg", "GitHub", site.clone())).child(go("about-notes", "icons/file-text.svg", "Release Notes", update::release_page(current))))
                .into_any_element(),
        ]
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
        let moving = {
            let mut turns = self.turns.borrow_mut();
            let now = Instant::now();
            let t = turns.entry(id).or_insert((on, now - TURN * 2, 0));
            if t.0 != on {
                *t = (on, now, t.2 + 1);
            }
            (t.2 > 0 && t.1.elapsed() < TURN).then_some(t.2)
        };
        let at = move |t: f32| px(INSET + travel * t);
        let fill = div().absolute().inset_0().rounded_full().bg(theme.primary);
        let knob = div().absolute().top(px(INSET)).size(px(KNOB)).rounded_full().bg(gpui::white()).shadow_sm();
        let (fill, knob) = match moving {
            Some(n) => (
                fill.with_animation(ElementId::Name(format!("switch-{id}-fill-{n}").into()), Animation::new(TURN).with_easing(ease_out_quint()), move |d, t| d.opacity(if on { t } else { 1. - t })).into_any_element(),
                knob.with_animation(ElementId::Name(format!("switch-{id}-knob-{n}").into()), Animation::new(TURN).with_easing(ease_out_quint()), move |d, t| d.left(at(if on { t } else { 1. - t }))).into_any_element(),
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

/// The face the app's name is set in on About: a light, elegant sans with
/// flared strokes, Optima, else Avenir Next, both the Mac's own.
fn wordmark_face(cx: &App) -> SharedString {
    let names = cx.text_system().all_font_names();
    ["Optima", "Avenir Next"].into_iter().find(|f| names.iter().any(|n| n == f)).unwrap_or(".SystemUIFont").into()
}
