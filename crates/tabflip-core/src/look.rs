//! How the list looks: its size, and the colour of the selected row.
//!
//! Both are kept in the config as text, so a value from a newer or
//! hand-edited file that does not parse falls back to the default rather
//! than costing the whole file.

/// The list's size. Small is the original: 30-point rows, 13-point text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Size {
    #[default]
    Small,
    Medium,
    Large,
}

impl Size {
    pub const ALL: [Size; 3] = [Size::Small, Size::Medium, Size::Large];

    /// How much bigger than Small every measure of the list is.
    pub fn scale(self) -> f32 {
        match self {
            Size::Small => 1.,
            Size::Medium => 1.25,
            Size::Large => 1.5,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Size::Small => "Small",
            Size::Medium => "Medium",
            Size::Large => "Large",
        }
    }

    pub fn to_config(self) -> String {
        self.name().to_lowercase()
    }

    /// The size the config names, or Small.
    pub fn from_config(text: &str) -> Size {
        Size::ALL.into_iter().find(|s| s.name().eq_ignore_ascii_case(text.trim())).unwrap_or_default()
    }
}

/// A highlight colour offered by name: the system's own colours, each in
/// its light and dark appearance, as `0xRRGGBB`.
pub struct Preset {
    pub name: &'static str,
    pub light: u32,
    pub dark: u32,
}

pub const PRESETS: [Preset; 8] = [
    Preset { name: "Blue", light: 0x007aff, dark: 0x0a84ff },
    Preset { name: "Purple", light: 0xaf52de, dark: 0xbf5af2 },
    Preset { name: "Pink", light: 0xff2d55, dark: 0xff375f },
    Preset { name: "Red", light: 0xff3b30, dark: 0xff453a },
    Preset { name: "Orange", light: 0xff9500, dark: 0xff9f0a },
    Preset { name: "Green", light: 0x34c759, dark: 0x30d158 },
    Preset { name: "Teal", light: 0x30b0c7, dark: 0x40c8e0 },
    Preset { name: "Graphite", light: 0x8e8e93, dark: 0x98989d },
];

pub const DEFAULT_HIGHLIGHT: &str = "blue";

/// The preset a config value names, if it names one.
pub fn preset(text: &str) -> Option<&'static Preset> {
    PRESETS.iter().find(|p| p.name.eq_ignore_ascii_case(text.trim()))
}

/// `#rgb` or `#rrggbb`, with or without the `#`, as `0xRRGGBB`.
pub fn parse_hex(text: &str) -> Option<u32> {
    let digits = text.trim().trim_start_matches('#');
    if !digits.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    match digits.len() {
        6 => u32::from_str_radix(digits, 16).ok(),
        3 => {
            let short = u32::from_str_radix(digits, 16).ok()?;
            let (r, g, b) = (short >> 8 & 0xf, short >> 4 & 0xf, short & 0xf);
            Some((r * 0x11) << 16 | (g * 0x11) << 8 | b * 0x11)
        }
        _ => None,
    }
}

/// `0xRRGGBB` as the config writes a colour of one's own: `#rrggbb`.
pub fn to_hex(rgb: u32) -> String {
    format!("#{:06x}", rgb & 0xff_ffff)
}

/// Whether a config value is a highlight: a preset's name or a hex colour.
pub fn is_highlight(text: &str) -> bool {
    preset(text).is_some() || parse_hex(text).is_some()
}

/// The highlight a config value gives, in the light or dark appearance:
/// a preset follows the appearance, a hex colour is the same in both, and
/// anything else is Blue.
pub fn highlight(text: &str, dark: bool) -> u32 {
    let pick = |p: &Preset| if dark { p.dark } else { p.light };
    preset(text).map(pick).or_else(|| parse_hex(text)).unwrap_or_else(|| pick(&PRESETS[0]))
}

/// Whether text on this colour should be dark rather than white, so it
/// can be read on a pale highlight (yellow, light grey) as on a deep one.
pub fn dark_text_on(rgb: u32) -> bool {
    let lin = |c: u32| {
        let c = c as f32 / 255.;
        if c <= 0.04045 { c / 12.92 } else { ((c + 0.055) / 1.055).powf(2.4) }
    };
    let l = 0.2126 * lin(rgb >> 16 & 0xff) + 0.7152 * lin(rgb >> 8 & 0xff) + 0.0722 * lin(rgb & 0xff);
    // Well above where black and white have equal contrast (0.18): the
    // system's own colours, its green and orange included, keep white
    // text, as macOS draws them; only pale ones take dark.
    l > 0.5
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sizes_round_trip_and_unknown_is_small() {
        for s in Size::ALL {
            assert_eq!(Size::from_config(&s.to_config()), s);
        }
        assert_eq!(Size::from_config("LARGE"), Size::Large);
        assert_eq!(Size::from_config("huge"), Size::Small);
        assert!(Size::Small.scale() < Size::Medium.scale() && Size::Medium.scale() < Size::Large.scale());
    }

    #[test]
    fn hex_long_short_and_wrong() {
        assert_eq!(parse_hex("#0a84ff"), Some(0x0a84ff));
        assert_eq!(parse_hex("0A84FF"), Some(0x0a84ff));
        assert_eq!(parse_hex(" #f80 "), Some(0xff8800));
        assert_eq!(parse_hex("#12345"), None);
        assert_eq!(parse_hex("#gggggg"), None);
        assert_eq!(parse_hex("+12345"), None);
        assert_eq!(parse_hex(""), None);
        assert_eq!(to_hex(0x0a84ff), "#0a84ff");
    }

    #[test]
    fn presets_follow_the_appearance_hex_does_not() {
        assert_eq!(highlight("blue", false), 0x007aff);
        assert_eq!(highlight("Blue", true), 0x0a84ff);
        assert_eq!(highlight("green", true), 0x30d158);
        assert_eq!(highlight("#123456", false), 0x123456);
        assert_eq!(highlight("#123456", true), 0x123456);
        assert_eq!(highlight("nonsense", true), 0x0a84ff);
        assert!(is_highlight("teal") && is_highlight("#fff") && !is_highlight("teel"));
    }

    #[test]
    fn text_is_dark_only_on_pale_colours() {
        assert!(dark_text_on(0xffd60a)); // yellow
        assert!(dark_text_on(0xffffff));
        assert!(!dark_text_on(0x0a84ff));
        assert!(!dark_text_on(0x000000));
        assert!(!dark_text_on(0xaf52de));
        for p in &PRESETS {
            assert!(!dark_text_on(p.light) && !dark_text_on(p.dark), "{}", p.name);
        }
    }
}
