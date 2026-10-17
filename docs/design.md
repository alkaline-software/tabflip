# Design

The standard every part of tabflip a person sees is held to: the list
that shows while the shortcut is held, the Settings window and its
controls, the menu bar item and its menu, the app icon. The bar is the
work that wins Awwwards, the Webbys and the FWA: minimal, calm,
intuitive, finished in every state. Every change meets it unasked.

tabflip is an Alkaline Software product and reads as one of the family
(Emaki, Kotoba) because it is drawn to the same rules. This file is
tabflip's whole standard: its accent, its icon's subject, its window's
size, the parts it has and the helpers in its code that draw each
pattern. Where a helper is named in brackets it is in
`crates/tabflip-app/src/`.

## Rules

- **Calm.** One accent, lavender (`theme::LAVENDER`), for what is on,
  in use, chosen, or the one thing a page is for. Nothing else competes
  with it. The list's highlight is the person's own choice of colour and
  marks the selected window alone; it is never used as an accent in
  Settings.
- **Everything fits.** Every section of Settings shows all it has
  without scrolling. The page still scrolls, for a window dragged
  smaller or what is added later; a new row that would push a section
  past the window means the section is reworked, not left to scroll.
- **Nothing jumps.** What appears, goes, opens, closes, grows, moves,
  fills or turns does so over a moment, with the pattern its neighbours
  use (Motion). A section's name fades with its page. A jump is a bug.
- **Words are few.** A row's name says what it is. A line under it is
  there only where the name does not say it all, in a few plain words,
  and never repeats what the control shows.
- **Words are for everyone.** tabflip is used in many languages; nothing
  says how it does in one of them.
- **One way to draw each thing.** A setting is a row on a card (`line`,
  `group`); on or off is a switch (`switch`); a choice among a few is
  `segmented`; a list of many drops down (`pick`); a button is a pill
  with an icon before its word (`ui::pill`); a key is a cap
  (`ui::key_caps`). A new control is a new pattern for every page, or
  it is not made.
- **Things line up.** Names start on one line, controls end on another,
  and the widest controls on a page start and end together
  (`CONTROL_W`). Every field, button and choice in a row is one height.
- **Every language.** A label never depends on English being short:
  controls grow to their words, and a page that fits in English fits in
  French and German. Each language is written as its own Mac writes it,
  with the system's names for its settings.
- **Both appearances, every state.** Light and dark are designed
  together, never one inverted: every colour has a value in each. A
  change is looked at in both, and in every state it has: empty,
  waiting, refused, off, hover, pressed, mid-motion, a narrow window.
- **Readable.** Words read on their ground at 4.5 to 1 or better. A key
  is drawn as a key, never as code. Text on the list's highlight turns
  dark on a pale colour (`look::dark_text_on`), so any colour the person
  picks can be read.
- **Look before saying it is done.** At all the change shows, hides or
  moves, in both appearances (Checking).

## The palette

### The neutrals

One set, in `themes/tabflip.json`, the family's: near white with a
trace of warmth for the page in the light appearance, the darkest
ground the page in the dark one. Three things are kept apart by more
than a shade: the frame, the page, and the accent.

| Role | Light | Dark |
|---|---|---|
| The page | `#FBFBFA` | `#1A1918` |
| Words (the ink) | `#1F1E1D` | `#F3F1EA` |
| Muted: a line under a name, an icon at rest | `#77746B` | `#A09D94` |
| Panel: a card, a control's well | `#F0EFED` | `#222120` |
| Border: every hairline | `#DFDFDC` | `#32312E` |

- The rail stands on the muted shade at half in the light appearance
  and the sidebar's in the dark. A card is a shade off the page: the
  muted colour at 40% in the light appearance, the sidebar's in the
  dark (`card_ground`). A control on a card takes the muted shade.
- A colour that is not in the theme file, the accent or the list's own
  (below) is a mistake.

### The accent

Lavender, the one colour the window has of its own. It is a hue of the
family's wheel at the family's weight: OKLCH lightness 0.52 and chroma
0.125 in the light appearance, 0.74 and 0.105 in the dark, at hue 296,
halfway between periwinkle (272) and orchid (318), so the family's
steps stay even. Hover is 0.02 darker in the light appearance and 0.04
lighter in the dark; active is 0.06 and 0.04 darker. A link is the
hover shade in the light appearance and the base in the dark.

