# Platform

About `crates/tabflip-app/src/sys/`: every call to macOS (`ax.rs`, `windows.rs`, `focus.rs`, `tap.rs`, `private.rs`, `status_item.rs`, and window placement, languages, login and the Dock in `mod.rs`), the bundle, and the dev build.

Tabflip is a Mac's alone. What it does (read every app's windows, raise one, take ⌘Tab from the system) is Accessibility, the window server and an event tap, none of which has a counterpart it could share code with elsewhere. The core (`tabflip-core`) needs neither a window nor the OS and is tested anywhere; everything else here is the Mac's.

## Rules

- **The system's ⌘Tab is always given back.** `CGSSetSymbolicHotKeyEnabled` outlives the process. It is switched back on first thing in `main` (before anything can fail), at quit, on SIGHUP, SIGINT and SIGTERM, and in the panic hook; a kill -9 is covered by the next launch. It is only ever turned off while the shortcut is exactly ⌘Tab.
- **One Tabflip at a time.** A Tabflip opened while another runs, the release or the build, asks it to quit and makes it a few seconds on (`sys::end_other_tabflips`): two would each take the shortcut. The one quitting gives the system its ⌘Tab back, so the one staying applies its shortcut again once the other is gone. A probe copy, with its own `TABFLIP_HOME`, ends none.
- **The tap runs on a thread of its own.** Every key and click in the session waits for an event tap to answer. On the main run loop it waited for each frame the app drew, so the app's frames held up every keystroke on the Mac. `tap::start` makes the tap where it is called, so a refusal is known at once, and serves its port from a run loop on the "key tap" thread.
- **The tap callback never does slow work.** It matches the event, sends a `tap::Key` down a channel and returns: no Accessibility call, no allocation beyond the send. A slow tap is disabled by the system (`DISABLED_BY_TIMEOUT`), and the callback enables it again.
- **The tap holds keys back only while it must.** Closed, it swallows the trigger alone; open (`OPEN`), every key down and up, so the app in front never sees half a keystroke. Modifiers always pass, so every app agrees on what is held.
- **The main thread never waits on another app.** Every Accessibility question goes to the background executor: the round's look, the window in front (`windows::focused`), the focus itself. An app that hangs may be the one in front.
- **Other apps are asked only when it matters.** A look asks every running app for its windows and each must answer, so it is taken when a round begins, never at every app switch; a switch asks the one app in front for its window in front (`docs/switcher.md`, Rules).
- **At rest Tabflip does nothing**: no timer runs but the hourly look at whether an update is due, and with Settings closed it is at 0% with no wakeups. While a window of gpui's is on screen its display link wakes the main thread at every display frame and draws only what changed: about 0.4% of a core while Settings sit open in front, 30 frames a second behind another app, nothing once they close or are covered.
- **Every AX app element gets a 0.25 s messaging timeout** (`Element::app`), and so does the system-wide element (`ax::limit_waits`), which covers the windows an app returns. One hung app must not stall the list.
- **Accessibility is the one permission.** Titles come from AX; stacking order and window numbers from `CGWindowListCopyWindowInfo`, which needs no Screen Recording for those fields. `kCGWindowName` is never read: it would bring in Screen Recording.
- **Private calls are looked up by name** (`private.rs`, `dlsym`), so a macOS that removes one leaves that feature missing, not the app unable to launch. `_AXUIElementGetWindow` is the one linked directly; every window manager on the Mac depends on it.
- **Only the current Space**, plus minimized and hidden windows. A window neither on screen nor put away is on another Space and is left out (`windows::list`). All Spaces needs the private CGS space calls.

## The windows

- **The apps** are those the Dock shows (a regular activation policy), read with `NSWorkspace` (`windows::apps`). Each is asked on a thread of its own for its windows, their titles, subroles and whether they are minimized, so a look takes as long as the slowest app, and no app longer than the timeout a question.
- **Kept**: standard windows, and dialogs with a title. Not palettes, sheets or popovers.
- **The stack** is the window server's list of layer 0 windows on screen, front to back (`windows::on_screen`).
- **Icons** are each app's own, drawn once at 32 px into a PNG when the app is first seen and kept while it has windows (`Controller::looked`).

## Focus

`focus::focus` brings one window to the front with the keyboard in it, off the main thread. On macOS 14 and later a background app's `activateWithOptions` can be declined by the app in front (cooperative activation), so the window server's way goes first: `_SLPSSetFrontProcessWithOptions` and the two make-key event records (`private::bring_to_front`). Then `AXRaise`, `AXMain` and the app's `AXFocusedWindow`; the public call is the fallback. A hidden app is unhidden and a minimized window restored first.

