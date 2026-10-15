//! What the switcher does with each key, as states and the effects the
//! app carries out. No clock and no OS here: the app says when the delay
//! has passed and when the list is ready.
//!
//! A press of the shortcut starts a round. The windows are listed while
//! the shortcut is held, and the list shows once the appear delay has
//! passed and the windows are in. Letting go of the held modifiers picks
//! the selected window, at any point: a quick press and release picks the
//! window used before this one without the list ever showing.
//!
//! The selection is kept as a step count from the first window until the
//! list's length is known, so keys pressed before the windows are in
//! still count.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Effect {
    /// A round began: list the windows and start the delay, both tagged
    /// with this round's number.
    Begin(u64),
    /// Show the list.
    Show,
    /// The selection moved while the list shows.
    Redraw,
    /// Bring this window (an index into the list) to the front, and close.
    Focus(usize),
    /// Close without changing anything.
    Close,
}

#[derive(Debug, Default)]
pub struct Switcher {
    round: u64,
    active: bool,
    steps: i64,
    len: Option<usize>,
    delay_done: bool,
    shown: bool,
    released: bool,
}

impl Switcher {
    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn is_shown(&self) -> bool {
        self.shown
    }

    pub fn round(&self) -> u64 {
        self.round
    }

    /// The selected window, once the list is in.
    pub fn selected(&self) -> Option<usize> {
        match self.len {
            Some(0) | None => None,
            Some(n) => Some(self.steps.rem_euclid(n as i64) as usize),
        }
    }

    /// The shortcut was pressed, with ⇧ when `back`.
    pub fn trigger(&mut self, back: bool) -> Option<Effect> {
        if !self.active {
            self.round += 1;
            *self = Switcher { round: self.round, active: true, steps: if back { -1 } else { 1 }, ..Default::default() };
            return Some(Effect::Begin(self.round));
        }
        self.step(if back { -1 } else { 1 })
    }

    /// Up or down a row.
    pub fn step(&mut self, by: i64) -> Option<Effect> {
        if !self.active {
            return None;
        }
        self.steps += by;
        self.shown.then_some(Effect::Redraw)
    }

    /// The pointer is over row `i`.
    pub fn hover(&mut self, i: usize) -> Option<Effect> {
        if !self.shown || self.selected() == Some(i) {
            return None;
        }
        self.steps = i as i64;
        Some(Effect::Redraw)
    }

    /// Row `i` was clicked.
    pub fn click(&mut self, i: usize) -> Option<Effect> {
        if !self.shown || self.len.is_none_or(|n| i >= n) {
            return None;
        }
        self.steps = i as i64;
        self.finish()
    }

    /// The round's windows are in.
    pub fn listed(&mut self, round: u64, len: usize) -> Option<Effect> {
        if !self.active || round != self.round {
            return None;
        }
        self.len = Some(len);
        if self.released {
            return self.finish();
        }
        self.show_if_ready()
    }

    /// The round's appear delay has passed.
    pub fn delay_passed(&mut self, round: u64) -> Option<Effect> {
        if !self.active || round != self.round {
            return None;
        }
        self.delay_done = true;
        self.show_if_ready()
    }

    /// The held modifiers were let go, or Return was pressed.
    pub fn release(&mut self) -> Option<Effect> {
        if !self.active {
            return None;
        }
        if self.len.is_none() {
            // Picked as soon as the windows are in.
            self.released = true;
            return None;
        }
        self.finish()
    }

    pub fn cancel(&mut self) -> Option<Effect> {
        if !self.active {
            return None;
        }
        self.end();
        Some(Effect::Close)
    }

    fn show_if_ready(&mut self) -> Option<Effect> {
        if self.delay_done && self.len.is_some() && !self.shown {
            self.shown = true;
            return Some(Effect::Show);
        }
        None
    }

    fn finish(&mut self) -> Option<Effect> {
        let picked = self.selected();
        self.end();
        Some(picked.map_or(Effect::Close, Effect::Focus))
    }

    fn end(&mut self) {
        let round = self.round;
        *self = Switcher { round, ..Default::default() };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Effect::*;

    #[test]
    fn quick_press_picks_the_previous_window_unseen() {
        let mut s = Switcher::default();
        assert_eq!(s.trigger(false), Some(Begin(1)));
        assert_eq!(s.listed(1, 4), None);
        assert_eq!(s.release(), Some(Focus(1)));
        assert!(!s.is_active());
    }

    #[test]
    fn release_before_the_list_is_in_waits_for_it() {
        let mut s = Switcher::default();
        s.trigger(false);
        assert_eq!(s.release(), None);
        assert_eq!(s.listed(1, 3), Some(Focus(1)));
    }

    #[test]
    fn shows_after_delay_and_list_in_either_order() {
        let mut s = Switcher::default();
        s.trigger(false);
        assert_eq!(s.delay_passed(1), None);
        assert_eq!(s.listed(1, 3), Some(Show));

        let mut s = Switcher::default();
        s.trigger(false);
        assert_eq!(s.listed(1, 3), None);
        assert_eq!(s.delay_passed(1), Some(Show));
        assert_eq!(s.delay_passed(1), None);
    }

    #[test]
    fn tab_cycles_and_wraps_shift_goes_back() {
        let mut s = Switcher::default();
        s.trigger(false);
        s.listed(1, 3);
        s.delay_passed(1);
        assert_eq!(s.selected(), Some(1));
        assert_eq!(s.trigger(false), Some(Redraw));
        assert_eq!(s.selected(), Some(2));
        s.trigger(false); // autorepeat counts the same
        assert_eq!(s.selected(), Some(0));
        s.trigger(true);
        assert_eq!(s.selected(), Some(2));
        assert_eq!(s.release(), Some(Focus(2)));
    }

    #[test]
    fn shift_first_starts_at_the_last() {
        let mut s = Switcher::default();
        s.trigger(true);
        s.listed(1, 5);
        assert_eq!(s.selected(), Some(4));
    }

    #[test]
    fn keys_before_the_list_still_count() {
        let mut s = Switcher::default();
        s.trigger(false);
        s.trigger(false);
        s.trigger(false);
        s.listed(1, 2);
        assert_eq!(s.selected(), Some(1));
    }

    #[test]
    fn escape_cancels_and_stale_rounds_are_ignored() {
        let mut s = Switcher::default();
        s.trigger(false);
        assert_eq!(s.cancel(), Some(Close));
        assert_eq!(s.listed(1, 3), None);
        assert_eq!(s.trigger(false), Some(Begin(2)));
        assert_eq!(s.delay_passed(1), None);
        assert_eq!(s.listed(1, 3), None);
        assert_eq!(s.listed(2, 3), None);
        assert_eq!(s.delay_passed(2), Some(Show));
    }

    #[test]
    fn one_window_or_none() {
        let mut s = Switcher::default();
        s.trigger(false);
        s.listed(1, 1);
        assert_eq!(s.release(), Some(Focus(0)));

        s.trigger(false);
        s.listed(2, 0);
        assert_eq!(s.release(), Some(Close));
    }

    #[test]
    fn pointer_selects_and_clicks() {
        let mut s = Switcher::default();
        s.trigger(false);
        assert_eq!(s.hover(2), None); // not shown yet
        s.listed(1, 4);
        s.delay_passed(1);
        assert_eq!(s.hover(3), Some(Redraw));
        assert_eq!(s.hover(3), None);
        assert_eq!(s.click(9), None);
        assert_eq!(s.click(2), Some(Focus(2)));
    }
}
