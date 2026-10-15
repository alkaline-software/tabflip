//! Settings, kept as JSON in `~/Library/Application Support/tabflip/config.json`
//! (`TABFLIP_HOME` puts the folder elsewhere, for a probe). Loading never
//! fails: a missing or malformed file, or a field that does not parse,
//! falls back to the default.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::shortcut::Shortcut;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// The switcher's shortcut, as `Shortcut::to_config` writes it.
    pub shortcut: String,
    /// How long the shortcut must be held before the list shows. A quick
    /// press and release goes to the previous window without it.
    pub appear_delay_ms: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self { shortcut: Shortcut::default().to_config(), appear_delay_ms: 120 }
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
    fn missing_fields_take_defaults() {
        let c: Config = serde_json::from_str(r#"{"shortcut":"alt-tab"}"#).unwrap();
        assert_eq!(c.appear_delay_ms, 120);
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
