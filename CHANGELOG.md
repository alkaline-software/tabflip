# Changelog

## v0.2.3

Allow works for someone who had an earlier tabflip.

- **Allow forgets the old entry first.** macOS binds an Accessibility
  grant to the app's signature. An entry made for an earlier copy of
  tabflip, signed otherwise, showed as on in System Settings and
  counted for nothing, and switching it off and on changed nothing;
  only removing it with the minus button and granting again worked.
  Allow now does that itself: it forgets tabflip's entry and asks the
  system's question afresh, so one press and one switch is all it takes
  whether or not an older tabflip was ever allowed.

## v0.2.2

tabflip updates itself, and the disk image is laid out.

- **Update to x.** The button in Updates fetches the newer version,
  puts it in place and opens it again as the new tabflip, saying how far
  the download is as it goes. Nothing is fetched or installed without
  the press.
- **The disk image** opens on "Drag tabflip to Applications", with the
  app and the Applications folder side by side under a lavender arrow.

For developers:

- `update::download` and `update::install` in the core, `tabflip
  --try-update <bundle>` to try them on a throwaway bundle.
- `scripts/dmg/background.py` draws the background; `release-mac.sh`
  lays the image out through Finder, as Kotoba's does.

## v0.2.1

Accessibility given while tabflip runs is seen at once.

- **A grant is noticed without a relaunch.** v0.2.0 asked the system
  once whether it was trusted and kept the answer, so Allow followed by
  the switch in System Settings left the row on "Open Settings" and
  ⌘Tab with the Mac until tabflip was opened again. The tap is now tried
  itself, for a few seconds after Allow, after each change to
  Accessibility's list, and whenever Settings come back in front, and it
  starts the moment the grant is written.
- **One tabflip at a time.** A tabflip opened while another runs asks
  it to quit, so two never both take the shortcut; the one opened last
  stays.

## v0.2.0

tabflip takes the look of Alkaline Software's family, speaks eight
languages, and gets its own icon.

- **The icon.** A stack of three windows fanned as a hand flips through
  them, the front one lavender, on the studio's cream, drawn in the
  manner of Emaki's and Kotoba's icons. The menu bar item is the same
  stack as a template image the system tints.
- **Lavender.** The accent of Settings is lavender, a hue of the
  family's wheel halfway between Emaki's periwinkle and orchid, in a
  light and a dark shade that read on either page.
- **Languages.** Settings speak English, Chinese (Simplified and
  Traditional), Japanese, Korean, French, German and Spanish, following
  the system by default; a list at the right of General's title picks
  one, which takes at once and is the app's own language in System
  Settings, Language & Region, Applications too. The list of windows is
  untouched: each window's own title, its app's own name.
- **The menu bar item.** A left click opens Settings; a right click
  shows the system's own menu of the sections, each with its icon, and
  Quit. **Swap clicks** in General swaps the two.
- **Accessibility is asked by Allow**, never unasked at launch: the
  first press asks the system's own question, a later one opens Privacy
  & Security. The row says Allowed the moment it is given.
- **Keys on caps.** ⌘ ⌥ ⇧ ⌃ in the shortcut are drawn as icons on their
  caps, level with the letters.
- **About** is the family's title card: "Alkaline Software presents",
  the icon, the name in Optima and the accent, the version, and the
  studio's owners.
- **⌘` goes back.** The shortcut's modifiers with ` move up the list,
  as ⇧ with the shortcut's key does.
- **The list no longer shakes.** Its rows were 2 points taller than
  their scroller (the stack's hairline edge was not counted), so a move
  to the first or last row scrolled the whole list by that much, and
  the next round opened where the last one left off. The highlight now
  slides on gpui's own animation clock at the display's rate (gpui held
  an inactive window, which the list always is, to 30 frames a second),
  over 150 ms with a gentler ease, and a row's words take the
  highlight's colour only as far as the plate covers them.
- **Motion.** The rail's dots and the permission's state come and go
  over a moment; the language's list rises in and fades out; the window
  fades into a new language.

New settings in `config.json`: `"menu_bar_swap"` (default `false`) and
`"language"` (default `"system"`).

For developers:

- `docs/design.md` is tabflip's design standard, written from the
  studio's constitution: the palette, the type, the icons, the app icon,
  the list, the window, the menu bar, motion, languages, probing.
- `scripts/icon/icon.html` draws the icon on a canvas with proofs of
  every size and candidates (`VARIANT`); `scripts/make-icon.sh` renders
  every size on its own pixels through WebKit. `draw.swift` is gone.
- `i18n.rs` and `locales/words.json`: every word through `t`/`tf`;
  `cargo test -p tabflip-app` fails on a phrase missing anywhere.
- Probes: `TABFLIP_LANG`, `TABFLIP_APPEARANCE`, and `TABFLIP_GO` steps
  (`dark`, `pick:language`, `untrusted`, `record`, `newer`, …) before a
  `TABFLIP_SHOT_SETTINGS` picture.

## v0.1.3

tabflip stays out of the Dock, and Settings are redrawn in the look
Alkaline Software's other apps share.

- **Out of the Dock.** tabflip was meant to live in the menu bar alone,
  but showed a Dock icon all the time: gpui makes every app a regular one
  as it starts, over the bundle's `LSUIElement`. It now says otherwise at
  launch, and is in the Dock (and ⌘Tab's own list) only while Settings
  are open, so that window can be found again once another app covers
  it. Closing Settings, by its button or ⌘W, takes it out again.
- **Show in menu bar.** A new switch in General hides the menu bar item.
  With it hidden, opening tabflip again from Finder or Spotlight opens
  Settings.
- **Settings, redrawn.** A rail of sections at the left (General,
  Customize, Updates, About) and each section's settings as rows on
  cards, with switches for on and off, a sliding choice for the list's
  size, and the window's top dragged by as a title bar. The palette is
  the one Kotoba and Emaki use, with tabflip's blue as the accent.
  - **General:** the shortcut (shown as keys), Launch at login, Show in
    menu bar, and Accessibility.
  - **Customize:** the list's size, its highlight colour or a hex code of
    your own, and a preview row.
  - **Updates:** the version, Check for updates, and Check
    automatically (once a day, on by default). A newer published release
    is offered as its installer to download; nothing installs by itself.
    Drafts are not seen until they are published.
  - **About:** the app and its version, as an open source package by
    Alkaline Software.
- While open, tabflip has the Mac's usual app menu (Settings…, Hide,
  Quit) and ⌘W closes Settings.

New settings in `config.json`: `"menu_bar_icon"` (default `true`),
`"check_updates"` (default `true`) and `"last_update_check"`.

For developers:

- `tabflip-core/src/update.rs` reads the newest published version from
  the redirect GitHub's `releases/latest` answers with (via `curl`, so the
  app carries no HTTPS client), with tests for the parsing and the
  version order.
- `tabflip-app` gains `theme.rs` (the palette, `themes/tabflip.json`),
  `ui.rs` (pill buttons, key caps), and `assets.rs` (Tabler icons in
  `assets/icons` over gpui-component's set).
- `TABFLIP_SETTINGS=<section>` with `TABFLIP_SHOT_SETTINGS` shoots that
  section.

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
