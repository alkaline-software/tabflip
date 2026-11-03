# Probing

How to look at a state of the list or of Settings from a script, without the person's hands, and how to measure what a change costs.

## Rules

- Probe with a copy of its own, never the copy the person is using: `TABFLIP_HOME` at a scratch folder. A copy with its own home ends no other Tabflip at launch (`docs/platform.md`, One Tabflip at a time).
- None of the probes needs a permission: a process may take a picture of its own windows without Screen Recording access, and the list's sample rows need no Accessibility.
- Say what was checked and what was not. A synthetic click does not reach a click handler in a background window, so a picture of the state a click leads to does not show the click works. A key the person presses goes through the tap, which a probe copy without Accessibility does not have.

## A copy

```
cargo build -p tabflip-app
TABFLIP_HOME=<scratch>/home TABFLIP_SHOT=<panel.png> target/debug/tabflip
TABFLIP_HOME=<scratch>/home TABFLIP_SETTINGS=customize TABFLIP_GO="dark" TABFLIP_SHOT_SETTINGS=<settings.png> target/debug/tabflip
```

| Variable | What it does |
|---|---|
| `TABFLIP_HOME` | the copy's own settings |
| `TABFLIP_SHOT` | the list with sample rows, written to the file once it has come in, then quit |
| `TABFLIP_SHOT_SETTINGS` | Settings, written to the file after the steps, then quit |
| `TABFLIP_SETTINGS` | the section Settings open on: `general`, `customize`, `about` |
| `TABFLIP_GO` | steps before the picture, `;` between, a moment apart (`settings::go`) |
| `TABFLIP_APPEARANCE` | `light` or `dark` for the list, in place of the system's |
| `TABFLIP_LANG` | a language's code (`fr`, `de`, `zh-Hans`), in place of the system's |
| `TABFLIP_DEMO_LOOP` | a folder: the list shown and put away four times, as rounds are, two rows down each round, writing `demo0.png`… there |
| `TABFLIP_FRAME_LOG` | each frame of the highlight's slide written to the dev log with its time |

`TABFLIP_GO` steps: `dark`, `light`; `pick:language` (the language's list open); `language:<code>`; `section:<key>`; `untrusted` and `asked` (Accessibility not given, before and after one Allow); `record` (the shortcut's field waiting for keys); `newer` (a newer version found: About's dot and the Update button; nothing is fetched); `downloading` (its download under way).

A bug that shows only from the second round on (a resize gpui never heard of) shows in `TABFLIP_DEMO_LOOP` and not in `TABFLIP_SHOT`. With `TABFLIP_FRAME_LOG`, the frames of a slide are 16.7 ms apart on a 60 Hz display.

## The windows from a terminal

```
target/debug/tabflip --list
```

prints the windows in the switcher's order with their labels, window numbers and full titles. The terminal running it needs Accessibility, or the titles are missing. Debug builds log what each round did and how long its look took to `~/Library/Logs/tabflip-dev.log`.

## Measuring

A speed-up starts from a number and ends with another. The stopwatch goes in a copy and comes out when it is done; none is kept.

- **At rest:** `top -pid <pid> -stats cpu,idlew,mem`, with Settings closed and open. Tabflip at rest is at about 0% with a few wakeups a second.
- **A frame:** the time from the top of `render` to the next frame (`window.on_next_frame`). A hover draws the window whole, so whatever `render` calls costs that much at every hover. A frame has about 16 ms; a frame that came back at 16 ms waited for the display, not for the app.
- **A wait outside a frame:** a timer of 5 ms on the main thread that prints any gap over 30 ms. It finds a slow question to the system that no frame shows.
- **A round:** the dev log's `looked:` line says how many apps were asked and how long the look took; the list shows from the windows last seen while it runs.
- **Nothing lost:** what the switcher lists (`--list`) is compared with the old build's, kept beside the new one.
