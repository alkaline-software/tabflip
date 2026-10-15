//! Settings: the shortcut, the list's size and highlight colour,
//! Accessibility, and opening at login. Opened from the menu bar item, and
//! by itself while Accessibility is missing.

use gpui::prelude::FluentBuilder as _;
use gpui::*;
use gpui_component::button::{Button, ButtonVariants as _};
use gpui_component::input::{Input, InputEvent, InputState};
use gpui_component::switch::Switch;
use gpui_component::{h_flex, v_flex, ActiveTheme as _, Disableable as _, Root, Sizable as _};
use tabflip_core::look::{self, Size};
use tabflip_core::shortcut::Shortcut;

use crate::controller::{self, Controller};
use crate::sys;

pub struct Settings {
    controller: Entity<Controller>,
    login: bool,
    login_error: Option<SharedString>,
    /// A highlight colour of one's own, typed as hex.
    hex: Entity<InputState>,
    hex_error: bool,
    _hex_changes: Subscription,
}

impl Settings {
    fn new(controller: Entity<Controller>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        cx.observe(&controller, |_, _, cx| cx.notify()).detach();
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
            // marked once the field is left with it.
            let ok = look::parse_hex(&text).is_some() && this.controller.update(cx, |c, cx| c.set_highlight(&text, cx));
            this.hex_error = !ok && !text.trim().is_empty() && text.trim().trim_start_matches('#').len() >= 6;
            cx.notify();
        });
        Settings { controller, login: sys::launches_at_login(), login_error: None, hex, hex_error: false, _hex_changes: changes }
    }
}

struct Open(WindowHandle<Root>);

impl Global for Open {}

