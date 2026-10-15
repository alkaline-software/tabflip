# AGENTS.md

## What this is

**tabflip** is a macOS window switcher (Rust + GPUI, `crates/`). Hold the
shortcut (⌘Tab by default) and a vertical list of every window shows, each
row an app icon, a word or two for the window, and the app's name. Let go
and that exact window comes to the front. That is all it does. It is built
on emaki's stack (`../emaki`, github.com/alkaline-software/emaki) and
follows its conventions.

## Layout

```
crates/tabflip-core/src/
  shortcut.rs   held modifiers + a Mac key code; config text "cmd-tab"; display "⌘ Tab"
  label.rs      a window title → "Gmail"; twins in one app get a second part
  order.rs      on-screen stacking order, then most recently used
  switcher.rs   the states a round goes through, and the effects asked of the app
  config.rs     ~/Library/Application Support/tabflip/config.json (TABFLIP_HOME overrides)
crates/tabflip-app/src/
  main.rs       start-up, the --list and TABFLIP_SHOT probes, restoring ⌘Tab on exit
  controller.rs keys in, core effects out, carried out on the Mac
  panel.rs      the list's window and its drawing
  settings.rs   the Settings window
  sys/          every macOS call: ax (Accessibility FFI), windows, focus, tap,
                private (SkyLight), status_item, and window placement in mod.rs
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
- **Motion, as in emaki:** nothing jumps. The panel fades and drifts in, the
  highlight slides between rows, and both ease out (`ease`, a cubic).
- **Only the current Space** (plus minimized and hidden windows) in v1. A
  window neither on screen nor put away is on another Space and is left out
  in `windows::list`. All-Spaces needs the private CGS space APIs; see
  AltTab for how.

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
  `TABFLIP_SHOT_SETTINGS=/path/settings.png` does the same for Settings.
  Neither needs any permission: a process may capture its own windows.

## Dev builds and signing

`scripts/dev-app.sh` builds `target/debug/dev.noindex/tabflip.app` and
signs it with the first identity it finds (see the script). macOS keeps the
Accessibility grant by signer: an ad-hoc build must be allowed again after
every build. `scripts/make-dev-cert.sh` makes a self-signed "tabflip dev"
certificate once, so it doesn't have to be.

Releases are signed and notarized by whoever holds the Developer ID (at
present Pingfan, as for emaki). WORKFLOW.md.

## Dependencies

gpui is a `git` dependency with no `rev`; the pin is in `Cargo.lock`, at the
same zed commit emaki uses (`cargo update -p gpui --precise <rev>` moves
it). A `rev` in the manifest makes a second gpui that gpui-component won't
build against. gpui-component is upstream at the revision emaki vendored
from; tabflip uses none of the parts emaki patched.
