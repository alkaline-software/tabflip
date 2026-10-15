//! The list: a small stack of rows, each an app's icon, a word or two for
//! the window and the app's name, dim. It is a window of its own that is
//! kept for the app's life, put away between rounds, and never takes the
//! keyboard: the tap brings the keys.
//!
//! It fades in where it is and slides its highlight from row to row. On
//! the way out it fades over the window just picked, and is put away once
//! a fully clear frame has been drawn, so the next round never shows the
//! last one's rows for a frame.

use std::time::{Duration, Instant};

use gpui::prelude::FluentBuilder as _;
use gpui::*;

use crate::controller::{self, Controller};

const ROW_H: f32 = 30.;
const WIDTH: f32 = 300.;
const PAD: f32 = 6.;
/// Room around the stack for its shadow.
const MARGIN: f32 = 24.;
const ENTER: Duration = Duration::from_millis(110);
const LEAVE: Duration = Duration::from_millis(110);
const SLIDE: Duration = Duration::from_millis(90);
const HOVER_AFTER: Duration = Duration::from_millis(250);

pub struct Panel {
    controller: Entity<Controller>,
    shown: Instant,
    leaving: Option<Instant>,
    /// Which round of showing this is, so a put-away from an earlier one
    /// does not hide a later one.
    showing: u64,
    /// Where the highlight slides from, and since when.
    slide_from: f32,
    slide_since: Instant,
    last_selected: Option<usize>,
    scroll: ScrollHandle,
}

pub fn open(controller: Entity<Controller>, cx: &mut App) -> Option<WindowHandle<Panel>> {
    let bounds = Bounds::new(point(px(0.), px(0.)), size(px(WIDTH + 2. * MARGIN), px(ROW_H + 2. * (PAD + MARGIN))));
    cx.open_window(
        WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            titlebar: None,
            focus: false,
            show: false,
            kind: WindowKind::PopUp,
            is_movable: false,
            is_resizable: false,
            is_minimizable: false,
            window_background: WindowBackgroundAppearance::Transparent,
            ..Default::default()
        },
        |_, cx| {
            let now = Instant::now() - LEAVE * 2;
            cx.new(|cx| {
                cx.observe(&controller, |_, _, cx| cx.notify()).detach();
                Panel { controller, shown: now, leaving: Some(now), showing: 0, slide_from: 0., slide_since: now, last_selected: None, scroll: ScrollHandle::new() }
            })
        },
    )
    .ok()
}

/// Fast at first and slow to rest.
fn ease(t: f32) -> f32 {
    1. - (1. - t).powi(3)
}

fn progress(since: Instant, over: Duration) -> f32 {
    (since.elapsed().as_secs_f32() / over.as_secs_f32()).min(1.)
}

impl Panel {
    /// Shown for a round with `rows` rows, sized to them.
    pub fn appear(&mut self, rows: usize, window: &mut Window, cx: &mut Context<Self>) {
        let most = ((crate::sys::screen_height() as f32 * 0.7 - 2. * PAD) / ROW_H).floor().max(1.) as usize;
        let height = rows.clamp(1, most) as f32 * ROW_H + 2. * (PAD + MARGIN);
        self.showing += 1;
        self.shown = Instant::now();
        self.leaving = None;
        self.last_selected = None;
        crate::sys::show_panel(window, (WIDTH + 2. * MARGIN) as f64, height as f64);
        // gpui hears of a resize through an update of this window, which
        // is the one being updated now, so it would never hear of it: the
        // list would be drawn at the size it was opened with, one row.
        window.bounds_changed(cx);
        crate::sys::log(format_args!("panel shown, {rows} rows, {height} high"));
        cx.notify();
    }

    pub fn leave(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.leaving.is_some() {
            return;
        }
        self.leaving = Some(Instant::now());
        cx.notify();
        let showing = self.showing;
        cx.spawn_in(window, async move |this, cx| {
            cx.background_executor().timer(LEAVE + Duration::from_millis(50)).await;
            let _ = this.update_in(cx, |p, window, _| {
                if p.showing == showing && p.leaving.is_some() {
                    crate::sys::hide_panel(window);
                }
            });
        })
        .detach();
    }
}

struct Colors {
    ground: Hsla,
    edge: Hsla,
    text: Hsla,
    dim: Hsla,
    pick: Hsla,
    pick_text: Hsla,
}