/// The Settings window brought forward, opened if it is not.
pub fn show(cx: &mut App) {
    if let Some(Open(handle)) = cx.try_global::<Open>() {
        let handle = *handle;
        if handle.update(cx, |_, window, _| window.activate_window()).is_ok() {
            cx.activate(true);
            return;
        }
    }
    let controller = controller::get(cx);
    let bounds = Bounds::centered(None, size(px(460.), px(530.)), cx);
    let opened = cx.open_window(
        WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            titlebar: Some(TitlebarOptions { title: Some("tabflip".into()), appears_transparent: false, traffic_light_position: None }),
            is_resizable: false,
            is_minimizable: false,
            focus: true,
            ..Default::default()
        },
        |window, cx| {
            gpui_component::Theme::sync_system_appearance(Some(window), cx);
            window.observe_window_appearance(|window, cx| gpui_component::Theme::sync_system_appearance(Some(window), cx)).detach();
            let view = cx.new(|cx| Settings::new(controller, window, cx));
            // Closing Settings while it records gives up the recording.
            let c = view.read(cx).controller.clone();
            window.on_window_should_close(cx, move |_, cx| {
                c.update(cx, |c, cx| c.stop_recording(cx));
                true
            });
            cx.new(|cx| Root::new(view, window, cx))
        },
    );
    if let Ok(handle) = opened {
        cx.set_global(Open(handle));
        cx.activate(true);
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

fn row(label: &'static str, cx: &App) -> Div {
    h_flex().w_full().min_h(px(36.)).gap_3().child(div().w(px(130.)).flex_shrink_0().text_color(cx.theme().muted_foreground).child(label))
}

impl Render for Settings {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let c = self.controller.read(cx);
        let (listening, recording) = (c.listening, c.recording);
        let shortcut = c.config.shortcut();
        let note: Option<SharedString> = if let Some(e) = &c.record_error {
            Some(e.to_string().into())
        } else if recording {
            Some("Press the keys, or Esc to keep the old one".into())
        } else if c.system_switcher_stuck {
            Some("macOS kept its own ⌘Tab; try ⌥Tab".into())
        } else if shortcut.is_command_tab() {
            Some("Replaces the Mac's own ⌘Tab while tabflip runs".into())
        } else {
            None
        };
        let controller = self.controller.clone();

        let shortcut_button = {
            let controller = controller.clone();
            Button::new("shortcut")
                .label(if recording { SharedString::from("Press shortcut…") } else { shortcut.to_string().into() })
                .when(recording, |b| b.primary())
                .when(!recording, |b| b.outline())
                .disabled(!listening)
                .on_click(move |_, _, cx| {
                    controller.update(cx, |c, cx| if c.recording { c.stop_recording(cx) } else { c.start_recording(cx) });
                })
        };
        let reset = {
            let controller = controller.clone();
            Button::new("reset").label("Reset").ghost().small().when(shortcut == Shortcut::default() || recording, |b| b.invisible()).on_click(move |_, _, cx| {
                controller.update(cx, |c, cx| {
                    c.set_shortcut(Shortcut::default());
                    cx.notify();
                })
            })
        };

        let access = if listening {
            h_flex().gap_2().child(div().text_color(cx.theme().success).child("✓")).child("Allowed").into_any_element()
        } else {
            Button::new("grant").label("Open System Settings").primary().small().on_click(|_, _, _| sys::open_url(sys::ACCESSIBILITY_SETTINGS_URL)).into_any_element()
        };

        let size_now = c.config.size();
        let sizes = h_flex().gap_1().children(Size::ALL.into_iter().map(|size| {
            let controller = controller.clone();
            Button::new(size.name())
                .label(size.name())
                .small()
                .when(size == size_now, |b| b.primary())
                .when(size != size_now, |b| b.outline())
                .on_click(move |_, _, cx| controller.update(cx, |c, cx| c.set_size(size, cx)))
        }));

        // The swatches in the appearance the list will be drawn in.
        let dark = matches!(window.appearance(), WindowAppearance::Dark | WindowAppearance::VibrantDark);
        let highlight = c.config.highlight().to_string();
        let swatches = h_flex().gap(px(6.)).children(look::PRESETS.iter().map(|p| {
            let on = p.name.eq_ignore_ascii_case(&highlight);
            let (controller, hex) = (controller.clone(), self.hex.clone());
            let name = p.name;
            div()
                .id(name)
                .size(px(20.))
                .rounded_full()
                .bg(rgb(if dark { p.dark } else { p.light }))
                .border_2()
                .border_color(if on { cx.theme().foreground } else { gpui::transparent_black() })
                .cursor_pointer()
                .tooltip(move |window, cx| gpui_component::tooltip::Tooltip::new(name).build(window, cx))
                .on_click(move |_, window, cx| {
                    controller.update(cx, |c, cx| c.set_highlight(name, cx));
                    hex.update(cx, |h, cx| h.set_value("", window, cx));
                })
        }));

        // A row as the list draws it, at the chosen size and colour.
        let preview = {
            let k = size_now.scale();
            let pick = look::highlight(&highlight, dark);
            let text = if look::dark_text_on(pick) { rgb(0x1d1d1f).into() } else { gpui::white() };
            h_flex()
                .w(px(220. * k.min(1.25)))
                .h(px(30. * k))
                .px(px(9. * k))
                .gap(px(8. * k))
                .rounded(px(7. * k))
                .bg(rgb(pick))
                .text_color(text)
                .child(div().size(px(16. * k)).rounded(px(4. * k)).bg(text.opacity(0.35)))
                .child(div().text_size(px(13. * k)).child("Gmail"))
                .child(div().ml_auto().text_size(px(11. * k)).opacity(0.75).child("Google Chrome"))
        };

        let login = Switch::new("login").checked(self.login).on_click(cx.listener(|this, on: &bool, _, cx| {
            match sys::set_launch_at_login(*on) {
                Ok(()) => this.login_error = None,
                Err(e) => this.login_error = Some(e.into()),
            }
            this.login = sys::launches_at_login();
            cx.notify();
        }));

        v_flex()
            .size_full()
            .p_5()
            .gap_1()
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .text_size(px(13.))
            .child(div().text_size(px(15.)).font_weight(FontWeight::SEMIBOLD).child("Switch between windows, not apps"))
            .child(div().pb_3().text_color(cx.theme().muted_foreground).child("Hold the shortcut, press it again to move down the list, let go to switch."))
            .child(row("Shortcut", cx).child(shortcut_button).child(reset))
            .child(div().pl(px(142.)).h(px(18.)).text_size(px(11.)).text_color(if c.record_error.is_some() { cx.theme().danger } else { cx.theme().muted_foreground }).children(note))
            .child(row("List size", cx).child(sizes))
            .child(row("Highlight", cx).child(swatches))
            .child(row("", cx).child(div().w(px(110.)).child(Input::new(&self.hex).small())).when(self.hex_error, |r| r.child(div().text_size(px(11.)).text_color(cx.theme().danger).child("Not a colour"))))
            .child(div().pl(px(142.)).py_2().child(preview))
            .child(row("Accessibility", cx).child(access))
            .when(!listening, |d| d.child(div().pl(px(142.)).text_size(px(11.)).text_color(cx.theme().muted_foreground).child("Turn tabflip on in the list, and it starts by itself.")))
            .child(row("Open at login", cx).child(login))
            .children(self.login_error.clone().map(|e| div().pl(px(142.)).text_size(px(11.)).text_color(cx.theme().danger).child(e)))
            .child(
                h_flex()
                    .mt_auto()
                    .w_full()
                    .justify_between()
                    .text_size(px(11.))
                    .text_color(cx.theme().muted_foreground)
                    .child(format!("tabflip {}", env!("CARGO_PKG_VERSION")))
                    .child(Button::new("quit").label("Quit tabflip").ghost().small().on_click(|_, _, cx| cx.quit())),
            )
    }
}
