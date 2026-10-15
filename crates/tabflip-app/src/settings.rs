//! Settings: the shortcut, Accessibility, and opening at login. Opened
//! from the menu bar item, and by itself while Accessibility is missing.

use gpui::prelude::FluentBuilder as _;
use gpui::*;
use gpui_component::button::{Button, ButtonVariants as _};
use gpui_component::switch::Switch;
use gpui_component::{h_flex, v_flex, ActiveTheme as _, Disableable as _, Root, Sizable as _};
use tabflip_core::shortcut::Shortcut;

use crate::controller::{self, Controller};
use crate::sys;

pub struct Settings {
    controller: Entity<Controller>,
    login: bool,
    login_error: Option<SharedString>,
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
    let bounds = Bounds::centered(None, size(px(460.), px(300.)), cx);
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
            let view = cx.new(|cx| {
                cx.observe(&controller, |_, _, cx| cx.notify()).detach();
                Settings { controller, login: sys::launches_at_login(), login_error: None }
            });
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
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
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
