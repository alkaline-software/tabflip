# Changelog

## v0.1.2

The list's size and highlight colour can be chosen in Settings.

- **List size: Small, Medium or Large.** Small is the list as it was.
  Medium makes everything in it a quarter bigger and Large half again:
  rows, icons, text, spacing and the list's width grow together. A
  change applies from the next ⌘Tab.
- **Highlight colour.** The selected row can be any of eight of the
  Mac's own colours (Blue, Purple, Pink, Red, Orange, Green, Teal,
  Graphite), which follow light and dark mode as the system's do, or a
  colour of your own typed as a hex code (`#ff8800` or `#f80`). A code
  is kept as soon as it is complete; one that is not a colour is marked
  "Not a colour" and not kept.
- **Readable on any colour.** Text on the highlight turns dark on a pale
  colour (yellow, light grey) and stays white on the rest.
- **A preview in Settings.** A sample row under the colours shows the
  chosen size and highlight.
- Settings is taller to fit the new rows.

Both are stored in `config.json` as `"size"` (`"small"`, `"medium"`,
`"large"`) and `"highlight"` (a colour's name or `"#rrggbb"`). A value
that is not one falls back to Small and Blue without touching the other
settings.

For developers:

- `tabflip-core/src/look.rs` holds the sizes (`Size::scale`), the
  presets, hex parsing and the dark-or-white text rule, with tests. The
  panel multiplies every measure by the scale it was shown at.

## v0.1.1

Fixes the list showing only one row.

- **The list shows every window.** From the second ⌘Tab on, the list
  was drawn at the height it was first opened with, one row, so only
  one window could be seen at a time. The window grew to fit every row
  but its contents did not: gpui, which draws them, was told of the new
  size through an update of the window already being updated, and the
  notice was dropped. The panel now tells gpui its new size itself.
- **The list comes up at once.** The windows last looked at are shown
  as soon as the list appears, and a fresh look replaces them a moment
  later, keeping the same window selected. They are looked at again
  each time another app comes forward.
- **A slow app can no longer hold up the list.** Each app is asked for
  its windows on a thread of its own, and every Accessibility question
  to any app now gives up after a quarter of a second; before, only the
  first question to each app did, and questions about its windows could
  wait six seconds.

For developers:

- Dev builds log each round (keys, how long listing took, the panel
  shown) to `~/Library/Logs/tabflip-dev.log`. Release builds log nothing.
- `TABFLIP_DEMO_LOOP=<dir>` shows and puts away the panel four times and
  saves a picture of each round, which catches bugs that only appear
  after the first round.
- `scripts/dev-app.sh` names the dev copy "tabflip dev" in Accessibility
  and quits an installed tabflip before opening it, so two copies never
  both take the shortcut.

## v0.1.0

The first version: switch between windows, not apps.

- Hold ⌘Tab for a list of every window on this Space, plus minimized and
  hidden ones: the app's icon, a word or two for the window ("Gmail",
  "Facebook"), and the app's name.
- Tab and ⇧Tab, the arrows, or the pointer pick; letting go brings that
  window forward. A quick ⌘Tab goes to the window used last without
  showing the list. Esc cancels.
- Any shortcut of a modifier and a key, set in Settings. ⌘Tab replaces
  the Mac's own while tabflip runs, and is given back at quit.
- Open at login. A menu bar item; no Dock icon.