fn colors(window: &Window) -> Colors {
    let dark = matches!(window.appearance(), WindowAppearance::Dark | WindowAppearance::VibrantDark);
    if dark {
        Colors { ground: rgba(0x1e1e20f2).into(), edge: rgba(0xffffff1f).into(), text: rgba(0xf5f5f7ff).into(), dim: rgba(0xf5f5f780).into(), pick: rgb(0x0a84ff).into(), pick_text: gpui::white() }
    } else {
        Colors { ground: rgba(0xf6f6f6f5).into(), edge: rgba(0x0000001a).into(), text: rgba(0x1d1d1fff).into(), dim: rgba(0x1d1d1f80).into(), pick: rgb(0x007aff).into(), pick_text: gpui::white() }
    }
}

impl Render for Panel {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let c = self.controller.read(cx);
        let selected = c.selected();
        let rows = c.rows.clone();

        if selected != self.last_selected {
            if let Some(s) = selected {
                // From where the highlight is drawn now, so a slide cut
                // short by the next key goes on from there.
                self.slide_from = match self.last_selected {
                    Some(_) => self.highlight_y(),
                    None => s as f32 * ROW_H,
                };
                self.slide_since = Instant::now();
                self.scroll.scroll_to_item(s);
            }
            self.last_selected = selected;
        }

        let opacity = match self.leaving {
            Some(at) => 1. - ease(progress(at, LEAVE)),
            None => ease(progress(self.shown, ENTER)),
        };
        let moving = self.leaving.map_or(self.shown.elapsed() < ENTER, |at| at.elapsed() < LEAVE + Duration::from_millis(20)) || self.slide_since.elapsed() < SLIDE;
        if moving {
            window.request_animation_frame();
        }
        let colors = colors(window);
        // A few points of drift as it fades, down into place.
        let drift = 6. * (1. - opacity);
        let highlight_y = self.highlight_y();
        let controller = self.controller.clone();

        let list = div()
            .id("rows")
            .relative()
            .size_full()
            .overflow_y_scroll()
            .track_scroll(&self.scroll)
            .when(selected.is_some(), |d| {
                d.child(div().absolute().left_0().right_0().top(px(highlight_y)).h(px(ROW_H)).rounded(px(7.)).bg(colors.pick))
            })
            .children(rows.into_iter().enumerate().map(|(i, row)| {
                let on = selected == Some(i);
                let (hover, click) = (controller.clone(), controller.clone());
                let shown = self.shown;
                div()
                    .id(i)
                    .relative()
                    .h(px(ROW_H))
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .px(px(9.))
                    // The panel can open under a pointer at rest, and the
                    // system says the pointer moved as it does; only a move
                    // made after it is in counts.
                    .on_mouse_move(move |_, _, cx| {
                        if shown.elapsed() > HOVER_AFTER {
                            hover.update(cx, |c, cx| c.hover(i, cx));
                        }
                    })
                    .on_click(move |_, _, cx| click.update(cx, |c, cx| c.click(i, cx)))
                    .child(match row.icon {
                        Some(icon) => img(icon).size(px(16.)).flex_shrink_0().into_any_element(),
                        None => div().size(px(16.)).flex_shrink_0().into_any_element(),
                    })
                    .child(div().flex_shrink_1().min_w_0().overflow_hidden().whitespace_nowrap().text_ellipsis().text_size(px(13.)).text_color(if on { colors.pick_text } else { colors.text }).child(row.label))
                    .child(div().ml_auto().flex_shrink_0().max_w(px(110.)).overflow_hidden().whitespace_nowrap().text_ellipsis().text_size(px(11.)).text_color(if on { colors.pick_text.opacity(0.75) } else { colors.dim }).child(row.app))
            }));

        div().size_full().p(px(MARGIN)).font_family(".SystemUIFont").child(
            div()
                .size_full()
                .opacity(opacity)
                .p(px(PAD))
                .rounded(px(12.))
                .bg(colors.ground)
                .border_1()
                .border_color(colors.edge)
                .shadow_lg()
                .overflow_hidden()
                .mt(px(-drift))
                .child(list),
        )
    }
}

impl Panel {
    fn highlight_y(&self) -> f32 {
        let to = self.last_selected.unwrap_or(0) as f32 * ROW_H;
        let t = ease(progress(self.slide_since, SLIDE));
        self.slide_from + (to - self.slide_from) * t
    }
}

/// The panel's window, opened once.
pub fn install(cx: &mut App) {
    let controller = controller::get(cx);
    if let Some(handle) = open(controller.clone(), cx) {
        controller.update(cx, |c, _| c.set_panel(handle));
    }
}
