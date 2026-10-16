//! The Settings window's small parts, the same as Alkaline Software's
//! other apps draw them: the button (a pill with an icon), a page coming
//! in, a key on its cap, and the click that is swallowed without leaving
//! a text drag open.

use std::time::Duration;

use gpui::prelude::FluentBuilder as _;
use gpui::*;
use gpui_component::{h_flex, Icon, Sizable as _};

/// A switch turning, and a choice's plate sliding.
pub const TURN: Duration = Duration::from_millis(200);

/// Swallow a click without leaving the toolkit's text selection mid-drag.
/// gpui-component's selection layer begins a drag on every left mouse-down
/// and ends it on the bubble-phase mouse-up; a click handler that only
/// stops propagation eats that mouse-up, and every later mouse move then
/// extends a selection nobody is making.
pub fn swallow_click(window: &mut Window, cx: &mut App) {
    gpui_base::TextSelection::end(window, cx);
    cx.stop_propagation();
}

/// A page of the settings coming in: it fades in, rising a little.
pub fn page_in(id: impl Into<SharedString>, el: Div) -> AnyElement {
    el.with_animation(ElementId::Name(id.into()), Animation::new(Duration::from_millis(240)).with_easing(ease_out_quint()), |d, t| d.opacity(t).pt(px(8. * (1. - t)))).into_any_element()
}

/// What a button is, which is how its pill is drawn.
#[derive(Clone, Copy, PartialEq)]
pub enum Press {
    /// One of several, or the way out: an outline.
    Quiet,
    /// The answer the page is waiting on: filled with the accent.
    Primary,
}

/// The window's button: a pill with an icon before its word. `off` is one
/// that cannot be pressed now: paler, and the caller gives it no click.
pub fn pill(id: impl Into<ElementId>, icon: &'static str, label: impl Into<SharedString>, kind: Press, off: bool, theme: &gpui_component::Theme) -> Stateful<Div> {
    let ink = match kind {
        Press::Primary => theme.primary_foreground,
        Press::Quiet => theme.foreground,
    };
    let (ground, edge, hover) = match kind {
        Press::Primary => (Some(theme.primary), theme.primary, theme.primary.opacity(0.85)),
        // An edge of the ink, not the border's colour: on a card the
        // border is the card's own shade and the pill had no outline.
        Press::Quiet => (None, theme.foreground.opacity(0.16), theme.foreground.opacity(0.06)),
    };
    h_flex()
        .id(id)
        .h(px(28.))
        .px(px(12.))
        .gap(px(6.))
        .flex_shrink_0()
        .items_center()
        .justify_center()
        .rounded_full()
        .text_size(px(12.5))
        .text_color(ink)
        .border_1()
        .border_color(edge)
        .when_some(ground, |d, bg| d.bg(bg))
        .when(!icon.is_empty(), |d| d.child(Icon::default().path(icon).with_size(px(14.)).text_color(ink)))
        .child(label.into())
        .map(|d| if off { d.opacity(0.4) } else { d.cursor_pointer().hover(move |s| s.bg(hover)) })
}

/// A key's cap is this tall, and no narrower.
const KEY_CAP: f32 = 19.;

/// One key on its cap: a small plate, every one the same height and no
/// narrower than that, with the key in its middle.
fn key_cap(key: &str) -> Div {
    div()
        .flex_shrink_0()
        .h(px(KEY_CAP))
        .min_w(px(KEY_CAP))
        .px(px(4.))
        .rounded(px(4.5))
        .bg(gpui::hsla(0., 0., 0.5, 0.17))
        .flex()
        .items_center()
        .justify_center()
        .text_size(px(11.5))
        .line_height(px(KEY_CAP))
        .font_weight(FontWeight::MEDIUM)
        .child(key.to_string())
}

/// A shortcut as `Shortcut`'s display writes it ("⌃⌥ Tab"), each key on
/// its own cap.
pub fn key_caps(shortcut: &str) -> Div {
    let (signs, key) = shortcut.split_once(' ').unwrap_or(("", shortcut));
    h_flex().flex_shrink_0().gap(px(3.)).children(signs.chars().map(|c| key_cap(&c.to_string()))).when(!key.is_empty(), |d| d.child(key_cap(key)))
}
