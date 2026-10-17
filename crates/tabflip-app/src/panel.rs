//! The list: a small stack of rows, each an app's icon, a word or two for
//! the window and the app's name, dim. It is a window of its own that is
//! kept for the app's life, put away between rounds, and never takes the
//! keyboard: the tap brings the keys.
//!
//! It fades in where it is and slides its highlight from row to row, each
//! an animation on gpui's clock. On the way out it fades over the window
//! just picked, and is put away once a fully clear frame has been drawn,
//! so the next round never shows the last one's rows for a frame.

use std::cell::Cell;
use std::rc::Rc;
use std::time::{Duration, Instant};

use gpui::prelude::FluentBuilder as _;
use gpui::*;

use crate::controller::{self, Controller};

// The measures at Small; Medium and Large scale every one of them
// (`look::Size::scale`). The margin, room for the shadow, does not scale.
const ROW_H: f32 = 30.;
const WIDTH: f32 = 300.;
const PAD: f32 = 6.;
/// Room around the stack for its shadow.
const MARGIN: f32 = 24.;
const ENTER: Duration = Duration::from_millis(110);
const LEAVE: Duration = Duration::from_millis(110);
const SLIDE: Duration = Duration::from_millis(150);
/// The stack's hairline edge, on each side. It is inside the stack's
/// box, so the rows' room is the stack less its padding and its edge: a
/// height that left it out made the rows 2 points taller than their
/// scroller, and each move to the first or last row scrolled the whole
/// list by that much.
const EDGE: f32 = 1.;
const HOVER_AFTER: Duration = Duration::from_millis(250);

pub struct Panel {
    controller: Entity<Controller>,
    shown: Instant,
    leaving: Option<Instant>,
    /// Which round of showing this is, so a put-away from an earlier one
    /// does not hide a later one.
    showing: u64,
    /// The highlight's slides, counted, so each plays once on gpui's
    /// clock; where the one under way started, and where the highlight
    /// is drawn now, which the next slide starts from.
    slide: u32,
    slide_from: f32,
    at: Rc<Cell<f32>>,
    last_selected: Option<usize>,
    scroll: ScrollHandle,
    /// The size the list was shown at: kept for the round, so a change in
    /// Settings takes effect from the next one.
    scale: f32,
}

pub fn open(controller: Entity<Controller>, cx: &mut App) -> Option<WindowHandle<Panel>> {
    let bounds = Bounds::new(point(px(0.), px(0.)), size(px(WIDTH + 2. * (MARGIN + EDGE)), px(ROW_H + 2. * (PAD + MARGIN + EDGE))));
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
            // gpui holds an inactive window's animations to 30 frames a
            // second to save energy. The list never activates (the app in
            // front keeps the keyboard) and moves only while it shows, so
            // it is let run at the display's rate: at 30 the slide was a
            // jump with one smeared frame.
            inactive_frame_interval: None,
            ..Default::default()
        },
        |_, cx| {
            let now = Instant::now() - LEAVE * 2;
            cx.new(|cx| {
                cx.observe(&controller, |_, _, cx| cx.notify()).detach();
                Panel { controller, shown: now, leaving: Some(now), showing: 0, slide: 0, slide_from: 0., at: Rc::default(), last_selected: None, scroll: ScrollHandle::new(), scale: 1. }
            })
        },
    )
    .ok()
}