| | Base | Hover | Active |
|---|---|---|---|
| Light | `#7057A8` | `#6B51A2` | `#5F4695` |
| Dark | `#B09DE6` | `#BDAAF3` | `#A491D8` |

White on the light shade is 5.8 to 1; the ink on the dark shade is 7.0
to 1. The light shade is 2 to 1 on the frame, so the icon's lavender is
the dark shade, which stands clear of the navy at 5.9 to 1. It is
painted into both theme configs before the toolkit gets them
(`theme::install`), so every appearance switch keeps it. There is no
choice of accents: a menu bar app's one window has no need of one.

### The list's own colours

The list is drawn over whatever is on screen, in the system's
appearance, and is the one part of tabflip not on the family's page
(`panel::colors`):

| | Light | Dark |
|---|---|---|
| Ground | `#F6F6F6` at 96% | `#1E1E20` at 95% |
| Edge | black at 10% | white at 12% |
| Words | `#1D1D1F` | `#F5F5F7` |
| The app's name, dim | the words at 50% | the words at 50% |

The selected row's colour is the person's (`look::highlight`): one of
the Mac's own eight (Blue, Purple, Pink, Red, Orange, Green, Teal,
Graphite), each following the appearance as the system's do, or a hex
colour of their own. Blue is the default. Text on it is white, or the
dark ink on a pale colour.

## The type

- **Every word of the window** is in the system's face. The list is
  the same face, so it reads as the Mac's own.
- **The name** on About is in Optima, else Avenir Next, both the Mac's
  own and never carried (`wordmark_face`), 42 points in the accent.
- **Sizes in the window**, in points: a section's name 15 semibold; a
  row's name 13; text in a control 12.5; a group's name 12 semibold in
  the muted ink; a row's line 11.5 on a line of 15 in the muted ink.
  Bold is 600, never 700.
- **Sizes in the list**, at Small: the window's word 13, the app's name
  11. Medium and Large scale every measure by 1.25 and 1.5
  (`look::Size::scale`).

## Icons in the window

- **Tabler's outline icons** (tabler.io/icons), MIT, with
  `TABLER-LICENSE` beside them in `assets/icons/`, drawn at a stroke of
  1.75 where Tabler's own is 2: at the window's sizes, 2 reads heavy.
- **Each file is under the name the toolkit asks for**, on one line
  without Tabler's empty bounding path, and named in `OWN`
  (`assets.rs`): a file not named there draws nothing, and nothing says
  so. No call site names Tabler.
- **Every button has an icon before its word.** No button has a word
  and no icon.
- **A choice in the menu bar's menu has an icon**, the rail's for its
  section.
- **The app's own icon** in the window is drawn from the size nearest
  its use (the 512 for About), never one stretched.
- **A key is drawn on a cap** (`ui::key_caps`): small plates of one
  height (19), none narrower than that, each key in its plate's middle,
  a sign (⌘ ⌥ ⇧ ⌃) as an icon and a letter as a letter, both of one
  size. The signs as characters stood at two heights. A shortcut is
  handed to the window as `Shortcut`'s display ("⌃⌥ Tab") and never
  shown as a string.

## The app icon

One drawing in the studio's manner, so the Dock reads it as one of the
family's.

### The manner

- **The ground** is the Alkaline Software cream, a diagonal gradient
  from `#fdf8ee` at the top left to `#f1e4d0` at the bottom right,
  computed per pixel: a canvas gradient fill is dithered by the browser,
  and that noise makes the PNG about ten times larger.
- **The lines** are navy (`#2b2556`), with one amber stroke (`#ffb83e`)
  and the violet four-point spark (`#6a3fd9`), the studio's mark. These
  three are the same in every icon of the family.
- **The product's accent** takes the one place the subject has for a
  colour of its own: lavender's dark shade (`#B09DE6`) fills the window
  that is chosen.
- **The subject** is a stack of three windows seen plainly from the
  front, rounded cards fanned up and to the right as a hand flips
  through them. The front one is the one chosen, filled with lavender,
  and its title bar is the amber stroke. Lines only, few of them, no
  words.
- **It is one icon everywhere.** No dark variant.

