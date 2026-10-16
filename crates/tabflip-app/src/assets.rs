//! Our own icons and pictures, over the set gpui-component ships.
//!
//! `Icon::default().path(…)` and `img(…)` resolve through the app's
//! `AssetSource`; this answers for the files named in `OWN` and hands
//! everything else to the bundled set. The icons are Tabler's outline
//! icons (MIT, `TABLER-LICENSE` beside them). A file not named in `OWN`
//! draws nothing, and nothing says so.

use std::borrow::Cow;

use gpui::{AssetSource, Result, SharedString};

pub struct Assets;

const OWN: &[(&str, &[u8])] = &[
    ("icon/app-large.png", include_bytes!("../assets/icon/icon-512.png")),
    ("icons/check.svg", include_bytes!("../assets/icons/check.svg")),
    ("icons/download.svg", include_bytes!("../assets/icons/download.svg")),
    ("icons/external-link.svg", include_bytes!("../assets/icons/external-link.svg")),
    ("icons/file-text.svg", include_bytes!("../assets/icons/file-text.svg")),
    ("icons/github.svg", include_bytes!("../assets/icons/github.svg")),
    ("icons/info.svg", include_bytes!("../assets/icons/info.svg")),
    ("icons/pencil.svg", include_bytes!("../assets/icons/pencil.svg")),
    ("icons/redo-2.svg", include_bytes!("../assets/icons/redo-2.svg")),
    ("icons/rotate.svg", include_bytes!("../assets/icons/rotate.svg")),
    ("icons/settings.svg", include_bytes!("../assets/icons/settings.svg")),
    ("icons/shield.svg", include_bytes!("../assets/icons/shield.svg")),
    ("icons/sparkles.svg", include_bytes!("../assets/icons/sparkles.svg")),
    ("icons/triangle-alert.svg", include_bytes!("../assets/icons/triangle-alert.svg")),
];

impl AssetSource for Assets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        if let Some((_, bytes)) = OWN.iter().find(|(p, _)| *p == path) {
            return Ok(Some(Cow::Borrowed(bytes)));
        }
        gpui_component_assets::Assets.load(path)
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        let mut v = gpui_component_assets::Assets.list(path)?;
        v.extend(OWN.iter().filter(|(p, _)| p.starts_with(path)).map(|(p, _)| SharedString::from(*p)));
        Ok(v)
    }
}