impl Panel {
    /// Shown for a round with `rows` rows, sized to them.
    pub fn appear(&mut self, rows: usize, window: &mut Window, cx: &mut Context<Self>) {
        self.scale = self.controller.read(cx).config.size().scale();
        let (row_h, pad, width) = (ROW_H * self.scale, PAD * self.scale, WIDTH * self.scale);
        let most = ((crate::sys::screen_height() as f32 * 0.7 - 2. * pad) / row_h).floor().max(1.) as usize;
        let height = rows.clamp(1, most) as f32 * row_h + 2. * (pad + MARGIN + EDGE);
        self.showing += 1;
        self.shown = Instant::now();
        self.leaving = None;
        self.last_selected = None;
        // From the top, whatever the last round left.
        self.scroll.set_offset(point(px(0.), px(0.)));
        crate::sys::show_panel(window, (width + 2. * (MARGIN + EDGE)) as f64, height as f64);
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

#[derive(Clone, Copy)]
struct Colors {
    ground: Hsla,
    edge: Hsla,
    text: Hsla,
    dim: Hsla,
    pick: Hsla,
    pick_text: Hsla,
}

/// The panel's colours in the system's appearance, with the highlight
/// Settings chose (`look::highlight`), and text on it that can be read.
fn colors(window: &Window, highlight: &str) -> Colors {
    use tabflip_core::look;
    // `TABFLIP_APPEARANCE=dark|light`, for a probe, in place of the system's.
    let dark = match std::env::var("TABFLIP_APPEARANCE").as_deref() {
        Ok("dark") => true,
        Ok("light") => false,
        _ => matches!(window.appearance(), WindowAppearance::Dark | WindowAppearance::VibrantDark),
    };
    let pick = look::highlight(highlight, dark);
    let pick_text = if look::dark_text_on(pick) { rgb(0x1d1d1f).into() } else { gpui::white() };
    if dark {
        Colors { ground: rgba(0x1e1e20f2).into(), edge: rgba(0xffffff1f).into(), text: rgba(0xf5f5f7ff).into(), dim: rgba(0xf5f5f780).into(), pick: rgb(pick).into(), pick_text }
    } else {
        Colors { ground: rgba(0xf6f6f6f5).into(), edge: rgba(0x0000001a).into(), text: rgba(0x1d1d1fff).into(), dim: rgba(0x1d1d1f80).into(), pick: rgb(pick).into(), pick_text }
    }
}

impl Render for Panel {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let c = self.controller.read(cx);
        let selected = c.selected();
        let rows = c.rows.clone();
        let highlight = c.config.highlight().to_string();
        let k = self.scale;
        let row_h = ROW_H * k;

        if selected != self.last_selected {
            if let Some(s) = selected {
                // From where the highlight is drawn now, so a slide cut
                // short by the next key goes on from there.
                self.slide_from = match self.last_selected {
                    Some(_) => self.at.get(),
                    None => s as f32 * row_h,
                };
                self.slide += 1;
                self.scroll.scroll_to_item(s);
            }
            self.last_selected = selected;
        }
        let colors = colors(window, &highlight);
        let controller = self.controller.clone();
        // The highlight, one plate under the rows, sliding from the row it
        // was on to the one chosen, on gpui's clock, once a slide. The rows
        // are drawn inside the same animation, so each row's words take the
        // plate's colour exactly as far as the plate covers it: with the
        // colour flipping at the key, the chosen row's words went white on
        // the light ground before the plate arrived, and vanished.
        let (from, to, at) = (self.slide_from, selected.unwrap_or(0) as f32 * row_h, self.at.clone());
        let shown = self.shown;
        let list = div()
            .id("rows")
            .relative()
            .size_full()
            .overflow_y_scroll()
            .track_scroll(&self.scroll)
            .with_animation(ElementId::Name(format!("highlight-{}", self.slide).into()), Animation::new(SLIDE).with_easing(slide_ease), move |d, t| {
                let y = from + (to - from) * t;
                at.set(y);
                if std::env::var_os("TABFLIP_FRAME_LOG").is_some() {
                    crate::sys::log(format_args!("frame slide t={t:.3} y={y:.1}"));
                }
                let plate = selected.map(|_| div().absolute().left_0().right_0().top(px(y)).h(px(row_h)).rounded(px(7. * k)).bg(colors.pick));
                d.children(plate).children(rows.iter().cloned().enumerate().map(|(i, row)| {
                    // How much of this row the plate covers now.
                    let (top, bottom) = (i as f32 * row_h, (i + 1) as f32 * row_h);
                    let covered = if selected.is_some() { ((y + row_h).min(bottom) - y.max(top)).max(0.) / row_h } else { 0. };
                    let (hover, click) = (controller.clone(), controller.clone());
                    div()
                        .id(i)
                        .relative()
                        .h(px(row_h))
                        .flex()
                        .items_center()
                        .gap(px(8. * k))
                        .px(px(9. * k))
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
                            Some(icon) => img(icon).size(px(16. * k)).flex_shrink_0().into_any_element(),
                            None => div().size(px(16. * k)).flex_shrink_0().into_any_element(),
                        })
                        .child(div().flex_shrink_1().min_w_0().overflow_hidden().whitespace_nowrap().text_ellipsis().text_size(px(13. * k)).text_color(mix(colors.text, colors.pick_text, covered)).child(row.label))
                        .child(div().ml_auto().flex_shrink_0().max_w(px(110. * k)).overflow_hidden().whitespace_nowrap().text_ellipsis().text_size(px(11. * k)).text_color(mix(colors.dim, colors.pick_text.opacity(0.75), covered)).child(row.app))
                }))
            });

        // The stack fades in where it is, drifting a few points down into
        // place, and fades out where it is; each on gpui's clock, once a
        // round.
        let stack = div().size_full().p(px(PAD * k)).rounded(px(12. * k)).bg(colors.ground).border_1().border_color(colors.edge).shadow_lg().overflow_hidden().child(list);
        let stack = match self.leaving {
            Some(_) => stack.with_animation(ElementId::Name(format!("out-{}", self.showing).into()), Animation::new(LEAVE).with_easing(ease_out_quint()), |d, t| d.opacity(1. - t)).into_any_element(),
            None => stack.with_animation(ElementId::Name(format!("in-{}", self.showing).into()), Animation::new(ENTER).with_easing(ease_out_quint()), |d, t| d.opacity(t).mt(px(-6. * (1. - t)))).into_any_element(),
        };
        div().size_full().p(px(MARGIN)).font_family(".SystemUIFont").child(stack)
    }
}

/// The highlight's slide eases out as a cubic: the quint every other
/// motion uses did most of a 150 ms slide in its first 40, and the plate
/// read as a jump with one smeared frame.
fn slide_ease(t: f32) -> f32 {
    1. - (1. - t).powi(3)
}

/// `a` toward `b` by `t`: the rows' inks are greys and whites, whose
/// lightness and alpha mix straight.
fn mix(a: Hsla, b: Hsla, t: f32) -> Hsla {
    Hsla { h: a.h + (b.h - a.h) * t, s: a.s + (b.s - a.s) * t, l: a.l + (b.l - a.l) * t, a: a.a + (b.a - a.a) * t }
}

/// The panel's window, opened once.
pub fn install(cx: &mut App) {
    let controller = controller::get(cx);
    if let Some(handle) = open(controller.clone(), cx) {
        controller.update(cx, |c, _| c.set_panel(handle));
    }
}
