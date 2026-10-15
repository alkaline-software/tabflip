//! A word or two for a window, from its title: "Gmail" for
//! "Inbox (3) - john@gmail.com - Gmail", "Facebook" for "(2) Facebook".
//!
//! A title is cut into segments at the separators apps put between its
//! parts. Browsers end with the site's name, so they take the last segment;
//! most other apps start with what the window is, so they take the first.
//! A few apps are known to do otherwise (`LAST`). Two windows of one app
//! that would read the same get a second segment each (`labels`).

/// Apps whose title ends with what tells their windows apart.
const LAST: &[&str] = &[
    // Browsers: page — site.
    "com.google.Chrome",
    "com.google.Chrome.beta",
    "com.google.Chrome.canary",
    "org.chromium.Chromium",
    "com.apple.Safari",
    "com.apple.SafariTechnologyPreview",
    "company.thebrowser.Browser",
    "org.mozilla.firefox",
    "org.mozilla.firefoxdeveloperedition",
    "com.microsoft.edgemac",
    "com.brave.Browser",
    "com.vivaldi.Vivaldi",
    "com.operasoftware.Opera",
    "app.zen-browser.zen",
    // Editors that write file — folder.
    "com.microsoft.VSCode",
    "com.microsoft.VSCodeInsiders",
    "com.vscodium",
    "com.todesktop.230313mzl4w4u92",
    "com.exafunction.windsurf",
];

/// Browsers, whose single-segment titles are often "site: what it is".
const BROWSERS: usize = 14;

/// Names an app may append to its own titles, besides its display name.
const APP_SUFFIXES: &[&str] = &["Google Chrome", "Mozilla Firefox", "Microsoft Edge", "Brave", "Safari", "Arc", "Visual Studio Code", "Cursor"];

const SEPARATORS: &[&str] = &[" - ", " — ", " – ", " | ", " · ", " • "];

const MAX_CHARS: usize = 24;
const MAX_WORDS: usize = 3;

/// The row's label for one window.
pub fn short_label(app_name: &str, bundle_id: &str, title: &str) -> String {
    let parts = segments(app_name, title);
    match pick(bundle_id, &parts) {
        Some(i) => fit(&single(bundle_id, &parts, i)),
        None => fit(app_name),
    }
}

/// One window: what the app calls it, its bundle id and its title.
#[derive(Debug, Clone)]
pub struct Titled<'a> {
    pub app_name: &'a str,
    pub bundle_id: &'a str,
    pub title: &'a str,
}

/// The labels for a list of windows. Where two windows of the same app
/// would read the same, each gets the next most telling segment of its
/// title too ("Gmail · Inbox", "Gmail · Sent"), so the rows differ.
pub fn labels(windows: &[Titled]) -> Vec<String> {
    let first: Vec<String> = windows.iter().map(|w| short_label(w.app_name, w.bundle_id, w.title)).collect();
    let mut out = first.clone();
    for i in 0..windows.len() {
        let twin = (0..windows.len()).any(|j| j != i && windows[j].bundle_id == windows[i].bundle_id && first[j] == first[i]);
        if !twin {
            continue;
        }
        let w = &windows[i];
        let parts = segments(w.app_name, w.title);
        let Some(chosen) = pick(w.bundle_id, &parts) else { continue };
        // The other segment closest to the chosen one's far end: the page
        // for a browser, the file for an editor, the next part otherwise.
        let other = if last_wins(w.bundle_id) { (0..chosen).next() } else { (chosen + 1..parts.len()).next() };
        if let Some(o) = other {
            out[i] = format!("{} · {}", fit(&parts[chosen]), fit_to(&parts[o], MAX_CHARS - 6));
        }
    }
    out
}

/// The title's parts, cleaned: no unread counts or unsaved marks, and
/// nothing from the app's own name on.
pub fn segments(app_name: &str, title: &str) -> Vec<String> {
    let mut parts: Vec<String> = vec![clean(title)];
    for sep in SEPARATORS {
        parts = parts.iter().flat_map(|p| p.split(sep).map(str::to_string).collect::<Vec<_>>()).collect();
    }
    let mut parts: Vec<String> = parts.into_iter().map(|p| clean(&p)).filter(|p| !p.is_empty()).collect();
    // "Page - Google Chrome - John (Work)": the app's name and the profile
    // after it say nothing about this window.
    if let Some(at) = parts.iter().skip(1).position(|p| is_app_name(app_name, p)) {
        parts.truncate(at + 1);
    }
    if parts.len() == 1 && is_app_name(app_name, &parts[0]) {
        parts.clear();
    }
    parts
}

fn is_app_name(app_name: &str, part: &str) -> bool {
    part.eq_ignore_ascii_case(app_name) || APP_SUFFIXES.iter().any(|s| part.eq_ignore_ascii_case(s))
}

fn last_wins(bundle_id: &str) -> bool {
    LAST.contains(&bundle_id)
}

fn is_browser(bundle_id: &str) -> bool {
    LAST[..BROWSERS].contains(&bundle_id)
}

fn pick(bundle_id: &str, parts: &[String]) -> Option<usize> {
    if parts.is_empty() {
        return None;
    }
    Some(if last_wins(bundle_id) { parts.len() - 1 } else { 0 })
}

