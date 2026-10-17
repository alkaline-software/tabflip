//! Settings, kept as JSON in `~/Library/Application Support/tabflip/config.json`
//! (`TABFLIP_HOME` puts the folder elsewhere, for a probe). Loading never
//! fails: a missing or malformed file, or a field that does not parse,
//! falls back to the default.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::look::{self, Size};
use crate::shortcut::Shortcut;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// The switcher's shortcut, as `Shortcut::to_config` writes it.
    pub shortcut: String,
    /// How long the shortcut must be held before the list shows. A quick
    /// press and release goes to the previous window without it.
    pub appear_delay_ms: u64,
    /// The list's size: "small", "medium" or "large".
    pub size: String,
    /// The selected row's colour: a preset's name ("blue") or "#rrggbb".
    pub highlight: String,
    /// Whether the item stands in the menu bar. Without it, Settings are
    /// reached by opening tabflip again from Finder or Spotlight.
    pub menu_bar_icon: bool,
    /// The item's clicks swapped: the menu on a left click, Settings on
    /// a right one.
    pub menu_bar_swap: bool,
    /// The window's language: "system", or a code the window has
    /// (`i18n::CHOICES`). On a Mac the app's own `AppleLanguages` is the
    /// setting itself, and this follows it.
    pub language: String,
    /// Whether a newer release is looked for once a day.
    pub check_updates: bool,
    /// When a newer release was last looked for, Unix seconds; 0 = never.
    pub last_update_check: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self { shortcut: Shortcut::default().to_config(), appear_delay_ms: 120, size: Size::default().to_config(), highlight: look::DEFAULT_HIGHLIGHT.into(), menu_bar_icon: true, menu_bar_swap: false, language: "system".into(), check_updates: true, last_update_check: 0 }
    }
}

impl Config {
    pub fn load() -> Self {
        std::fs::read(path()).ok().and_then(|bytes| serde_json::from_slice(&bytes).ok()).unwrap_or_default()
    }

    /// Written to a temporary file and moved over the old one, so a crash
    /// never leaves half a file.
    pub fn save(&self) -> std::io::Result<()> {
        let path = path();
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let tmp = path.with_extension("json.tmp");
        std::fs::write(&tmp, serde_json::to_vec_pretty(self).unwrap_or_default())?;
        std::fs::rename(tmp, path)
    }

    /// The shortcut, or the default when the text is not one.
    pub fn shortcut(&self) -> Shortcut {
        Shortcut::from_config(&self.shortcut).unwrap_or_default()
    }

    pub fn set_shortcut(&mut self, shortcut: Shortcut) {
        self.shortcut = shortcut.to_config();
    }

    pub fn size(&self) -> Size {
        Size::from_config(&self.size)
    }

    pub fn set_size(&mut self, size: Size) {
        self.size = size.to_config();
    }

    /// The highlight as the config gives it, or the default when it is
    /// neither a preset nor a colour.
    pub fn highlight(&self) -> &str {
        if look::is_highlight(&self.highlight) {
            &self.highlight
        } else {
            look::DEFAULT_HIGHLIGHT
        }
    }

    /// A preset's name, or a colour, written as `#rrggbb`. Anything else
    /// is refused.
    pub fn set_highlight(&mut self, text: &str) -> bool {
        if let Some(p) = look::preset(text) {
            self.highlight = p.name.to_lowercase();
        } else if let Some(rgb) = look::parse_hex(text) {
            self.highlight = look::to_hex(rgb);
        } else {
            return false;
        }
        true
    }

    /// Kept between 0 and a second, whatever the file says.
    pub fn appear_delay_ms(&self) -> u64 {
        self.appear_delay_ms.min(1000)
    }
}

/// The folder tabflip keeps its files in.
pub fn home() -> PathBuf {
    if let Some(dir) = std::env::var_os("TABFLIP_HOME") {
        return PathBuf::from(dir);
    }
    dirs::config_dir().unwrap_or_else(std::env::temp_dir).join("tabflip")
}

pub fn path() -> PathBuf {
    home().join("config.json")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn look_fields_parse_or_fall_back() {
        let c: Config = serde_json::from_str(r##"{"size":"large","highlight":"#F80"}"##).unwrap();
        assert_eq!(c.size(), Size::Large);
        assert_eq!(c.highlight(), "#F80");
        let c: Config = serde_json::from_str(r#"{"size":"enormous","highlight":"chartreuse"}"#).unwrap();
        assert_eq!(c.size(), Size::Small);
        assert_eq!(c.highlight(), "blue");

        let mut c = Config::default();
        assert!(c.set_highlight("Green"));
        assert_eq!(c.highlight, "green");
        assert!(c.set_highlight("#F80"));
        assert_eq!(c.highlight, "#ff8800");
        assert!(!c.set_highlight("#ff88"));
        assert_eq!(c.highlight, "#ff8800");
    }

    #[test]
    fn missing_fields_take_defaults() {
        let c: Config = serde_json::from_str(r#"{"shortcut":"alt-tab"}"#).unwrap();
        assert_eq!(c.appear_delay_ms, 120);
        assert!(c.menu_bar_icon && c.check_updates && !c.menu_bar_swap);
        assert_eq!(c.language, "system");
        assert_eq!(c.shortcut().to_config(), "alt-tab");
    }

    #[test]
    fn a_bad_shortcut_is_the_default() {
        let c = Config { shortcut: "nonsense".into(), ..Config::default() };
        assert_eq!(c.shortcut(), Shortcut::default());
    }

    #[test]
    fn delay_is_clamped() {
        let c = Config { appear_delay_ms: 60_000, ..Config::default() };
        assert_eq!(c.appear_delay_ms(), 1000);
    }
}
