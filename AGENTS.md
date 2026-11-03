# AGENTS.md

## What this is

**Tabflip** is a macOS window switcher (Rust and GPUI, `crates/`). Hold the shortcut (⌘Tab by default) and a vertical list of every window shows, each row an app icon, a word or two for the window, and the app's name. Let go and that exact window comes to the front. That is all it does. It is a stand-alone app with no code or runtime tie to any other project.

The one idea: **the core decides, the app carries out.** Every rule about which window comes first, what a row says and what a key does is a pure function or a state machine in `tabflip-core`, with tests. The app reads the Mac, hands the core plain data, and does the `Effect` it gets back, so the behavior is tested without a Mac and a change of behavior starts as a core test (`docs/switcher.md`).

## The standard

Tabflip is an Alkaline Software product, held to the studio's standard: minimal, calm, intuitive, finished in every state. `docs/design.md` is that standard in Tabflip's own terms (its sapphire accent, its icon, its window, its list, the menu bar item, motion, languages), and every change a person can see meets it unasked. Read it before touching anything a person sees. Three of its rules hold in every session:

- **Nothing jumps.** What appears, goes, opens, closes, grows or moves does so over a moment, with the pattern its neighbors use.
- **Every word a person sees goes through `i18n::t`** (or `tf`) and is in `locales/words.json` in every language; `cargo test -p tabflip-app` fails otherwise.
- **Look before saying it is done**: at all the change shows, hides or moves, in both appearances, in English and in French or German (`docs/probing.md`).

## Rules that hold everywhere