### The drawing

- `scripts/icon/icon.html` is the source: a page that draws the icon on
  a canvas in a 100-unit box, with proofs of every size beside it, zoomed
  and at true pixels, on the light and the dark menu bar. The page is the
  place to look at it, and the renderer runs the same page.
- **Every size is drawn on its own pixels**, never scaled down from a
  larger one. Below 40 px across the back card and the title bar give
  way, the cards' edges land on whole pixels and the line thickens from
  2.6 to 3.2 units, and two cards with the lavender fill are what still
  read.
- **Two bodies.** `bleed` fills the whole canvas as a square (the
  `.icns`, which macOS 26 masks itself; anything transparent shows as a
  grey border). Otherwise the body is an inset squircle with a margin
  and a shadow (`rgba(20, 16, 50, 0.28)`, blur 4.5% of the size, offset
  down 2.2%): the PNGs the window shows.
- **The menu bar's image** comes from the same page: the two front
  cards as one black silhouette with the gap between them cut out, on a
  transparent square, scaled to fill it. The system tints it, so only
  its alpha counts.
- **Chosen from candidates.** The page holds a `VARIANT` switch and
  draws every candidate with proofs. The one in use is 1, the fanned
  stack. 2, a window lying flat with one turning on its edge as a page
  does, read as a laptop; 3, one window with three rows and the middle
  row lit (the first icon's drawing, in the manner) read as a list, not a
  switch between windows; 4, the stack with the amber stroke as an arrow
  off the back of the stack onto the front, crowded the spark and read as
  a refresh sign.

### Rendering

`scripts/make-icon.sh`, on a Mac, runs `scripts/icon/render.swift`,
which loads the page in a windowless `WKWebView` and writes what
`exportIcon(size, bleed)` or `exportTemplate(size)` returns, once per
job:

| Output | Sizes | Body |
|---|---|---|
| `assets/icon/icon-<s>.png`, for the window | 1024, 512, 256, 128, 32 | margin |
| `assets/icon/icon.icns`, an iconset through `iconutil` | 16, 32, 128, 256, 512, each at 1x and 2x | bleed |
| `assets/icons/menubar.png`, `menubar-1x.png`, the menu bar item | 36 (18 pt at 2x), 18 | template |

A change to the icon is made on the page, looked at there, and rendered
again.

## The list

The list is tabflip itself: a small stack of rows over everything,
there while the shortcut is held and gone when it is let go. It is a
window of its own, opened once and put away with `orderOut` between
rounds, and never takes the keyboard.

| Part | At Small |
|---|---|
| A row | 30 tall; the app's icon 16, the window's word 13, the app's name 11 at the right, dim |
| The stack | 300 wide, 6 of padding, rounded 12, a hairline edge, a soft shadow |
| The selected row | the highlight colour, rounded 7 |
| Its place | centred on the screen under the pointer, lifted 8% of the screen's height; at most 70% of it tall, scrolling past that |

- Medium and Large scale every measure by the same factor, so rows,
  icons, text and spacing grow together; the margin kept for the shadow
  does not scale.
- A row's label is `label::short_label`'s: a word or two for the
  window, the app's name after it. Twins in one app get a second part.
- The shortcut's key moves down and ⇧ with it moves up; ` with the
  shortcut's modifiers moves up too, as the Mac's own ⌘` is already in
  the hand. The arrows move as well.
- The size the list was shown at is kept for the round, so a change in
  Settings takes effect from the next one.
- The pointer picks only once it has moved after the list came in: the
  panel can open under a pointer at rest, and the system says the
  pointer moved as it does.

## The window

Settings, tabflip's only window, laid out as the Mac's own settings
are: a rail of sections at the left and the section's page at the
right, the section's name in a band at the top level with the traffic
lights, both fading in as the section changes. The window is 780 by
560, no smaller than 680 by 440. Every section fits without scrolling.

| Part | Size |
|---|---|
| The rail | 190 wide; a row a section, 30 tall, an icon and a name, the chosen one on a plate |
| The page's margin | 36 at either side |
| The top band | 52 tall; the section's name 15 semibold |
| Groups | 18 apart |
| A group's name | 12 semibold in the muted ink, 6 over its card, in line with the rows' names |
| A card | rounded 12, a hairline edge in the border's colour, the card's ground; rows parted by the same hairline |
| A row | at least 46 tall, 14 in at either side, 8 above and below; its name 13; its line 11.5 on 15 in the muted ink; its control at the right |
| A control | 28 tall, its text 12.5; the widest 264 (`CONTROL_W`): the shortcut's field of 168 and Reset filling the rest, the size's choice, the highlight's swatches, the preview |
| A switch | 36 by 20, its knob 16; the track the switch colour off and the accent on, both over a moment |
| The language's list | 136 wide, at the right of General's title |

- The rail's General wears a dot in the accent while Accessibility is
  missing, and Updates while a newer tabflip waits.
- A new row is made with `line` in a `group`, never laid out by hand.
- The window's top and the rail are the app's to drag by
  (`drag_region`); a press on a control there takes its press before
  the drag does.

### The sections

- **General** is three groups. Switcher: Shortcut, the keys on their
  caps in a field, Reset beside it, and a line under the row that says
  what the shortcut does, or what is wrong with the one pressed. App:
  Launch at login, Show in menu bar, Swap clicks. Permissions:
  Accessibility, which says whether it is given and the one thing to do
  when it is not: Allow (the system's own question, once an app's life),
  or Open Settings after that.
- **Customize** is one group, The list: Size (Small, Medium, Large on a
  segmented control), Highlight (the eight swatches), Your own colour (a
  field for a hex code, marked when it is not a colour), and Preview, a
  row as the list draws it at the chosen size and colour.
- **Updates** is one card: Version, with what a check found, and one
  button that is Check for updates until a check finds something and
  Update to x after, in the same place, saying how far the download is
  as it goes; and Check automatically. Nothing is fetched or installed
  without the press.
- **About** is the title card, set as a film's first one is: "Alkaline
  Software presents", the icon, the name in the name's face and the
  accent, the version in small grey words; under it a sentence naming
  the studio's owners, each name a link, and pills to the source and the
  release's notes.

### Controls

- **A button** is `ui::pill`, in one of two kinds (`Press`): quiet, the
  default, for one of several and for the way out; primary, filled with
  the accent, for the one thing a page waits on (Allow, Update to x). On
  About it is the smaller pill.
- **A switch** is for on or off, at a press anywhere on it.
- **A segmented control** is for a choice among a few: a muted track,
  the choice on a raised plate that slides from the old choice to the
  new (`segmented`).
- **A list of many** (the language) drops down from a field (`pick`):
  the window's own card, rising in as it fades in and fading out where
  it was, over a clear sheet any click or Escape puts away. The toolkit's
  popover has no way out but at once.
- **A field** (the hex colour) sits in a wrapper that tracks the focus.
- **A hover's label** (a swatch's colour name) is the toolkit's managed
  tooltip, which comes in and slides to the next element on the row.
- **A key** shown anywhere is on a cap.

## The menu bar

- tabflip has `LSUIElement` in every bundle and asks the system to stay
  in the menu bar at launch (`sys::stay_in_menu_bar`), because gpui
  makes every app a regular one as it starts. While Settings are open it
  is a regular app (`sys::in_dock`): in the Dock and the app switcher,
  with its own menu, so the window can be found again once another app
  covers it. Closing Settings, by its button or ⌘W, takes it out again.
- **The item** is at the menu bar's right: the icon's stack as a
  template image (`assets/icons/menubar.png`), 18 pt, drawn at 1x and
  2x, which the system tints for the light bar, the dark one and the
  highlighted state.
- **A left click opens Settings** where they were left; a right click,
  or Control and a click, shows the menu. Swap clicks (General) swaps
  the two, and Show in menu bar hides the item; both change at once
  (`sys::status_item::set`). Opened again from Finder or Spotlight while
  running, tabflip shows Settings, which is the way back with the item
  hidden.
- **The menu is the system's own**, never one drawn by the app: the
  sections in the rail's order with the rail's icons, a line between
  the sections and Updates and About, another before Quit tabflip. The
  item has no menu set on it, or the system would show it for either
  click: the button sends both clicks to a target, which opens Settings
  or sets the menu, clicks the button and takes the menu off again once
  it closes.
- **Its icons are the window's Tabler SVGs** rendered at one and two
  pixels a point into template images (`line_image`, by resvg), so the
  system tints them. An app built on the macOS 27 SDK has its menus'
  images hidden unless each item asks for its own
  (`preferredImageVisibility`).
- **The menu is named again** when the window's language changes
  (`sys::status_item::retitle`).
- **Launch at login** is the system's to keep (`SMAppService`): read
  from the system when the window opens or comes back, never written to
  the settings. The person may turn it off in Login Items.
- **Accessibility** is asked when the person presses Allow, never
  unasked. Allow first forgets the entry the system keeps for the app
  (`sys::forget_trust`): the system binds a grant to the app's
  signature, so an entry made for a copy signed otherwise (an earlier
  build at the same path) shows as on in System Settings and counts for
  nothing, and switching it off and on changes nothing. Then the
  system's own question puts the app in the list afresh. The system
  asks only once in an app's life, so a later press opens Privacy &
  Security itself. The list is watched (`sys::trust_changes`) and the
  switcher starts the moment it is given, by trying the tap, never by
  asking `AXIsProcessTrusted`, which a running process reads once;
  Settings open on General at launch while it is missing, which is where
  the permission is.

## Motion

Nothing in the window or the list appears or disappears from one frame
to the next. Each pattern below is in use; a new thing takes the pattern
its neighbours use, and a change that needs a new pattern writes it
here.

### Timing

Every motion, in the window and in the list, is a gpui animation
(`with_animation`) easing with `ease_out_quint`, keyed so it plays once.
The list's own clock (a frame asked for at each draw while something
moved) was tried first and dropped: the slide was rough.

| What | ms |
|---|---|
| The list coming in, fading as it drifts 6 points down into place | 110 (`ENTER`) |
| The list going out, fading where it is over the window picked | 110 (`LEAVE`) |
| The highlight sliding from row to row, a cubic ease | 150 (`SLIDE`) |
| A list over the window coming in, rising 6 points | 140 (`MENU_IN`) |
| A list over the window going out | 110 (`MENU_OUT`) |
| A switch turning; something small that is there or is not | 200 (`TURN`) |
| A segmented control's plate | 220 |
| A page's content and the section's name fading in, rising 8 points | 240 (`page_in`) |
| The window fading into a new language | 260 |

The list is shorter than a menu on purpose: it is under a held key, and
a hand that has pressed Tab is already waiting.

### Three rules of every pattern

- **The first sight plays nothing.** The window's first draw, a switch
  drawn for the first time: nothing moves. A change is played once,
  keyed on what changed (the number of the turn, the new choice of a
  segmented control, the serial of each opening of a list), so a control
  drawn again later does not play its change over.
- **Going is drawn from what it last showed.** The list is kept for the
  moment of its going and drawn once more, fading; a dropped-down list
  is kept and drawn once more, taking no click meanwhile.
- **The room moves with the thing.** A row's line that comes or goes is
  inside what opens, so the row grows and does not jump.

### The patterns

- **The list** fades in where it is, drifting a few points down into
  place, and on the way out fades over the window just picked. It is put
  away only after a fully clear frame has been drawn, so the next round
  never shows the last one's rows; a round number keeps a late put-away
  from hiding a newer showing.
- **The highlight** is one plate under the rows that slides from the
  old row to the new. A slide cut short by the next key goes on from
  where the plate is drawn now, never from where it was going.
- **A list over the window** (the language) comes in rising a few
  pixels into place as it fades in, and goes out fading where it was.
- **A choice that moves** (Size): one plate slides from the old to the
  new, placed from where each segment was drawn last (`seg_bounds`),
  never measured on the first draw, where the chosen segment paints its
  own plate so nothing flashes.
- **A switch**: the knob slides as the track fills with the accent.
- **Something small that is there or is not** (a dot on the rail,
  Allowed in place of Allow) is asked at every draw (`presence`), which
  plays a change and not the first sight.
- **Each page's content fades in** when it arrives, keyed on the page;
  a section's name fades with its page; the window changing its
  language fades into the new one.

### Left as they are, on purpose

The rows of the list changing as a fresh look replaces the remembered
one (the content is the thing, and the same window stays selected); a
hover's ground; the list's size, which changes from the next round.

### Traps

- A hover style that changes the text's colour is set whatever state
  the element is in, never under a `when`: gpui keeps the "over" flag
  only while a hover style is set, and a segment came back to rest lit.
- A measuring `canvas` laid over an element is `.absolute().inset_0()`;
  with `.size_full()` alone it stands after the element in the flow.
- gpui hears of a window's resize through an update of that window; the
  panel resizes itself from inside its own update, so it tells gpui
  itself (`window.bounds_changed`).
- gpui holds an inactive window's animations to 30 frames a second. The
  list is never the active window (it must not take the keyboard from
  the app in front), so it opts out (`inactive_frame_interval: None`):
  at 30 a 150 ms slide was a jump with one smeared frame.
- The highlight's slide eases out as a cubic, not the quint the window
  uses: over 150 ms the quint did most of the travel in its first 40,
  which read as a jump. The rows' inks follow the plate's coverage, so a
  row's words never turn white before the plate is under them.
- A border is inside the box. The list's rows are given exactly their
  room, so the stack's height counts its hairline edge (`EDGE`): without
  it the rows stood 2 points taller than their scroller, and every move
  to the first or last row scrolled the whole list by that much. The
  scroller starts each round at the top, whatever the last one left.

## Languages

The window speaks English, Chinese (Simplified and Traditional, told
apart by script or region), Japanese, Korean, French, German and
Spanish. By default it follows the system: the first of its preferred
languages tabflip has (`i18n::pick`), else English. Only the window's
words change: the list shows each window's own title and its app's own
name.

- **English is the key.** A phrase is written in the code in English
  inside `t("…")`, or `tf("… {name} …", …)` with blanks, and
  `locales/words.json` holds its other languages. A sentence with links
  in it (About) is woven from its translation (`weave`), so each
  language puts the links where its order wants them. `cargo test -p
  tabflip-app` fails on a phrase asked for and not in the table, one in
  the table no longer asked for, a language missing, or a blank that
  differs from the English.
- **The language is chosen** from a list at the right of General's
  title: System first, then each language by its flag and letters
  (`i18n::CHOICES`). A choice takes at once: the words, the menu bar's
  menu, the app's menu, and the window fades into the new language.
- **On a Mac the choice is the app's own `AppleLanguages`**, the setting
  System Settings, Language & Region, Applications writes, so the two
  are one (`sys::own_language`). The bundle declares its languages
  (`CFBundleLocalizations`) so the system offers them.
- **French and German run longest**: a control holds them by growing to
  its word, never by a width its English fits. Check a change in French,
  German and Spanish (`TABFLIP_LANG`).

## Checking

Shoot every state a change touches, in both appearances, and look at it
whole and close up. Then go round again: what is out of line, louder
than it needs to be, or not consistent with its neighbours is fixed and
shot again, until nothing is left to change. A page that reads well but
breaks a rule here is not done.

The probes need no permission: a process may capture its own windows.

```
TABFLIP_HOME=<scratch> …                                          # settings of the probe's own, never the person's
TABFLIP_SHOT=<panel.png> target/debug/tabflip                     # the list with sample rows, then quit
TABFLIP_APPEARANCE=dark …                                         # the list in that appearance, in place of the system's
TABFLIP_SHOT_SETTINGS=<settings.png> TABFLIP_SETTINGS=customize … # a section of Settings
TABFLIP_GO="dark;pick:language" …                                 # steps before the picture, a moment apart (`settings::go`)
TABFLIP_LANG=fr …                                                 # the window in a language, in place of the system's
TABFLIP_DEMO_LOOP=<dir> …                                         # the list shown and put away four times, demo0.png…
target/debug/tabflip --list                                       # the windows as the switcher sees them (needs Accessibility)
```

`TABFLIP_FRAME_LOG=1` writes each frame of the highlight's slide to the
dev log with its time, and `TABFLIP_DEMO_LOOP` steps down two rows each
round, so the frame rate can be read off: 16.7 ms apart on a 60 Hz
display.

`TABFLIP_GO` steps: `dark`, `light`; `pick:language` (the list open);
`language:<code>`; `section:<key>`; `untrusted` and `asked` (the
permission not given, before and after one Allow); `record` (the
shortcut's field waiting for keys); `newer` (a newer version found).
