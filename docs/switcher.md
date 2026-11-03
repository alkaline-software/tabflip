# Switcher

About `crates/tabflip-core/` (`shortcut.rs`, `label.rs`, `order.rs`, `switcher.rs`, `look.rs`, `config.rs`, `update.rs`) and `crates/tabflip-app/src/controller.rs`: what a round of the switcher is, which window comes first, what a row says, and where each piece runs.

## Rules

- **The core decides, the app carries out.** Every rule about which window comes first, what a row says and what a key does is a pure function or a state machine in `tabflip-core`, with tests. The app reads the Mac, hands the core plain data, and does the `Effect` it gets back. A change of behavior starts as a core test.
- **The list never waits for a look.** A round lists the windows last looked at at once, those still on screen or put away, and replaces them when a fresh look comes back from the background executor. An app slow to answer never keeps the list from showing.
- **The window selected stays selected** when a fresh look replaces the rows (`Switcher::keep`).
- **Settings are read at launch and written on each change** (`Config::save`); a field that does not parse falls back to its default, never the whole file. `TABFLIP_HOME` moves the folder, for tests and probes.

## A round

1. **The shortcut's key is pressed** with its modifiers held. The tap swallows it and sends `Key::Trigger`; the switcher begins a round (`Effect::Begin`) with a number of its own.
2. **The windows are listed**: the known ones at once, then a fresh look (`Controller::look`: the apps, the stack, each app's windows, the window in front). Each answer is tagged with its round, and one from an older round is dropped.
3. **The list shows** once both the appear delay (120 ms by default, `appear_delay_ms`) has passed and the windows are in (`Effect::Show`). A quick press and release picks before the delay, and the list never shows.
4. **Tab, ⇧Tab, the arrows and the pointer** move the selection (`Effect::Redraw`). Steps taken before the windows are in are counted from the first window and kept.
5. **Letting go of the modifiers, or Return,** picks the selected window (`Effect::Focus`): it is noted as just used and brought to the front off the main thread (`docs/platform.md`, Focus). A click on a row picks that row. Escape closes with nothing changed.

Between rounds, each time another app comes forward, the windows are looked at again (`Controller::refresh`), so the next round lists them at once. A look asked for while one is under way is taken when that one is back.

## Which window comes first

`order::order`: the windows on screen in the order the system stacks them, front to back, so the one in front is first and the one used before it second; then those not on screen (minimized, or their app hidden), most recently used first (`Recent`, the last 256 window numbers); then the rest in the order the apps listed them. A window an app lists twice is one row.

## What a row says

`label::short_label` cuts a title at ` - `, ` — `, ` – `, ` | `, ` · `, ` • ` after removing unread counts and unsaved marks, and drops the app's own name and anything after it (Chrome's profile). Apps in `LAST` (browsers, VS Code-family editors) take the last part, others the first. Two windows of one app that would read the same get a second part each (`labels`). A new app that reads wrong gets a test in `label.rs` and, if needed, an entry in `LAST`.

## The shortcut

`Shortcut` is the held modifiers and a Mac key code (US layout positions, the numbers the tap reads), kept in the config as text (`cmd-tab`) and shown as `⌘ Tab`. ⇧ with the key moves up; ` with the shortcut's modifiers moves up too, since ⌘` is already in the hand. A shortcut with no ⌘, ⌥ or ⌃, with ⇧ held, or on one of the switcher's own keys (Esc, Return, the arrows) is refused with the reason (`Invalid`), which Settings shows under the field.

## The look

`look.rs` holds the list's size (Small, Medium, Large, a scale on every measure: 1, 1.25, 1.5) and the list's accent (`highlight` in the code and the config): seven colors of one family by name (sapphire first), each with a light and a dark value, or a hex color of the person's own, with `dark_text_on` deciding when text on it turns dark.

## The update

`update.rs` asks GitHub's `releases/latest` redirect for the newest published tag through `curl` (the app carries no HTTPS client), fetches that release's disk image under its stable name, and swaps the bundle in place (`docs/release.md`, The pieces a release carries). A check runs off the main thread, once a day while Check automatically is on, and at a press.
