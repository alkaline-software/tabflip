# Design

The standard every part of tabflip a person sees is held to: the list that shows while the shortcut is held, the Settings window and its controls, the menu bar item and its menu, the app icon. The bar is the work that wins Awwwards, the Webbys and the FWA: minimal, calm, intuitive, finished in every state. Every change meets it unasked.

tabflip is an Alkaline Software product and reads as one of the studio's because it is drawn to the same rules. This file is tabflip's whole standard: its accent, its icon's subject, its window's size, the parts it has and the helpers in its code that draw each pattern. A helper named in brackets is in `crates/tabflip-app/src/`.

## Rules

- **Calm.** One accent, sapphire (`theme::SAPPHIRE`), for what is on, in use, chosen, or the one thing a page is for. Nothing else competes with it. The list's accent is the person's choice of color, sapphire unless they choose another, and marks the selected window alone; Settings keep the sapphire whatever the list wears.
- **Everything fits.** Every section of Settings shows all it has without scrolling. The page still scrolls, for a window dragged shorter or what is added later; a new row that would push a section past the window means the section is reworked, not left to scroll.
- **Nothing jumps.** What appears, goes, opens, closes, grows, moves, fills or turns does so over a moment, with the pattern its neighbors use (Motion). A section's name fades with its page. A jump is a bug.
- **No pause at the end of a motion.** It runs into its place and stops, with no crawl or second step in its last frames.
- **Nothing slow is asked while a frame is built** (Motion, Frames).
- **Words are few.** A row's name says what it is. A line under it is there only where the name does not say it all, in a few plain words, and never repeats what the control shows.
- **Words are for everyone.** tabflip is used in many languages; nothing says how it does in one of them.
- **One way to draw each thing.** A setting is a row on a card (`line`, `group`); on or off is a switch (`switch`); a choice among a few is `segmented`; a list of many drops down (`pick`); a button is a pill with an icon before its word (`ui::pill`); a key is a cap (`ui::key_caps`). A new control is a new pattern for every page, or it is not made.
- **Things line up.** Names start on one line, controls end on another, and the widest controls on a page start and end together (`CONTROL_W`). Every field, button and choice in a row is one height.
- **Every language.** A label never depends on English being short: controls grow to their words, and a page that fits in English fits in French and German. Each language is written as its own Mac writes it, with the system's names for its settings.
- **Both appearances, every state.** Light and dark are designed together, never one inverted: every color has a value in each. A change is looked at in both, and in every state it has: empty, waiting, refused, off, hover, pressed, mid-motion, a short window.
- **Readable.** Words read on their ground at 4.5 to 1 or better. A key is drawn as a key, never as code. Text on the list's accent turns dark on a pale color (`look::dark_text_on`), so any color the person picks can be read.
- **Look before saying it is done.** At all the change shows, hides or moves, in both appearances (Checking).

## The palette

### The neutrals

One set, in `themes/tabflip.json`, the studio's: near white with a trace of warmth for the page in the light appearance, the darkest ground the page in the dark one. Three things are kept apart by more than a shade: the frame, the page, and the accent.

| Role | Light | Dark |
|---|---|---|
| The page | `#FBFBFA` | `#1A1918` |
| Words (the ink) | `#1F1E1D` | `#F3F1EA` |
| Muted: a line under a name, an icon at rest | `#77746B` | `#A09D94` |
| Panel: a card, a control's well | `#F0EFED` | `#222120` |
| Border: every hairline | `#DFDFDC` | `#32312E` |

- The rail stands on the muted shade at half in the light appearance and the sidebar's in the dark. A card is a shade off the page: the muted color at 40% in the light appearance, the sidebar's in the dark (`card_ground`). A control on a card takes the muted shade.
- A color that is not in the theme file, the accent or the list's own (below) is a mistake.

### The accent

Sapphire, the one color the window has of its own, and the first and default color of the list. It is a blue at the studio's weight: OKLCH lightness 0.52 and chroma 0.135 in the light appearance, 0.74 and 0.105 in the dark, at hue 255. The Mac's own blue was the list's first color and read loud and electric; at the studio's lightness and a softer chroma the same idea is deep and calm. Hues 250 and 260 were tried beside it in the list: 250 leaned to cyan, 260 to violet. Hover is 0.02 darker in the light appearance and 0.04 lighter in the dark; active is 0.06 and 0.04 darker. A link is the hover shade in the light appearance and the base in the dark.

