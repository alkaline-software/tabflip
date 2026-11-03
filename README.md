# Tabflip

**Switch between windows, not apps.**

The Mac's ⌘Tab moves between apps. With two Chrome windows open, one on Gmail and one on Facebook, it can't take you to either one. Tabflip can: hold ⌘ and press Tab, and a small list shows every window, each with its app's icon and a word or two for what is in it.

```
 ◉  Gmail                Google Chrome
 ◉  Facebook             Google Chrome      ← Tab moves down, ⇧Tab up
 ▣  tabflip              Zed
 📁 Downloads            Finder
```

Let go of ⌘ and that window comes to the front. A quick ⌘Tab goes straight to the window you used last, with no list at all. ⌘⇧Tab or ⌘\` moves up; Esc cancels; the arrows and the pointer work too.

## Install

Download the disk image from the [latest release](https://github.com/alkaline-software/tabflip/releases/latest) and drag Tabflip to Applications: `tabflip-mac-arm64.dmg` for Apple silicon, `tabflip-mac-x64.dmg` for an Intel Mac.

Tabflip lives in the menu bar. Its Settings ask for **Accessibility**, which it needs to read window titles, bring a window forward, and see the shortcut in every app. That is the only permission it asks for: no Screen Recording, and nothing leaves the Mac. In Settings you can pick another shortcut, the list's size and accent color, and the language; Tabflip updates itself from About at a press.

## Building it

Rust, with [GPUI](https://www.gpui.rs) and [gpui-component](https://github.com/longbridge/gpui-component), and [objc2](https://github.com/madsmtm/objc2) for the macOS calls.

```
cargo test -p tabflip-core -p tabflip-app
scripts/dev-app.sh
```

`AGENTS.md` explains the code and where each part's doc is; `docs/release.md` is how a release is cut.

MIT.
