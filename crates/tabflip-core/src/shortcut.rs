//! The switcher's shortcut: modifiers held down, and a key pressed while
//! they are. ⌘Tab by default.
//!
//! Keys are the Mac's virtual key codes (US layout positions), the same
//! numbers the event tap reads, so matching an event is two comparisons.
//! The config keeps the shortcut as text ("cmd-tab") so it can be read and
//! edited by hand.

use std::fmt;

/// The Mac's modifier flags, as `CGEventGetFlags` reports them.
pub const CMD: u64 = 0x10_0000;
pub const SHIFT: u64 = 0x2_0000;
pub const CTRL: u64 = 0x4_0000;
pub const ALT: u64 = 0x8_0000;
/// The four that count; caps lock, fn and the side bits are ignored.
pub const MODS: u64 = CMD | SHIFT | CTRL | ALT;

pub const TAB: u16 = 48;
/// ` with the held modifiers goes back through the list, as ⇧ and the
/// shortcut's key do: the Mac's own ⌘` cycles windows, so the hand knows it.
pub const GRAVE: u16 = 50;
pub const RETURN: u16 = 36;
pub const KEYPAD_ENTER: u16 = 76;
pub const ESCAPE: u16 = 53;
pub const LEFT: u16 = 123;
pub const RIGHT: u16 = 124;
pub const DOWN: u16 = 125;
pub const UP: u16 = 126;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Shortcut {
    /// Held for as long as the switcher is open; letting go picks.
    pub hold: u64,
    pub key: u16,
}

impl Default for Shortcut {
    fn default() -> Self {
        Self { hold: CMD, key: TAB }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Invalid {
    /// No ⌘, ⌥ or ⌃: the switcher could never tell when to close.
    NoModifier,
    /// ⇧ goes backwards through the list, so it cannot be held as well.
    Shift,
    /// Esc, Return and the arrows are the switcher's own keys.
    Reserved,
    UnknownKey,
}

impl Invalid {
    pub const ALL: [Invalid; 4] = [Invalid::NoModifier, Invalid::Shift, Invalid::Reserved, Invalid::UnknownKey];

    /// What the window says of it, in English: the key of its other
    /// languages.
    pub fn said(&self) -> &'static str {
        match self {
            Invalid::NoModifier => "Hold ⌘, ⌥ or ⌃ with the key",
            Invalid::Shift => "⇧ is for going back; leave it out",
            Invalid::Reserved => "That key moves through the list",
            Invalid::UnknownKey => "That key can't be used",
        }
    }
}

impl fmt::Display for Invalid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.said())
    }
}

impl Shortcut {
    /// A shortcut from what was pressed, or why it cannot be one.
    pub fn new(flags: u64, key: u16) -> Result<Self, Invalid> {
        let hold = flags & MODS;
        if hold & SHIFT != 0 {
            return Err(Invalid::Shift);
        }
        if hold & (CMD | ALT | CTRL) == 0 {
            return Err(Invalid::NoModifier);
        }
        if matches!(key, ESCAPE | RETURN | KEYPAD_ENTER | LEFT | RIGHT | UP | DOWN) {
            return Err(Invalid::Reserved);
        }
        if key_name(key).is_none() {
            return Err(Invalid::UnknownKey);
        }
        Ok(Self { hold, key })
    }

    /// Whether this is the system's own ⌘Tab, which has to be switched off
    /// for the switcher to have it.
    pub fn is_command_tab(&self) -> bool {
        self.hold == CMD && self.key == TAB
    }

    /// "cmd-tab", "ctrl-alt-`": what the config keeps.
    pub fn to_config(&self) -> String {
        let mut parts: Vec<&str> = MOD_NAMES.iter().filter(|(bit, _, _)| self.hold & bit != 0).map(|(_, name, _)| *name).collect();
        parts.push(key_name(self.key).unwrap_or("tab"));
        parts.join("-")
    }

    pub fn from_config(text: &str) -> Option<Self> {
        let text = text.trim().to_ascii_lowercase();
        // The key may itself be "-", as in "cmd--".
        let (mods, key) = match text.strip_suffix("--") {
            Some(rest) => (rest, "-"),
            None => text.rsplit_once('-').unwrap_or(("", text.as_str())),
        };
        let mut hold = 0;
        for part in mods.split('-').filter(|p| !p.is_empty()) {
            hold |= MOD_NAMES.iter().find(|(_, name, _)| *name == part || alias(part) == Some(*name))?.0;
        }
        Shortcut::new(hold, key_code(key)?).ok()
    }
}

impl fmt::Display for Shortcut {
    /// "⌃⌥ Tab": the Mac's symbols in the Mac's order, then the key.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let symbols: String = MOD_NAMES.iter().filter(|(bit, _, _)| self.hold & bit != 0).map(|(_, _, s)| *s).collect();
        write!(f, "{symbols} {}", key_label(self.key))
    }
}

