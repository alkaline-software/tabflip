# AGENTS.md

## What this is

**tabflip** is a macOS window switcher (Rust + GPUI, `crates/`). Hold the
shortcut (⌘Tab by default) and a vertical list of every window shows, each
row an app icon, a word or two for the window, and the app's name. Let go
and that exact window comes to the front. That is all it does. It is a
stand-alone app with no code or runtime tie to any other project.

## The standard

tabflip is an Alkaline Software product, held to the family's standard:
minimal, calm, intuitive, finished in every state. `docs/design.md` is
that standard in tabflip's own terms (its lavender accent, its icon, its
window, its list, the menu bar item, motion, languages) and every
change a person can see meets it unasked. Read it before touching
anything a person sees: a row, a control, a colour, the list, the icon,
the menu. Three rules from it hold in every session:

- **Nothing jumps.** What appears, goes, opens, closes, grows or moves
  does so over a moment, with the pattern its neighbours use.
- **Every word a person sees goes through `i18n::t`** (or `tf`) and is
  in `locales/words.json` in every language; `cargo test -p tabflip-app`
  fails otherwise.
- **Look before saying it is done:** at all the change shows, hides or
  moves, in both appearances, in English and in French or German
  (`docs/design.md`, Checking).

## Layout

```
docs/design.md  the standard: palette, type, icons, the app icon, the list, the window, the menu bar, motion, languages, probing
crates/tabflip-core/src/
  shortcut.rs   held modifiers + a Mac key code; config text "cmd-tab"; display "⌘ Tab"
  label.rs      a window title → "Gmail"; twins in one app get a second part
  order.rs      on-screen stacking order, then most recently used
  switcher.rs   the states a round goes through, and the effects asked of the app
  look.rs       list size (a scale on every measure) and the highlight: presets, hex, text on it
  update.rs     the newest published release (GitHub's releases/latest redirect, via curl)
  config.rs     ~/Library/Application Support/tabflip/config.json (TABFLIP_HOME overrides)
crates/tabflip-app/src/
  main.rs       start-up, the menus, the probes (TABFLIP_SHOT…), restoring ⌘Tab on exit
  controller.rs keys in, core effects out, carried out on the Mac; the settings' changes
  panel.rs      the list's window and its drawing
  settings.rs   the Settings window: rail of sections, groups of rows on cards, the language's list
  ui.rs         the window's small parts: the pill button, key caps, a page coming in
  i18n.rs       the window's words in the person's language (locales/words.json)
  theme.rs      the Settings palette (themes/tabflip.json) with the lavender painted in
  assets.rs     our icons (assets/icons, Tabler) over gpui-component's
  sys/          every macOS call: ax (Accessibility FFI), windows, focus, tap,
                private (SkyLight), status_item (the menu bar item and its menu),
                and window placement, languages and login in mod.rs
scripts/icon/   icon.html draws the app icon and the menu bar's image; make-icon.sh renders them
```

## The one idea

**The core decides, the app carries out.** Every rule about which window
comes first, what a row says, and what a key does is a pure function or
state machine in `tabflip-core` with tests. The app reads the Mac, hands
the core plain data, and does the `Effect` it gets back. A behaviour change
starts as a core test.

## Rules

- **The system's ⌘Tab is always given back.** `CGSSetSymbolicHotKeyEnabled`
  outlives the process. It is switched back on first thing in `main`
  (before anything can fail), at quit, on SIGHUP/SIGINT/SIGTERM and in
  the panic hook. A kill -9 is covered by the next launch. It is only
  ever turned off while the shortcut is exactly ⌘Tab.
- **One tabflip runs at a time.** A tabflip opened while another runs,
  the release or the build, asks it to quit and makes it a few seconds
  on (`sys::end_other_tabflips`): two would each take the shortcut. The
  one quitting gives the system its ⌘Tab back, so the one staying
  applies its shortcut again once the other is gone. A probe copy, with
  its own `TABFLIP_HOME`, ends none.
- **The event tap callback never does slow work.** It matches the event,
  sends a `tap::Key` down a channel and returns. No Accessibility call, no
  allocation beyond the send. A slow tap gets disabled by the system
  (`DISABLED_BY_TIMEOUT`), and the callback re-enables it.
- **The tap holds keys back only while it must.** When closed, it swallows
  the trigger alone; when open (`OPEN`), every key down and up, so the app
  in front never sees half a keystroke. Modifiers always pass.
- **Accessibility is the one permission.** Titles come from AX, stacking
  order and window numbers from `CGWindowListCopyWindowInfo` (which needs no
  Screen Recording for those fields). Do not read `kCGWindowName`; it would
  bring in Screen Recording.
- **Every AX app element gets a 0.25 s messaging timeout** (`Element::app`).
  One hung app must not stall the list. Listing runs on the background
  executor.
- **Private calls are looked up by name** (`sys/private.rs`, `dlsym`), so a
  macOS that removes one leaves that feature missing, not the app unable to
  launch. `_AXUIElementGetWindow` is the one linked directly; every window
  manager on the Mac depends on it.