/// A browser tab with a one-part title like "owner/repo: what it is"
/// is named by what comes before the colon.
fn single(bundle_id: &str, parts: &[String], i: usize) -> String {
    let part = &parts[i];
    if parts.len() == 1 && is_browser(bundle_id) && part.chars().count() > MAX_CHARS {
        if let Some((head, _)) = part.split_once(": ") {
            return head.to_string();
        }
    }
    part.clone()
}

/// Without unread counts ("(3) ", " (12)", "(99+)"), unsaved marks ("● ")
/// and the spaces around them.
fn clean(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(open) = rest.find('(') {
        let after = &rest[open + 1..];
        match after.find(')') {
            Some(close) if is_count(&after[..close]) => {
                out.push_str(&rest[..open]);
                rest = &after[close + 1..];
            }
            _ => {
                out.push_str(&rest[..=open]);
                rest = after;
            }
        }
    }
    out.push_str(rest);
    let trimmed = out.trim_matches(|c: char| c.is_whitespace() || matches!(c, '●' | '•' | '*' | '⦁'));
    trimmed.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn is_count(inner: &str) -> bool {
    let digits = inner.strip_suffix('+').unwrap_or(inner);
    !digits.is_empty() && digits.chars().all(|c| c.is_ascii_digit())
}

/// At most three words and `MAX_CHARS` characters, with "…" where cut.
fn fit(text: &str) -> String {
    fit_to(text, MAX_CHARS)
}

fn fit_to(text: &str, max_chars: usize) -> String {
    let words: Vec<&str> = text.split_whitespace().collect();
    let mut out = words.iter().take(MAX_WORDS).copied().collect::<Vec<_>>().join(" ");
    let mut cut = words.len() > MAX_WORDS;
    if out.chars().count() > max_chars {
        out = out.chars().take(max_chars.saturating_sub(1)).collect::<String>().trim_end().to_string();
        cut = true;
    }
    if cut {
        out = out.trim_end_matches(|c: char| c.is_ascii_punctuation() || c.is_whitespace()).to_string();
        out.push('…');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const CHROME: &str = "com.google.Chrome";

    fn chrome(title: &str) -> String {
        short_label("Google Chrome", CHROME, title)
    }

    #[test]
    fn browsers_take_the_site() {
        assert_eq!(chrome("Inbox (3) - john@gmail.com - Gmail"), "Gmail");
        assert_eq!(chrome("(2) Facebook"), "Facebook");
        assert_eq!(chrome("Facebook"), "Facebook");
        assert_eq!(chrome("(12) Rust in 100 Seconds - YouTube"), "YouTube");
        assert_eq!(chrome("Inbox - Gmail - Google Chrome - John (Work)"), "Gmail");
        assert_eq!(short_label("Safari", "com.apple.Safari", "Pull requests · alkaline-software/emaki"), "alkaline-software/emaki");
        assert_eq!(chrome("alkaline-software/emaki: Every coding-agent session on your machine"), "alkaline-software/emaki");
    }

    #[test]
    fn other_apps_take_the_first_part() {
        assert_eq!(short_label("Finder", "com.apple.finder", "Downloads"), "Downloads");
        assert_eq!(short_label("Terminal", "com.apple.Terminal", "jhelvy — -zsh — 80×24"), "jhelvy");
        assert_eq!(short_label("Zed", "dev.zed.Zed", "tabflip — main.rs"), "tabflip");
        assert_eq!(short_label("Slack", "com.tinyspeck.slackmacgap", "general (Channel) - Alkaline - Slack"), "general (Channel)");
    }

    #[test]
    fn editors_that_end_with_the_folder() {
        assert_eq!(short_label("Code", "com.microsoft.VSCode", "● main.rs — tabflip"), "tabflip");
        assert_eq!(short_label("Code", "com.microsoft.VSCode", "main.rs — tabflip — Visual Studio Code"), "tabflip");
    }

    #[test]
    fn no_title_is_the_app() {
        assert_eq!(short_label("Notes", "com.apple.Notes", ""), "Notes");
        assert_eq!(short_label("Notes", "com.apple.Notes", "Notes"), "Notes");
        assert_eq!(short_label("Calculator", "com.apple.calculator", "  "), "Calculator");
    }

    #[test]
    fn long_titles_are_cut() {
        assert_eq!(short_label("Preview", "com.apple.Preview", "Quarterly results for the whole company.pdf"), "Quarterly results for…");
        assert_eq!(short_label("Preview", "com.apple.Preview", "Supercalifragilisticexpialidocious_draft.pdf"), "Supercalifragilisticexp…");
    }

    #[test]
    fn counts_and_marks_go() {
        assert_eq!(clean("(99+) Inbox"), "Inbox");
        assert_eq!(clean("Inbox (3)"), "Inbox");
        assert_eq!(clean("Report (final)"), "Report (final)");
        assert_eq!(clean("● notes.md"), "notes.md");
    }

    #[test]
    fn twins_get_a_second_part() {
        let rows = [
            Titled { app_name: "Google Chrome", bundle_id: CHROME, title: "Inbox (3) - john@gmail.com - Gmail" },
            Titled { app_name: "Google Chrome", bundle_id: CHROME, title: "Sent Mail - john@gmail.com - Gmail" },
            Titled { app_name: "Google Chrome", bundle_id: CHROME, title: "(2) Facebook" },
            Titled { app_name: "Finder", bundle_id: "com.apple.finder", title: "Gmail" },
        ];
        assert_eq!(labels(&rows), ["Gmail · Inbox", "Gmail · Sent Mail", "Facebook", "Gmail"]);
    }
}