- **The system's ⌘Tab is always given back**: on at launch before anything can fail, at quit, on a signal and in the panic hook (`docs/platform.md`).
- **One Tabflip runs at a time**; a probe copy with its own `TABFLIP_HOME` ends none (`docs/platform.md`).
- **The key tap runs on a thread of its own, and its callback only matches and forwards** (`docs/platform.md`).
- **The main thread never waits on another app**: every Accessibility question runs on the background executor, each with a 0.25 s timeout (`docs/platform.md`).
- **Nothing slow is asked while a frame is built**: what the system has is found once, at launch (`docs/design.md`, Frames).
- **Accessibility is the one permission**: never read `kCGWindowName` (`docs/platform.md`).
- **Whether Accessibility is given is whether the tap can be made**, never `AXIsProcessTrusted` alone, and Allow first forgets the system's old entry (`docs/platform.md`, Accessibility).
- **Private calls are looked up by name** (`sys/private.rs`), so a macOS without one loses a feature, not the app (`docs/platform.md`).
- **The list never waits for a look**: the windows last seen show at once and a fresh look replaces them (`docs/switcher.md`).
- **The panel window is opened once and put away with `orderOut`** after a clear frame (`docs/platform.md`, The list's window).
- **Settings close by `on_window_closed`**, never `on_window_should_close`, which ⌘W never asks (`docs/platform.md`).
- **One copy of gpui**: the pin is in `Cargo.lock`, never a `rev` in a manifest. The toolkit is vendored, and every change to it is marked `(Studio addition.)` and listed in `vendor/gpui-component/UPSTREAM.md`. Build from the root, never inside `vendor/` (Dependencies, below).
- **Other apps are asked only when a round begins**, and nothing runs at rest (`docs/platform.md`).

## Before you edit, read

| Touching | Read first |
|---|---|
| Anything a person sees: a row, a control, a color, the list, Settings, the icon, the menu bar item, motion, a language | `docs/design.md` |
| The shortcut, the order, the labels, a round of the switcher, the config, the update (`tabflip-core`, `controller.rs`) | `docs/switcher.md` |
| A call to macOS: the tap, Accessibility, the window list, focus, SkyLight, the panel's window, the menu bar and the Dock, login, languages, the dev build's signing (`sys/`) | `docs/platform.md` |
| Looking at a state from a script, measuring a speed | `docs/probing.md` |
| A release, a script under `scripts/release-*`, CI | `docs/release.md` |

A subagent handed work on a file is told to read its doc too.

## Layout

```
AGENTS.md         this file
CHANGELOG.md      what each release changed, for the person installing it
docs/             design.md, switcher.md, platform.md, probing.md, release.md
crates/tabflip-core/src/
  shortcut.rs     held modifiers and a Mac key code; config text "cmd-tab"; display "⌘ Tab"
  label.rs        a window title to "Gmail"; twins in one app get a second part
  order.rs        on-screen stacking order, then most recently used
  switcher.rs     the states a round goes through, and the effects asked of the app
  look.rs         the list's size and accent: presets, hex, text on it
  update.rs       the newest published release (GitHub's releases/latest redirect, via curl)
  config.rs       ~/Library/Application Support/tabflip/config.json (TABFLIP_HOME overrides)
crates/tabflip-app/src/
  main.rs         start-up, the menus, the probes (TABFLIP_SHOT…), restoring ⌘Tab on exit
  controller.rs   keys in, core effects out, carried out on the Mac; the settings' changes
  panel.rs        the list's window and its drawing
  settings.rs     the Settings window: rail, groups of rows on cards, About, the language's list
  ui.rs           the window's small parts: the pill, key caps, a page coming in, the name's face
  i18n.rs         the window's words in the person's language (locales/words.json)
  theme.rs        the Settings palette (themes/tabflip.json) with the sapphire painted in
  assets.rs       our icons (assets/icons, Tabler) over gpui-component's
  sys/            every macOS call: ax, windows, focus, tap, private (SkyLight), status_item, and placement, languages, login and the Dock in mod.rs
scripts/          dev-app.sh, make-dev-cert.sh, make-icon.sh and icon/, dmg/, release-*.sh
vendor/           gpui-component and block, each with an UPSTREAM.md
```

## Running it

```
scripts/dev-app.sh              build, bundle, sign and open the dev app
target/debug/tabflip --list     the windows as the switcher sees them (needs Accessibility)
```

One dev build at a time: `scripts/dev-app.sh` quits a running copy, the installed one included, before it opens the build. The dev bundle is signed so Accessibility survives a rebuild (`docs/platform.md`, Dev builds and signing). A release ends with `scripts/release-clean.sh`, which deletes everything compiled (`docs/release.md`, step 8).

## Writing things down

- A fact goes where its code is: how a part works and its traps in that part's doc, under `## Rules` when breaking it loses data or brings back a bug. This file gets a line only for a rule every session follows, or a row in its table. Keep it under 300 lines.
- What is so now and why, not how it got there. An approach tried and dropped gets one sentence, only where someone might try it again.
- No hard wraps: a paragraph or a list item is one line. American spelling. No em dashes. Name the place, not a line number (`docs/design.md`, Motion; `ui::wordmark`).
- A lesson learned goes in these docs, never in an agent's private memory.
- The changelog is written for the person installing the app (`docs/release.md`, step 2).

## Testing

```
cargo test -p tabflip-core -p tabflip-app
```

The core's tests need no Mac and no window. The app's test holds the languages' table to the phrases the code asks for. No test reads or writes the person's settings: the config's tests parse text, and every probe gives itself a `TABFLIP_HOME`.

## Dependencies

gpui is a `git` dependency with no `rev`; the pin is in `Cargo.lock`, at a fixed zed commit (`cargo update -p gpui --precise <rev>` moves it). A `rev` in the manifest makes a second gpui that gpui-component will not build against. gpui-component is pinned by `rev` in the app's manifest and built from `vendor/gpui-component/`, the studio's copy of that revision, through `[patch]` in the root `Cargo.toml`, so the manifest still names where the code came from. Each change made to it is marked `(Studio addition.)` in the source and numbered in its `UPSTREAM.md`, or it is lost at the next move to a newer upstream. `vendor/block` is the Objective-C blocks crate under gpui's Mac backend, changed so a future Rust still compiles it. The vendored crates are workspaces of their own: cargo run inside `vendor/` resolves another lock and writes a second `target/`.
