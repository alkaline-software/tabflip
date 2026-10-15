# tabflip

**Switch between windows, not apps.**

The Mac's ⌘Tab moves between apps. With two Chrome windows open, one on
Gmail and one on Facebook, it can't take you to either one. tabflip can:
hold ⌘ and press Tab, and a small list shows every window, each with its
app's icon and a word or two for what is in it.

```
 ◉  Gmail                Google Chrome
 ◉  Facebook             Google Chrome      ← Tab moves down, ⇧Tab up
 ▣  tabflip              Zed
 📁 Downloads            Finder
```

Let go of ⌘ and that window comes to the front. A quick ⌘Tab goes straight
to the window you used last, with no list at all. Esc cancels; the arrows
and the pointer work too.

## Install

Download the disk image from the
[latest release](https://github.com/alkaline-software/tabflip/releases/latest)
and drag tabflip to Applications:

```
tabflip-mac-arm64.dmg    Apple silicon
tabflip-mac-x64.dmg      Intel Mac
```

On first launch macOS asks to give tabflip **Accessibility**, in System
Settings → Privacy & Security. It needs it to read window titles, bring a
window forward, and see the shortcut in every app. That is the only
permission it asks for: no Screen Recording, and nothing leaves the Mac.

## Settings

tabflip lives in the menu bar. **Settings…** there sets:

- **Shortcut.** ⌘Tab by default, which turns off the Mac's own ⌘Tab
  while tabflip runs (it comes back when tabflip quits). Click the
  shortcut and press another: ⌥Tab, ⌃Tab, ⌘\`, any modifier and a key.
- **Open at login.**

## How it is built

Rust, with [GPUI](https://www.gpui.rs) (the Zed editor's UI framework) and
[gpui-component](https://github.com/longbridge/gpui-component), and
[objc2](https://github.com/madsmtm/objc2) for the macOS calls.

```
crates/tabflip-core   the shortcut, the labels, the order, the switcher's states; tested anywhere
crates/tabflip-app    the panel, Settings, and every macOS call (src/sys/)
scripts/              the dev bundle, the icon, releases
```

## Development

```
cargo test -p tabflip-core
scripts/dev-app.sh              build, bundle, sign and open the dev app
target/debug/tabflip --list     the windows as the switcher sees them
```

`AGENTS.md` explains the code and the reasoning behind it. `WORKFLOW.md` is
how a release is cut.

MIT.
