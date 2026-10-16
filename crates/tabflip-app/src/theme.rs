//! The Settings window's look: the palette Alkaline Software's apps share
//! (`themes/tabflip.json`, one config per appearance, near white with a
//! trace of warmth in the light appearance), with tabflip's accent, the
//! icon's blue, painted into both configs before the toolkit gets them,
//! so `Theme::change` keeps it on every appearance switch.
//!
//! The switcher's list is drawn in its own colours (`panel.rs`), not these.

use gpui::App;
use gpui_component::theme::{Theme, ThemeConfig, ThemeSet};

/// The accent in each appearance (base, hover, active) and the link
/// colour, darker in the light palette and lighter in the dark one so
/// text on it stays readable.
struct Accent {
    light: [&'static str; 3],
    dark: [&'static str; 3],
    link: [&'static str; 2],
}

const BLUE: Accent = Accent { light: ["#2F6FDB", "#2862C4", "#2255AC"], dark: ["#3D84F0", "#5A97F3", "#2F6FDB"], link: ["#2255AC", "#7AAAF5"] };

/// The inks the palette names by name ("@l-blue", "@d-rose"), one set an
/// appearance, for what in the toolkit draws code.
struct Inks {
    blue: u32,
    violet: u32,
    gold: u32,
    rose: u32,
    teal: u32,
    green: u32,
    red: u32,
    grey: u32,
}

const INKS_LIGHT: Inks = Inks { blue: 0x286983, violet: 0x7A6398, gold: 0x8A6A1E, rose: 0xA3566F, teal: 0x357783, green: 0x4C7B58, red: 0xB0505F, grey: 0x85818F };
const INKS_DARK: Inks = Inks { blue: 0x7FB4D0, violet: 0xC4A7E7, gold: 0xE9C083, rose: 0xE8B0AE, teal: 0x9CCFD8, green: 0xA3CCA4, red: 0xEB8FA6, grey: 0x8F8BA0 };

/// The theme file with each ink's name replaced by its colour.
fn inked(json: &str) -> String {
    let mut out = json.to_string();
    for (p, inks) in [("l", &INKS_LIGHT), ("d", &INKS_DARK)] {
        for (name, c) in [("blue", inks.blue), ("violet", inks.violet), ("gold", inks.gold), ("rose", inks.rose), ("teal", inks.teal), ("green", inks.green), ("red", inks.red), ("grey", inks.grey)] {
            out = out.replace(&format!("\"@{p}-{name}\""), &format!("\"#{c:06X}\""));
        }
    }
    out
}

fn paint(cfg: &mut ThemeConfig, a: &Accent) {
    let dark = cfg.mode.is_dark();
    let [base, hover, active] = if dark { a.dark } else { a.light };
    let link = if dark { a.link[1] } else { a.link[0] };
    let c = &mut cfg.colors;
    c.primary = Some(base.into());
    c.primary_hover = Some(hover.into());
    c.primary_active = Some(active.into());
    c.ring = Some(base.into());
    c.link = Some(link.into());
    c.link_hover = Some(active.into());
    c.link_active = Some(active.into());
    c.sidebar_primary = Some(base.into());
    c.list_active_border = Some(base.into());
    c.drag_border = Some(base.into());
    c.progress_bar = Some(base.into());
    c.slider_bar = Some(base.into());
    c.drop_target = Some(format!("{base}33").into());
    c.selection = Some(format!("{base}{}", if dark { "66" } else { "59" }).into());
}

/// Hand the toolkit both palettes with the accent painted in, once at
/// start. The window then follows the system's appearance.
pub fn install(cx: &mut App) {
    let set: ThemeSet = match serde_json::from_str(&inked(include_str!("../themes/tabflip.json"))) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("tabflip: theme file is invalid, using the default look: {e}");
            return;
        }
    };
    let theme = Theme::global_mut(cx);
    for mut cfg in set.themes {
        paint(&mut cfg, &BLUE);
        let cfg = std::rc::Rc::new(cfg);
        if cfg.mode.is_dark() {
            theme.dark_theme = cfg;
        } else {
            theme.light_theme = cfg;
        }
    }
    Theme::sync_system_appearance(None, cx);
}