## The list's window

- **Opened once and put away with `orderOut`.** It fades out over the picked window and is hidden only after a clear frame has been drawn, so the next round never flashes the last one's rows. Put away, it draws nothing at all, and it is not drawn again for a change elsewhere in the app. A turn number keeps a late put-away from hiding a newer round.
- **Readied at the press, shown when due.** gpui starts a window's frames (its display link) only once the system reports the window visible, tens of milliseconds after it is put on screen; ordered front when the list was due, the list's first frame came 40 to 49 ms late at every round. So the window goes on screen at the shortcut's press, at the size it has, clear and letting the pointer through (`sys::ready_panel`), and the appear delay covers the wait: the first frame now comes 1 to 7 ms after the list is due. A round that picks before then puts the clear window away once any fade of the round before is over. Keeping the window on screen between rounds was the other way, and it would run the display link, and wake the app, at every display frame for as long as Tabflip runs.
- **Placed by hand** (`sys::show_panel`): centered on the screen under the pointer, lifted by 8% of the screen's height, on every Space and over full-screen apps, without the system's shadow, and shown with `orderFrontRegardless` so it never takes the keyboard.
- **It never becomes active**, so it opts out of gpui's 30 frames a second for inactive windows (`docs/design.md`, Motion, Traps).

## The menu bar and the Dock

- Tabflip is a menu bar app: `LSUIElement` in the bundle, and `sys::stay_in_menu_bar` at launch, because gpui makes every app a regular one as it starts, over the bundle's own say.
- While Settings are open it is a regular app (`sys::in_dock(true)` in `settings::show`), in the Dock and the app switcher with its own menu (`app_menus`), so the window can be found again once another app covers it. `settings::watch_close` puts it back on `on_window_closed`, never `on_window_should_close`: ⌘W closes the window from code (`remove_window`), which never asks.
- The item and its menu are `status_item.rs`: the item has no menu set on it, or the system would show it for either click; the button sends both clicks to a target, which opens Settings or sets the menu, clicks the button and takes the menu off again once it closes (`docs/design.md`, The menu bar).
- Opening Tabflip again from Finder or Spotlight shows Settings (`on_reopen`).
- **Launch at login** is `SMAppService`'s, read when Settings open or come back to the front, never written to the settings.

## Accessibility

- Asked by the person's press on Allow, never unasked at launch. Settings open on General at launch while it is missing, which is where Allow is.
- **Allow first forgets the entry the system keeps** (`tccutil reset Accessibility <bundle id>`, `sys::forget_trust`), then asks the system's own question (`ax::ask_trust`). The system binds a grant to the app's signature, so an entry made for a copy signed otherwise (an earlier build) shows as on in System Settings and counts for nothing, and switching it off and on changes nothing; only removing it and granting again works. The system asks only once in an app's life, so a later press opens Privacy & Security itself.
- **Whether it is given is whether the tap can be made**, never `AXIsProcessTrusted` alone: a running process reads that once, and after a grant it went on saying no until the app was opened again. A tap is tried only while that flag says yes or after Allow was pressed in this launch, since a tap made unasked has the system ask (`Controller::listen`). It is tried again over a few seconds after Allow, at each change to Accessibility's list (`sys::trust_changes`, a notice that comes a moment before the change is written), and whenever Settings come back to the front (`listen_soon`).

## Languages

The window's language is the app's own `AppleLanguages` in its defaults, the setting System Settings, Language & Region, Applications writes, so the two are one (`sys::own_language`, `sys::set_own_language`). The system's preferred languages are read from the global domain alone, apart from the app's own. The bundle declares its languages (`CFBundleLocalizations`) so the system offers them.

## Dev builds and signing

`scripts/dev-app.sh` builds `target/debug/dev.noindex/Tabflip.app` (bundle id `com.alkalinesoftware.tabflip.dev`), quits a copy already running and opens the new one. macOS keeps the Accessibility grant by signer, so the bundle is signed with the first identity found: `TABFLIP_SIGN_IDENTITY`, a Developer ID, an Apple Development certificate, or "Tabflip dev". An ad-hoc build must be allowed again after every build; `scripts/make-dev-cert.sh` makes the self-signed "Tabflip dev" certificate once so it does not have to be. `.noindex` keeps Spotlight and Launchpad from listing a second Tabflip. The system keeps an app's icon by its bundle and reads it again only when the bundle looks changed, so the script touches the bundle and registers it afresh (`lsregister -f`), or a new icon shows only after a restart. Releases are signed and notarized by whoever holds the Developer ID (`docs/release.md`).