| | Base | Hover | Active |
|---|---|---|---|
| Light | `#2A69B4` | `#2463AE` | `#1657A1` |
| Dark | `#7DAEEC` | `#89BAFA` | `#71A1DF` |

White on the light shade is 5.6 to 1; the ink on the dark shade is 7.2 to 1. The light shade is 2 to 1 on the frame, so the icon's sapphire is the dark shade, which stands clear of the navy at 6.1 to 1. It is painted into both theme configs before the toolkit gets them (`theme::install`), so every appearance switch keeps it. The person's choice of accent colors the list alone: a menu bar app's one window has no need of a choice of its own.

### The list's own colors

The list is drawn over whatever is on screen, in the system's appearance, and is the one part of tabflip not on the studio's page (`panel::colors`):

| | Light | Dark |
|---|---|---|
| Ground | `#F6F6F6` at 96% | `#1E1E20` at 95% |
| Edge | black at 10% | white at 12% |
| Words | `#1D1D1F` | `#F5F5F7` |
| The app's name, dim | the words at 50% | the words at 50% |

The selected row's color is the list's accent, the person's choice (`look::highlight`, kept in the config as `highlight`): seven colors of one family, or a hex color of the person's own. The seven are drawn to the sapphire's standard: OKLCH lightness 0.52 in the light appearance and 0.55 to 0.56 on the dark list, a step lighter so white still reads on it, at one chroma eased where a hue is loud, so no color weighs more than another and white reads on each at 4.5 to 1 or better. The Mac's own colors were the list's first set; at the system's brightness its blue read electric and its orange and green would not carry white. Red is left out: beside pink and orange it was a fourth warm color.

| Color | Hue | Light | Dark |
|---|---|---|---|
| Blue, the default (sapphire) | 255 | `#2A69B4` | `#3475C4` |
| Purple | 305 | `#7B51A6` | `#885CB5` |
| Pink | 355 | `#A4406D` | `#B14C79` |
| Orange, at 0.56 and 0.58, where it is not brown | 55 | `#AD5C15` | `#B3621E` |
| Green, chroma eased | 145 | `#3E7941` | `#448247` |
| Teal, chroma eased | 210 | `#147684` | `#177F8E` |
| Graphite, the sapphire's gray | 255 | `#626A73` | `#6E757F` |

Text on the selected row is white, or the dark ink on a pale color of the person's own.

## The type

- **Every word of the window** is in the system's face. The list is the same face, so it reads as the Mac's own.
- **The name** on About is in Optima, else Avenir Next, both the Mac's own and never carried, found once at launch (`ui::find_faces`). Optima has only regular and bold, so the regular is drawn twice half a pixel apart for a weight between (`ui::wordmark`), at 42 points in the accent.
- **Sizes in the window**, in points: a section's name 15 semibold; a row's name 13; text in a control 12.5; a group's name 12 semibold in the muted ink; a row's line 11.5 on a line of 15 in the muted ink. Bold is 600, never 700.
- **Sizes in the list**, at Small: the window's word 13, the app's name 11. Medium and Large scale every measure by 1.25 and 1.5 (`look::Size::scale`).

## Icons in the window

- **Tabler's outline icons** (tabler.io/icons), MIT, with `TABLER-LICENSE` beside them in `assets/icons/`, drawn at a stroke of 1.75 where Tabler's own is 2: at the window's sizes, 2 reads heavy.
- **Each file is under the name the toolkit asks for**, on one line without Tabler's empty bounding path, and named in `OWN` (`assets.rs`): a file not named there draws nothing, and nothing says so. No call site names Tabler.
- **Every button has an icon before its word.** No button has a word and no icon.
- **A choice in the menu bar's menu has an icon**, the rail's for its section.
- **The app's own icon** in the window is drawn from the size nearest its use (the 512 for About), never one stretched.
- **A key is drawn on a cap** (`ui::key_caps`): small plates of one height (19), none narrower than that, each key in its plate's middle, a sign (⌘ ⌥ ⇧ ⌃) as an icon and a letter as a letter, both of one size. The signs as characters stood at two heights. A shortcut is handed to the window as `Shortcut`'s display ("⌃⌥ Tab") and never shown as a string.

