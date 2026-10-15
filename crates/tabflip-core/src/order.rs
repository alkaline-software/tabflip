//! Which window comes first. The windows on screen are in the order the
//! system stacks them, front to back, so the one in front is first and
//! the one used before it second. Windows that are not on screen
//! (minimized, or their app hidden) follow, most recently used first, as
//! the app has seen them used (`Recent`).

/// The order to show `windows` in, as indices into it. `id` is the
/// system's window number; `stack` lists the numbers on screen, front to
/// back. Windows with neither come last, in the order given.
pub fn order<T>(windows: &[T], id: impl Fn(&T) -> Option<u32>, stack: &[u32], recent: &Recent) -> Vec<usize> {
    let rank = |w: &T| -> (u8, usize) {
        match id(w) {
            Some(n) => match stack.iter().position(|s| *s == n) {
                Some(at) => (0, at),
                None => match recent.position(n) {
                    Some(at) => (1, at),
                    None => (2, 0),
                },
            },
            None => (2, 0),
        }
    };
    let mut ix: Vec<usize> = (0..windows.len()).collect();
    // Stable, so equal ranks keep the order they were listed in.
    ix.sort_by_key(|&i| rank(&windows[i]));
    ix
}

/// Window numbers, most recently used first.
#[derive(Debug, Default, Clone)]
pub struct Recent {
    ids: Vec<u32>,
}

const KEEP: usize = 256;

impl Recent {
    /// `id` was just used.
    pub fn touch(&mut self, id: u32) {
        self.ids.retain(|i| *i != id);
        self.ids.insert(0, id);
        self.ids.truncate(KEEP);
    }

    pub fn position(&self, id: u32) -> Option<usize> {
        self.ids.iter().position(|i| *i == id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn on_screen_by_stack_then_recent_then_rest() {
        // (id) for each window, as the apps listed them.
        let windows = [Some(10), Some(20), Some(30), None, Some(40), Some(50)];
        let stack = [20, 10];
        let mut recent = Recent::default();
        recent.touch(40);
        recent.touch(30); // 30 used after 40
        let got = order(&windows, |w| *w, &stack, &recent);
        assert_eq!(got, [1, 0, 2, 4, 3, 5]);
    }

    #[test]
    fn touch_moves_to_front_once() {
        let mut r = Recent::default();
        r.touch(1);
        r.touch(2);
        r.touch(1);
        assert_eq!(r.position(1), Some(0));
        assert_eq!(r.position(2), Some(1));
        assert_eq!(r.ids.len(), 2);
    }
}
