//! Keeping tabflip current: whether a newer one is out, fetching its disk
//! image, and putting it in place.
//!
//! Releases live on GitHub under stable asset names (WORKFLOW.md), so no
//! API is needed: `releases/latest` answers with a redirect to the newest
//! published release's tag, and its installer is at
//! `releases/download/v<version>/<asset>`. Drafts are not seen until they
//! are published. `curl`, which every Mac has, does the talking, so the
//! app carries no HTTPS client.
//!
//! Installing: the disk image is mounted, `tabflip.app` copied out beside
//! the running bundle with `ditto` (which keeps the signature and the
//! attributes), and swapped in with two renames. The old process keeps
//! running from its unlinked files until the app quits, and the new app
//! is opened once it has: never two at once.

use std::path::{Path, PathBuf};
use std::process::Command;
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

/// Where the disk image is kept while it is fetched.
pub fn downloads_dir() -> PathBuf {
    dirs::cache_dir().unwrap_or_else(std::env::temp_dir).join("tabflip").join("updates")
}

/// How big the file at `url` is, as the server says after its redirects.
fn content_length(url: &str) -> Option<u64> {
    let out = Command::new("/usr/bin/curl").args(["-sIL", "--max-time", "20", url]).output().ok()?;
    let headers = String::from_utf8_lossy(&out.stdout);
    headers.lines().filter_map(|l| {
        let (name, value) = l.split_once(':')?;
        name.trim().eq_ignore_ascii_case("content-length").then(|| value.trim().parse::<u64>().ok()).flatten()
    }).last()
}

/// Fetch this Mac's disk image for `version` into the cache, reporting
/// bytes so far and the total when known. Answers the file. Blocks for
/// the download: call it off the main thread.
pub fn download(version: &str, progress: &dyn Fn(u64, Option<u64>)) -> Result<PathBuf, String> {
    let dir = downloads_dir();
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let dest = dir.join(asset_name());
    let tmp = dir.join(format!("{}.part", asset_name()));
    let _ = std::fs::remove_file(&tmp);
    let url = asset_url(version);
    let total = content_length(&url);
    progress(0, total);
    let mut child = Command::new("/usr/bin/curl").args(["-sSL", "--fail", "--max-time", "900", "-o"]).arg(&tmp).arg(&url).stderr(std::process::Stdio::piped()).spawn().map_err(|e| e.to_string())?;
    loop {
        if let Some(status) = child.try_wait().map_err(|e| e.to_string())? {
            if !status.success() {
                let mut said = String::new();
                if let Some(mut err) = child.stderr.take() {
                    let _ = std::io::Read::read_to_string(&mut err, &mut said);
                }
                let _ = std::fs::remove_file(&tmp);
                return Err(if said.trim().is_empty() { "the download stopped".into() } else { said.trim().to_string() });
            }
            break;
        }
        std::thread::sleep(Duration::from_millis(150));
        progress(std::fs::metadata(&tmp).map(|m| m.len()).unwrap_or(0), total);
    }
    let done = std::fs::metadata(&tmp).map(|m| m.len()).unwrap_or(0);
    if let Some(t) = total {
        if done != t {
            let _ = std::fs::remove_file(&tmp);
            return Err(format!("the download ended early ({done} of {t} bytes)"));
        }
    }
    progress(done, total);
    std::fs::rename(&tmp, &dest).map_err(|e| e.to_string())?;
    Ok(dest)
}

/// The `.app` the running binary lives in, if it is in one.
pub fn bundle_path() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    exe.ancestors().find(|p| p.extension().is_some_and(|e| e == "app")).map(Path::to_path_buf)
}

/// Put the fetched disk image's app in place of the running bundle and
/// have the new one open once this process has ended. On success the
/// caller quits.
pub fn install(dmg: &Path) -> Result<(), String> {
    let bundle = bundle_path().ok_or("tabflip is not running from an app bundle here; open the disk image and drag tabflip to Applications instead")?;
    install_into(dmg, &bundle, true)
}

/// `install` with the bundle to replace named, and whether the new app is
/// opened once this process ends (a probe tries the swap on a throwaway
/// bundle and opens nothing).
pub fn install_into(dmg: &Path, bundle: &Path, open: bool) -> Result<(), String> {
    let parent = bundle.parent().ok_or("the bundle has no parent folder")?;
    let mount = downloads_dir().join("mnt");
    let _ = Command::new("hdiutil").args(["detach", "-quiet"]).arg(&mount).output();
    std::fs::create_dir_all(&mount).map_err(|e| e.to_string())?;
    let out = Command::new("hdiutil").args(["attach", "-nobrowse", "-readonly", "-quiet", "-mountpoint"]).arg(&mount).arg(dmg).output().map_err(|e| e.to_string())?;
    if !out.status.success() {
        return Err(format!("the disk image would not mount: {}", String::from_utf8_lossy(&out.stderr).trim()));
    }
    let detach = || {
        let _ = Command::new("hdiutil").args(["detach", "-quiet"]).arg(&mount).output();
    };
    let fresh = mount.join("tabflip.app");
    if !fresh.is_dir() {
        detach();
        return Err("the disk image holds no tabflip.app".into());
    }
    let staged = parent.join("tabflip.app.update");
    let old = parent.join("tabflip.app.old");
    let _ = std::fs::remove_dir_all(&staged);
    let _ = std::fs::remove_dir_all(&old);
    let out = Command::new("ditto").arg(&fresh).arg(&staged).output().map_err(|e| e.to_string())?;
    detach();
    if !out.status.success() {
        let _ = std::fs::remove_dir_all(&staged);
        return Err(format!("could not copy the new app next to this one: {}", String::from_utf8_lossy(&out.stderr).trim()));
    }
    std::fs::rename(bundle, &old).map_err(|e| format!("could not move the current app aside ({}): {e}", bundle.display()))?;
    if let Err(e) = std::fs::rename(&staged, bundle) {
        let _ = std::fs::rename(&old, bundle);
        let _ = std::fs::remove_dir_all(&staged);
        return Err(format!("could not put the new app in place: {e}"));
    }
    let _ = std::fs::remove_dir_all(&old);
    if open {
        open_once_gone(std::process::id(), bundle).map_err(|e| format!("the new app is in place but did not start: {e}"))?;
    }
    Ok(())
}

/// Open `bundle` once the process `pid` has ended: a shell of its own
/// that outlives this one and waits on the pid, not on a length of time.
/// The new app must not start while the old one runs: two would each
/// take the shortcut, and the old one gives the Mac its ⌘Tab back as it
/// quits, after the new one took it.
fn open_once_gone(pid: u32, bundle: &Path) -> std::io::Result<()> {
    use std::os::unix::process::CommandExt;
    use std::process::Stdio;
    Command::new("/bin/sh")
        .args(["-c", r#"while kill -0 "$1" 2>/dev/null; do sleep 0.2; done; exec /usr/bin/open "$2""#, "sh"])
        .arg(pid.to_string())
        .arg(bundle)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .process_group(0)
        .spawn()
        .map(|_| ())
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
