//! Whether a newer tabflip is out, and where to get it.
//!
//! Releases live on GitHub under stable asset names (WORKFLOW.md), so no
//! API is needed: `releases/latest` answers with a redirect to the newest
//! published release's tag, and its installer is at
//! `releases/download/v<version>/<asset>`. Drafts are not seen until they
//! are published. tabflip does not install anything itself: Settings
//! offers the installer to download, and the person opens it.

use std::time::Duration;

pub const REPO: &str = "alkaline-software/tabflip";

/// How long an automatic check waits before looking again.
pub const DAILY: Duration = Duration::from_secs(24 * 60 * 60);

/// The version this binary is.
pub fn current_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// The installer for this Mac, as the release names it.
pub fn asset_name() -> &'static str {
    if cfg!(target_arch = "aarch64") {
        "tabflip-mac-arm64.dmg"
    } else {
        "tabflip-mac-x64.dmg"
    }
}

pub fn repo_page() -> String {
    format!("https://github.com/{REPO}")
}

pub fn release_page(version: &str) -> String {
    format!("https://github.com/{REPO}/releases/tag/v{version}")
}

pub fn asset_url(version: &str) -> String {
    format!("https://github.com/{REPO}/releases/download/v{version}/{}", asset_name())
}

/// The version a `releases/latest` redirect points at: the tag after
/// `/releases/tag/`, without its `v`.
pub fn version_from_location(location: &str) -> Option<String> {
    let (_, tail) = location.rsplit_once("/releases/tag/")?;
    let tag = tail.trim().trim_end_matches('/');
    let v = tag.strip_prefix('v').unwrap_or(tag);
    v.starts_with(|c: char| c.is_ascii_digit()).then(|| v.to_string())
}

/// The `location` header of an HTTP answer, as `curl -I` prints it.
pub fn location_in_headers(headers: &str) -> Option<&str> {
    headers.lines().find_map(|l| {
        let (name, value) = l.split_once(':')?;
        name.trim().eq_ignore_ascii_case("location").then(|| value.trim())
    })
}

/// Dotted version numbers compared part by part; anything unparsable is 0.
pub fn version_cmp(a: &str, b: &str) -> std::cmp::Ordering {
    let parts = |s: &str| -> Vec<u64> { s.split('.').map(|p| p.trim().parse().unwrap_or(0)).collect() };
    let (a, b) = (parts(a), parts(b));
    (0..a.len().max(b.len())).map(|i| a.get(i).copied().unwrap_or(0).cmp(&b.get(i).copied().unwrap_or(0))).find(|o| o.is_ne()).unwrap_or(std::cmp::Ordering::Equal)
}

pub fn is_newer(latest: &str, current: &str) -> bool {
    version_cmp(latest, current).is_gt()
}

/// Whether an automatic check is due, `last` and `now` in Unix seconds.
pub fn due(last: u64, now: u64) -> bool {
    now.saturating_sub(last) >= DAILY.as_secs()
}

/// The newest published version, asked of GitHub, or None while nothing
/// is published (the redirect then goes to the list of releases). `curl`, which every
/// Mac has, keeps an HTTPS client out of the app. Blocks for up to ten
/// seconds: call it off the main thread.
pub fn latest() -> Result<Option<String>, String> {
    let url = format!("https://github.com/{REPO}/releases/latest");
    let out = std::process::Command::new("/usr/bin/curl").args(["-sSI", "--max-time", "10", &url]).output().map_err(|e| e.to_string())?;
    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).trim().to_string());
    }
    let headers = String::from_utf8_lossy(&out.stdout);
    match location_in_headers(&headers) {
        Some(location) => Ok(version_from_location(location)),
        None => Err(headers.lines().next().unwrap_or("no answer").trim().to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_redirect_names_the_version() {
        assert_eq!(version_from_location("https://github.com/alkaline-software/tabflip/releases/tag/v0.2.0").as_deref(), Some("0.2.0"));
        assert_eq!(version_from_location("https://github.com/x/y/releases/tag/1.4/").as_deref(), Some("1.4"));
        assert_eq!(version_from_location("https://github.com/x/y/releases"), None);
        assert_eq!(version_from_location("https://github.com/x/y/releases/tag/nightly"), None);
    }

    #[test]
    fn location_is_read_from_the_headers() {
        let headers = "HTTP/2 302\r\nserver: github.com\r\nLocation: https://github.com/x/y/releases/tag/v0.1.3\r\n\r\n";
        assert_eq!(location_in_headers(headers), Some("https://github.com/x/y/releases/tag/v0.1.3"));
        assert_eq!(location_in_headers("HTTP/2 404\r\n\r\n"), None);
    }

    #[test]
    fn versions_compare_by_number() {
        assert!(is_newer("0.1.10", "0.1.9"));
        assert!(is_newer("1.0", "0.9.9"));
        assert!(!is_newer("0.1.2", "0.1.2"));
        assert!(!is_newer("0.1", "0.1.0"));
        assert!(!is_newer("0.1.1", "0.1.2"));
    }

    #[test]
    fn a_check_is_due_once_a_day() {
        assert!(due(0, 1_000_000));
        assert!(!due(1_000_000, 1_000_000 + 3600));
        assert!(due(1_000_000, 1_000_000 + 86_400));
        assert!(!due(2_000, 1_000));
    }
}