- **Focus goes through SkyLight first.** On macOS 14+ a background app's
  `activateWithOptions` can be declined by the app in front (cooperative
  activation). `_SLPSSetFrontProcessWithOptions` plus the two make-key event
  records (AltTab's method) is reliable; the public call is the fallback.
- **The panel window is opened once and put away with `orderOut`.** It fades
  out over the picked window and is hidden only after a clear frame has been
  drawn, so the next round never flashes the last one's rows. A round
  number keeps a late put-away from hiding a newer showing.
- **Motion:** nothing jumps. The panel fades and drifts in, the
  highlight slides between rows, and both ease out (`ease`, a cubic).
- **Only the current Space** (plus minimized and hidden windows) in v1. A
  window neither on screen nor put away is on another Space and is left out
  in `windows::list`. All-Spaces needs the private CGS space APIs; see
  AltTab for how.

## Menu bar and Dock

- tabflip is a menu bar app: `LSUIElement` in the bundle, and
  `sys::stay_in_menu_bar` at launch, because gpui makes every app a
  regular one as it starts, over the bundle's own say.
- While Settings are open it is a regular app (`sys::in_dock(true)` in
  `settings::show`), in the Dock and the app switcher with its own menu
  (`app_menus`), so the window can be found again once another app covers
  it. `settings::watch_close` puts it back on `on_window_closed`, never
  `on_window_should_close`: ⌘W closes the window from code
  (`remove_window`), which never asks.
- The item is the icon's stack as a template image; a left click opens
  Settings and a right click the system's own menu of the sections with
  the rail's icons (`sys::status_item`). General can swap the clicks and
  hide the item. Opening tabflip again from Finder or Spotlight shows
  Settings (`on_reopen`).
- Accessibility is asked by the person's press on Allow, never unasked
  at launch; the list is watched (`sys::trust_changes`) and the tap
  starts the moment it is given. **Whether it is given is whether the
  tap can be made**, never `AXIsProcessTrusted` alone: a running process
  reads that once, and after a grant it went on saying no until the app
  was opened again (v0.2.0). A tap is tried only while that flag says
  yes or after Allow was pressed in this launch, since a tap made
  unasked has the system ask (`Controller::listen`, `listen_soon`).

## Settings look

`docs/design.md` has every measure, colour and pattern. In short: 780
by 560, a rail of sections at the left, groups of rows on cards
(`group`, `line`), a switch for on or off, `segmented` for a choice
among a few, `pick` for the language's list, `ui::pill` for a button,
`ui::key_caps` for a key. The accent is lavender (`theme::LAVENDER`);
the list keeps its own colours and the person's highlight.

## Labels

`label::short_label` cuts a title at ` - `, ` — `, ` – `, ` | `, ` · `, ` • `
after removing unread counts and unsaved marks, and drops the app's own
name and anything after it (Chrome's profile). Apps in `LAST` (browsers,
VS Code-family editors) take the last part, others the first. A new app
that reads wrong gets a test in `label.rs` and, if needed, an entry in
`LAST`.

## Probing

- `target/debug/tabflip --list` prints the windows in switcher order with
  their labels. The terminal running it needs Accessibility.
- `TABFLIP_SHOT=/path/panel.png target/debug/tabflip` shows the panel with
  sample rows, writes a picture of it and quits.
  `TABFLIP_SHOT_SETTINGS=/path/settings.png` does the same for Settings
  (`TABFLIP_SETTINGS=customize` picks the section, `TABFLIP_GO` plays
  steps first, `TABFLIP_LANG` and `TABFLIP_APPEARANCE` stand in for the
  system's; `docs/design.md`, Checking). Give a probe a `TABFLIP_HOME`
  of its own. None needs any permission: a process may capture its own
  windows.
- `TABFLIP_DEMO_LOOP=/dir target/debug/tabflip` shows and puts away the
  panel four times, as rounds do, writing `demo0.png`… to `/dir`. A bug
  that only shows from the second round on (a resize gpui never heard of)
  shows here and not in `TABFLIP_SHOT`.
- Debug builds log what each round did to `~/Library/Logs/tabflip-dev.log`.

## Dev builds and signing

`scripts/dev-app.sh` builds `target/debug/dev.noindex/tabflip.app` and
signs it with the first identity it finds (see the script). macOS keeps the
Accessibility grant by signer: an ad-hoc build must be allowed again after
every build. `scripts/make-dev-cert.sh` makes a self-signed "tabflip dev"
certificate once, so it doesn't have to be.

Releases are signed and notarized by whoever holds the Developer ID (at
present Pingfan). WORKFLOW.md has every step; `scripts/release-check.sh`
holds a commit to what the tag will be held to, and
`scripts/release-clean.sh` ends a release by deleting everything
compiled.

## Dependencies

gpui is a `git` dependency with no `rev`; the pin is in `Cargo.lock`, at a
fixed zed commit (`cargo update -p gpui --precise <rev>` moves
it). A `rev` in the manifest makes a second gpui that gpui-component won't
build against. gpui-component is upstream, pinned by `rev` in the app's manifest.