/// In the order the Mac writes them: ⌃⌥⇧⌘.
const MOD_NAMES: [(u64, &str, &str); 4] = [(CTRL, "ctrl", "⌃"), (ALT, "alt", "⌥"), (SHIFT, "shift", "⇧"), (CMD, "cmd", "⌘")];

fn alias(name: &str) -> Option<&'static str> {
    match name {
        "command" | "super" | "secondary" => Some("cmd"),
        "option" | "opt" => Some("alt"),
        "control" => Some("ctrl"),
        _ => None,
    }
}

/// Key codes and the names the config uses, US layout.
const KEYS: &[(u16, &str)] = &[
    (0, "a"), (11, "b"), (8, "c"), (2, "d"), (14, "e"), (3, "f"), (5, "g"), (4, "h"), (34, "i"), (38, "j"),
    (40, "k"), (37, "l"), (46, "m"), (45, "n"), (31, "o"), (35, "p"), (12, "q"), (15, "r"), (1, "s"), (17, "t"),
    (32, "u"), (9, "v"), (13, "w"), (7, "x"), (16, "y"), (6, "z"),
    (29, "0"), (18, "1"), (19, "2"), (20, "3"), (21, "4"), (23, "5"), (22, "6"), (26, "7"), (28, "8"), (25, "9"),
    (GRAVE, "`"), (27, "-"), (24, "="), (33, "["), (30, "]"), (42, "\\"), (41, ";"), (39, "'"), (43, ","), (47, "."), (44, "/"),
    (TAB, "tab"), (49, "space"), (51, "backspace"), (ESCAPE, "escape"), (RETURN, "enter"),
    (LEFT, "left"), (RIGHT, "right"), (DOWN, "down"), (UP, "up"),
    (122, "f1"), (120, "f2"), (99, "f3"), (118, "f4"), (96, "f5"), (97, "f6"),
    (98, "f7"), (100, "f8"), (101, "f9"), (109, "f10"), (103, "f11"), (111, "f12"),
];

pub fn key_name(code: u16) -> Option<&'static str> {
    KEYS.iter().find(|(c, _)| *c == code).map(|(_, n)| *n)
}

pub fn key_code(name: &str) -> Option<u16> {
    KEYS.iter().find(|(_, n)| *n == name).map(|(c, _)| *c)
}

/// How a key is written on screen: "Tab", "Space", "A", "`".
pub fn key_label(code: u16) -> String {
    match key_name(code) {
        Some("tab") => "Tab".into(),
        Some("space") => "Space".into(),
        Some("backspace") => "⌫".into(),
        Some("escape") => "Esc".into(),
        Some("enter") => "↩".into(),
        Some(name) => name.to_uppercase(),
        None => format!("Key {code}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_command_tab() {
        let s = Shortcut::default();
        assert!(s.is_command_tab());
        assert_eq!(s.to_config(), "cmd-tab");
        assert_eq!(s.to_string(), "⌘ Tab");
    }

    #[test]
    fn round_trips_through_config() {
        for text in ["cmd-tab", "alt-tab", "ctrl-alt-`", "cmd--", "ctrl-space", "cmd-f5"] {
            let s = Shortcut::from_config(text).unwrap_or_else(|| panic!("{text}"));
            assert_eq!(s.to_config(), text);
        }
        assert_eq!(Shortcut::from_config("Option-Tab").unwrap().hold, ALT);
        assert_eq!(Shortcut::from_config(" command-tab ").unwrap(), Shortcut::default());
    }

    #[test]
    fn rejects_what_cannot_work() {
        assert_eq!(Shortcut::new(0, TAB), Err(Invalid::NoModifier));
        assert_eq!(Shortcut::new(CMD | SHIFT, TAB), Err(Invalid::Shift));
        assert_eq!(Shortcut::new(CMD, ESCAPE), Err(Invalid::Reserved));
        assert_eq!(Shortcut::new(CMD, DOWN), Err(Invalid::Reserved));
        assert_eq!(Shortcut::new(CMD, 999), Err(Invalid::UnknownKey));
        assert!(Shortcut::from_config("tab").is_none());
        assert!(Shortcut::from_config("hyper-tab").is_none());
        // Caps lock and other flags are ignored.
        assert_eq!(Shortcut::new(ALT | 0x1_0000, TAB), Ok(Shortcut { hold: ALT, key: TAB }));
    }

    #[test]
    fn writes_symbols_in_mac_order() {
        assert_eq!(Shortcut { hold: CMD | CTRL | ALT, key: 50 }.to_string(), "⌃⌥⌘ `");
        assert_eq!(Shortcut { hold: ALT, key: 96 }.to_string(), "⌥ F5");
    }
}