## The app icon

One drawing in the studio's manner, so the Dock reads it as one of the studio's.

### The manner

- **The ground** is the Alkaline Software cream with a fifth of sapphire in it, so the Dock tells the studio's icons apart by their grounds while the drawings stay one hand. The cream is a diagonal gradient from `#fdf8ee` at the top left to `#f1e4d0` at the bottom right; each end is mixed with the mean of sapphire's two base shades (`#2A69B4` and `#7DAEEC`), 16% at the top left and 24% at the bottom right, so the wash deepens toward the shadowed corner: `#e2e7e9` to `#cbcfd0` (`ground` in `icon.html`). The gradient is computed per pixel: a canvas gradient fill is dithered by the browser, and that noise makes the PNG about ten times larger.
- **The lines** are navy (`#2b2556`), with one amber stroke (`#ffb83e`), the same in every icon of the studio. tabflip's carries no violet spark: beside the cards it crowded the drawing.
- **The product's accent** takes the one place the subject has for a color of its own: sapphire's dark shade (`#7DAEEC`) fills the window that is chosen.
- **The subject** is two windows seen plainly from the front, rounded cards, the one in front up and to the right of the one behind, as the switcher moves from the window just used to the one chosen. The front one is filled with sapphire and its title bar is the amber stroke; the one behind is white paper (the studio's cream read warm and out of place on the cool ground). The menu bar item is the same two cards, so the item and the icon are one drawing. Lines only, few of them, no words.
- **It keeps to the room the studio's other drawings take**: about 16 to 84 across and 20 to 80 down in the 100-unit body, lines included, centered (`PAIR` in `icon.html`). The first stack ran from 8 to 92 and read as touching the edges beside its neighbors in the Dock.
- **It is one icon everywhere.** No dark variant.

### The drawing

- `scripts/icon/icon.html` is the source: a page that draws the icon on a canvas in a 100-unit box, with proofs of every size beside it, zoomed and at true pixels, on the light and the dark menu bar. The page is the place to look at it, and the renderer runs the same page.
- **Every size is drawn on its own pixels**, never scaled down from a larger one. Below 40 px across the title bar gives way, the cards' edges land on whole pixels and the line thickens from 2.6 to 3.2 units.
- **Two bodies.** `bleed` fills the whole canvas as a square (the `.icns`, which macOS 26 masks itself; anything transparent shows as a gray border). Otherwise the body is an inset squircle with a margin and a shadow (`rgba(20, 16, 50, 0.28)`, blur 4.5% of the size, offset down 2.2%): the PNGs the window shows.
- **The menu bar's image** comes from the same page: the two cards (`PAIR`) as one black silhouette with the gap between them cut out, on a transparent square, scaled to fill it (`pairTemplate`). The system tints it, so only its alpha counts.
- **Chosen from candidates.** The page holds a `VARIANT` switch and draws every candidate with proofs. The one in use is 5, two cards. 1, a stack of three fanned up and to the right, was the icon until its two front cards, already the menu bar item, were made the icon too; 6, a hand of cards fanned from one point, and 7, the list in one window, are drawn in the same room beside it. Older ones: 2, a window lying flat with one turning on its edge as a page does, read as a laptop; 3, one window with three rows and the middle row lit read as a list, not a switch between windows; 4, the stack with the amber stroke as an arrow off the back of the stack onto the front, crowded the corner and read as a refresh sign.

### Rendering

`scripts/make-icon.sh`, on a Mac, runs `scripts/icon/render.swift`, which loads the page in a windowless `WKWebView` and writes what `exportIcon(size, bleed)` or `exportTemplate(size)` returns, once per job:

| Output | Sizes | Body |
|---|---|---|
| `assets/icon/icon-<s>.png`, for the window | 1024, 512, 256, 128, 32 | margin |
| `assets/icon/icon.icns`, an iconset through `iconutil` | 16, 32, 128, 256, 512, each at 1x and 2x | bleed |
| `assets/icons/menubar.png`, `menubar-1x.png`, the menu bar item | 36 (18 pt at 2x), 18 | template |

A change to the icon is made on the page, looked at there, rendered again, built, and the assets committed.

## The list

The list is tabflip itself: a small stack of rows over everything, there while the shortcut is held and gone when it is let go. It is a window of its own, opened once and put away between rounds, and never takes the keyboard (`docs/platform.md`, The list's window).

| Part | At Small |
|---|---|
| A row | 30 tall; the app's icon 16, the window's word 13, the app's name 11 at the right, dim |
| The stack | 300 wide, 6 of padding, rounded 12, a hairline edge, a soft shadow |
| The selected row | the list's accent, rounded 7 |
| Its place | centered on the screen under the pointer, lifted 8% of the screen's height; at most 70% of it tall, scrolling past that |

- Medium and Large scale every measure by the same factor, so rows, icons, text and spacing grow together; the margin kept for the shadow does not scale.
- A row's label is `label::short_label`'s: a word or two for the window, the app's name after it (`docs/switcher.md`, What a row says).
- The shortcut's key moves down and ⇧ with it moves up; ` with the shortcut's modifiers moves up too. The arrows move as well.
- The size the list was shown at is kept for the round, so a change in Settings takes effect from the next one.
- The pointer picks only once it has moved after the list came in: the panel can open under a pointer at rest, and the system says the pointer moved as it does.

## The window

Settings, tabflip's only window, laid out as the Mac's own settings are: a rail of sections at the left and the section's page at the right, the section's name in a band at the top level with the traffic lights, both fading in as the section changes. The window opens at 820 by 600, as every product's settings do, so the studio's windows are one size. It is never narrower, since its rows are laid out for that width, and goes down to 460 tall, where the page scrolls. Every section fits without scrolling.

| Part | Size |
|---|---|
| The rail | 190 wide; a row a section, 30 tall, an icon and a name, the chosen one on a plate |
| The page's margin | 36 at either side |
| The top band | 52 tall; the section's name 15 semibold |
| Groups | 18 apart |
| A group's name | 12 semibold in the muted ink, 6 over its card, in line with the rows' names |
| A card | rounded 12, a hairline edge in the border's color, the card's ground; rows parted by the same hairline |
| A row | at least 46 tall, 14 in at either side, 8 above and below; its name 13; its line 11.5 on 15 in the muted ink; its control at the right |
| A control | 28 tall, its text 12.5; the widest 264 (`CONTROL_W`): the shortcut's field of 168 and Reset filling the rest, the size's choice, the highlight's swatches, the preview |
| A switch | 36 by 20, its knob 16; the track the switch color off and the accent on, both over a moment |
| The language's list | 136 wide, at the right of General's title |

- The rail's General wears a dot in the accent while Accessibility is missing, and About while a newer tabflip waits.
- A new row is made with `line` in a `group`, never laid out by hand.
- The window's top and the rail are the app's to drag by (`drag_region`); a press on a control there takes its press before the drag does.

### The sections

- **General** is three groups. Switcher: Shortcut, the keys on their caps in a field, Reset beside it, and a line under the row that says what the shortcut does, or what is wrong with the one pressed. App: Launch at login, Show in menu bar, Swap clicks. Permissions: Accessibility, which says whether it is given and the one thing to do when it is not: Allow (the system's own question, once an app's life), or Open Settings after that.
- **Customize** is one group, The list: Size (Small, Medium, Large on a segmented control), Accent (the eight swatches), Your own color (a field for a hex code, marked when it is not a color), and Preview, a row as the list draws it at the chosen size and color.
- **About** is the same in every product of the studio. A card holds the title card, set as a film's first one is: "Alkaline Software presents" at 14 points, the studio a link; the app's icon at 88; its name at 42 in the name's face and the accent; the version at 12.5 in the muted ink. Under the card, pills to the source (GitHub) and the release's notes; tabflip has no site of its own, so there is no pill to one. Then, a little further down, the updates: a card of two rows, Updates (what a check found, and the one button that is Check for updates until a check finds something and Update to x after, saying how far the download is as it goes, with the release's notes under it then) and Check automatically, with no line under it. Nothing is fetched or installed without the press. The studio is the only name About gives.

### Controls

- **A button** is `ui::pill`, in one of two kinds (`Press`): quiet, the default, for one of several and for the way out; primary, filled with the accent, for the one thing a page waits on (Allow, Update to x). Under About's card it is the smaller pill.
- **A switch** is for on or off, at a press anywhere on it.
- **A segmented control** is for a choice among a few: a muted track, the choice on a raised plate that slides from the old choice to the new (`segmented`).
- **A list of many** (the language) drops down from a field (`pick`): the window's own card, rising in as it fades in and fading out where it was, over a clear sheet any click or Escape puts away. The toolkit's popover has no way out but at once.
- **A field** (the hex color) sits in a wrapper that tracks the focus.
- **A hover's label** (a swatch's color name) is the toolkit's managed tooltip, which comes in and slides to the next element on the row.
- **A key** shown anywhere is on a cap.

## The menu bar

- tabflip has `LSUIElement` in every bundle and is a regular app only while Settings are open, so the window can be found again once another app covers it (`docs/platform.md`, The menu bar and the Dock).
- **The item** is at the menu bar's right: the icon's two cards as a template image (`assets/icons/menubar.png`), 18 pt, drawn at 1x and 2x, which the system tints for the light bar, the dark one and the highlighted state.
- **A left click opens Settings** where they were left; a right click, or Control and a click, shows the menu. Swap clicks (General) swaps the two, and Show in menu bar hides the item; both change at once (`sys::status_item::set`). Opened again from Finder or Spotlight while running, tabflip shows Settings, which is the way back with the item hidden.
- **The menu is the system's own**, never one drawn by the app: the sections in the rail's order with the rail's icons, a line before About, another before Quit tabflip.
- **Its icons are the window's Tabler SVGs** rendered at one and two pixels a point into template images (`line_image`, by resvg), so the system tints them. An app built on the macOS 27 SDK has its menus' images hidden unless each item asks for its own (`preferredImageVisibility`).
- **The menu is named again** when the window's language changes (`sys::status_item::retitle`).
- **Launch at login** is the system's to keep (`SMAppService`): read from the system when the window opens or comes back, never written to the settings. The person may turn it off in Login Items.
- **Accessibility** is asked when the person presses Allow, never unasked, and the row says whether it is given and the one thing to do when it is not (`docs/platform.md`, Accessibility).

## Motion

Nothing in the window or the list appears or disappears from one frame to the next. Each pattern below is in use; a new thing takes the pattern its neighbors use, and a change that needs a new pattern writes it here.

### Timing

Every motion, in the window and in the list, is a gpui animation (`with_animation`) easing with `ease_out_quint`, keyed so it plays once. The list's own clock (a frame asked for at each draw while something moved) was tried first and dropped: the slide was rough.

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

The list is shorter than a menu on purpose: it is under a held key, and a hand that has pressed Tab is already waiting.

### Three rules of every pattern

- **The first sight plays nothing.** The window's first draw, a switch drawn for the first time: nothing moves. A change is played once, keyed on what changed (the number of the turn, the new choice of a segmented control, the serial of each opening of a list), so a control drawn again later does not play its change over.
- **Going is drawn from what it last showed.** The list is kept for the moment of its going and drawn once more, fading; a dropped-down list is kept and drawn once more, taking no click meanwhile.
- **The room moves with the thing.** A row's line that comes or goes is inside what opens, so the row grows and does not jump.

### The patterns

- **The list** fades in where it is, drifting a few points down into place, and on the way out fades over the window just picked. It is put away only after a fully clear frame has been drawn, so the next round never shows the last one's rows; a round number keeps a late put-away from hiding a newer showing.
- **The highlight** is one plate under the rows that slides from the old row to the new. A slide cut short by the next key goes on from where the plate is drawn now, never from where it was going.
- **A list over the window** (the language) comes in rising a few pixels into place as it fades in, and goes out fading where it was.
- **A choice that moves** (Size): one plate slides from the old to the new, placed from where each segment was drawn last (`seg_bounds`), never measured on the first draw, where the chosen segment paints its own plate so nothing flashes.
- **A switch**: the knob slides as the track fills with the accent.
- **Something small that is there or is not** (a dot on the rail, Allowed in place of Allow) is asked at every draw (`presence`), which plays a change and not the first sight.
- **Each page's content fades in** when it arrives, keyed on the page; a section's name fades with its page; the window changing its language fades into the new one.

### Frames

gpui lays out and paints the whole of a window at every frame it draws, and a hover draws it, so whatever `render` calls runs at every hover and every frame of a motion.

- **Nothing slow is asked while a frame is built.** What the system has is asked once and kept. Listing every installed face takes about 135 ms on a Mac with many fonts; About asked for it at each draw to find the name's face, and each of its frames took 120 to 280 ms. It is found once at launch now (`ui::find_faces`), and a frame of About is back to one display frame. Nothing reads the disk or asks another app at a draw.
- **The list reads nothing at a frame.** What a probe set in the environment is read once (`panel::probe`); the rows are what the last look found.
- **The list's frames start when it is due.** Its window goes on screen clear at the shortcut's press, so gpui's frames are running when the list shows (`docs/platform.md`, The list's window): every slide and fade of four rounds runs at the display's rate, no frame more than 20 ms after the last at 60 Hz, the first round included.
- **A clock runs only while it has something to move.** The toolkit's notifications ticked every 50 ms for as long as a window was open, in a window that never shows one; they tick only from a notification's arrival to its going (`vendor/gpui-component/UPSTREAM.md`).
- **The main thread never waits on another app** (`docs/platform.md`, Rules). A motion runs on gpui's clock, so a held thread is a motion lost: its first frame is drawn at nothing and the next at its end.

### Left as they are, on purpose

The rows of the list changing as a fresh look replaces the remembered one (the content is the thing, and the same window stays selected); a hover's ground; the list's size, which changes from the next round.

### Traps

- A hover style that changes the text's color is set whatever state the element is in, never under a `when`: gpui keeps the "over" flag only while a hover style is set, and a segment came back to rest lit.
- A measuring `canvas` laid over an element is `.absolute().inset_0()`; with `.size_full()` alone it stands after the element in the flow.
- gpui hears of a window's resize through an update of that window; the panel resizes itself from inside its own update, so it tells gpui itself (`window.bounds_changed`).
- gpui holds an inactive window's animations to 30 frames a second. The list is never the active window (it must not take the keyboard from the app in front), so it opts out (`inactive_frame_interval: None`): at 30 a 150 ms slide was a jump with one smeared frame.
- The highlight's slide eases out as a cubic, not the quint the window uses: over 150 ms the quint did most of the travel in its first 40, which read as a jump. The rows' inks follow the plate's coverage, so a row's words never turn white before the plate is under them.
- A border is inside the box. The list's rows are given exactly their room, so the stack's height counts its hairline edge (`EDGE`): without it the rows stood 2 points taller than their scroller, and every move to the first or last row scrolled the whole list by that much. The scroller starts each round at the top, whatever the last one left.
- `Animation::new(Duration::ZERO)` stops a debug build: gpui divides by the length. An element that animates only sometimes keeps a real length and does nothing in its closure when nothing moves.

## Languages

The window speaks English, Chinese (Simplified and Traditional, told apart by script or region), Japanese, Korean, French, German and Spanish. By default it follows the system: the first of its preferred languages tabflip has (`i18n::pick`), else English. Only the window's words change: the list shows each window's own title and its app's own name.

- **English is the key.** A phrase is written in the code in English inside `t("…")`, or `tf("… {name} …", …)` with blanks, and `locales/words.json` holds its other languages. A sentence with links in it (About's title) is woven from its translation (`weave`), so each language puts the links where its order wants them. `cargo test -p tabflip-app` fails on a phrase asked for and not in the table, one in the table no longer asked for, a language missing, or a blank that differs from the English.
- **The language is chosen** from a list at the right of General's title: System first, then each language by its flag and letters (`i18n::CHOICES`). A choice takes at once: the words, the menu bar's menu, the app's menu, and the window fades into the new language.
- **On a Mac the choice is the app's own `AppleLanguages`** (`docs/platform.md`, Languages).
- **French and German run longest**: a control holds them by growing to its word, never by a width its English fits. Check a change in French, German and Chinese (`TABFLIP_LANG`).

## Checking

Shoot every state a change touches, in both appearances, and look at it whole and close up (`docs/probing.md` says how to drive the list and Settings from a script). Then go round again: what is out of line, louder than it needs to be, or not consistent with its neighbors is fixed and shot again, until nothing is left to change. A page that reads well but breaks a rule here is not done.
