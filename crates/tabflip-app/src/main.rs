//! tabflip: switch between windows, not apps.
//!
//! A menu bar app: an item in the menu bar and no Dock icon, except
//! while Settings are open. Hold the shortcut (⌘Tab by default) and a list
//! of every window shows; let go and the selected one comes to the front.
//! `tabflip --list` prints the windows as the switcher would show them.

mod assets;
mod controller;
mod i18n;
mod panel;
mod settings;
mod sys;
mod theme;
mod ui;

use gpui::*;

actions!(tabflip, [Quit, Hide, HideOthers, ShowAll, CloseWindow, OpenSettings, Escape]);

/// The app's menu at the screen's top, there while Settings are open and
/// tabflip is in the Dock: the Mac's own choices for an app, in the
/// language in use.
pub fn app_menus() -> Vec<Menu> {
    let t = i18n::t;
    vec![Menu {
        name: "tabflip".into(),
        disabled: false,
        items: vec![
            MenuItem::action(t("Settings…"), OpenSettings),
            MenuItem::separator(),
            MenuItem::action(t("Hide tabflip"), Hide),
            MenuItem::action(t("Hide Others"), HideOthers),
            MenuItem::action(t("Show All"), ShowAll),
            MenuItem::separator(),
            MenuItem::action(t("Quit tabflip"), Quit),
        ],
    }]
}

/// The menu bar item's menu: each section of Settings with the rail's
/// icon, a line before Updates, and Quit under a line of its own.
pub fn menu_lines() -> Vec<sys::status_item::MenuLine> {
    let icon = |path: &str| match assets::Assets.load(path) {
        Ok(Some(std::borrow::Cow::Borrowed(bytes))) => bytes,
        _ => &[],
    };
    let mut lines: Vec<sys::status_item::MenuLine> = settings::SECTIONS.iter().map(|&(key, title, path)| sys::status_item::MenuLine { key, title: i18n::t(title), icon: icon(path), keys: "", gap: key == "updates" }).collect();
    lines.push(sys::status_item::MenuLine { key: "quit", title: i18n::t("Quit tabflip"), icon: icon("icons/logout.svg"), keys: "q", gap: true });
    lines
}

fn main() {
    // The system's ⌘Tab is put back on before anything else: a tabflip
    // that crashed or was killed with it off leaves it off.
    sys::private::set_system_switcher(true);
    restore_on_exit();
    sys::ax::limit_waits();
    sys::log(format_args!("start: {}, Accessibility {}", std::env::current_exe().map(|p| p.display().to_string()).unwrap_or_default(), sys::ax::trusted()));

    if std::env::args().any(|a| a == "--list") {
        list();
        return;
    }
    // `tabflip --try-update <bundle>`: the newest release's disk image
    // fetched and swapped into that bundle, a throwaway copy, with
    // nothing opened: the update's steps short of the relaunch.
    if let Some(bundle) = std::env::args().skip_while(|a| a != "--try-update").nth(1) {
        try_update(&std::path::PathBuf::from(bundle));
        return;
    }

    let app = gpui_platform::application().with_assets(assets::Assets);
    // Opened again from Finder or Spotlight while running: Settings, which
    // is the way back to them with the menu bar item hidden.
    app.on_reopen(|cx| {
        if cx.try_global::<controller::Global_>().is_some() {
            settings::show(None, cx);
        }
    });
    app.run(|cx: &mut App| {
        gpui_component::init(cx);
        i18n::set(&i18n::chosen());
        sys::stay_in_menu_bar();
        // One tabflip at a time, the one opened last: two would each take
        // the shortcut. A probe copy, with a home of its own, leaves the
        // others be.
        let ended_others = std::env::var_os("TABFLIP_HOME").is_none() && sys::end_other_tabflips();
        theme::install(cx);
        let controller = cx.new(|_| controller::Controller::new());
        cx.set_global(controller::Global_(controller.clone()));
        if ended_others {
            // A copy that quits gives the system its ⌘Tab back, after this
            // one may have taken it: taken again once they are gone.
            let c = controller.clone();
            cx.spawn(async move |cx| {
                cx.background_executor().timer(std::time::Duration::from_millis(3500)).await;
                let _ = c.update(cx, |c, _| {
                    if c.listening {
                        c.apply_shortcut();
                    }
                });
            })
            .detach();
        }
        panel::install(cx);
        settings::watch_close(cx);

        cx.on_action(|_: &Quit, cx| cx.quit());
        cx.on_action(|_: &Hide, cx| cx.hide());
        cx.on_action(|_: &HideOthers, cx| cx.hide_other_apps());
        cx.on_action(|_: &ShowAll, cx| cx.unhide_other_apps());
        cx.on_action(|_: &OpenSettings, cx| settings::show(None, cx));
        cx.on_action(|_: &CloseWindow, cx| settings::close(cx));
        cx.bind_keys([
            KeyBinding::new("cmd-q", Quit, None),
            KeyBinding::new("cmd-h", Hide, None),
            KeyBinding::new("cmd-alt-h", HideOthers, None),
            KeyBinding::new("cmd-w", CloseWindow, None),
            KeyBinding::new("cmd-,", OpenSettings, None),
            KeyBinding::new("escape", Escape, Some(settings::KEY_CONTEXT)),
        ]);
        cx.set_menus(app_menus());

        cx.on_app_quit(|_| {
            sys::private::set_system_switcher(true);
            async {}
        })
        .detach();

        // The menu bar item: one click opens Settings, the other shows
        // its menu, of the sections and Quit.
        let (swap, shown) = {
            let c = controller.read(cx);
            (c.config.menu_bar_swap, c.config.menu_bar_icon)
        };
        if let Some(mut picks) = sys::status_item::install(&menu_lines(), swap, shown) {
            cx.spawn(async move |cx| {
                use futures::StreamExt as _;
                while let Some(pick) = picks.next().await {
                    cx.update(|cx| match pick {
                        sys::status_item::MenuPick::Settings => settings::show(None, cx),
                        sys::status_item::MenuPick::Line("quit") => cx.quit(),
                        sys::status_item::MenuPick::Line(section) => settings::show(Some(section), cx),
                    });
                }
            })
            .detach();
        }

        // Which window was in front, each time another app comes forward.
        let mut activations = sys::activations();
        let c = controller.clone();
        cx.spawn(async move |cx| {
            use futures::StreamExt as _;
            while activations.next().await.is_some() {
                c.update(cx, |c, cx| c.app_activated(cx));
            }
        })
        .detach();

        if let Some(path) = std::env::var_os("TABFLIP_SHOT_SETTINGS") {
            // `TABFLIP_SETTINGS=<section>` picks the section shot, and
            // `TABFLIP_GO="step;step"` plays steps first (`settings::go`),
            // a moment apart.
            let section = std::env::var("TABFLIP_SETTINGS").ok().and_then(|s| settings::SECTIONS.iter().find(|(k, _, _)| *k == s).map(|(k, _, _)| *k));
            let steps: Vec<String> = std::env::var("TABFLIP_GO").map(|s| s.split(';').filter(|s| !s.is_empty()).map(str::to_string).collect()).unwrap_or_default();
            controller.update(cx, |c, _| c.listening = sys::ax::trusted());
            settings::show(section, cx);
            cx.spawn(async move |cx| {
                for step in steps {
                    cx.background_executor().timer(std::time::Duration::from_millis(400)).await;
                    cx.update(|cx| settings::go(&step, cx));
                }
                cx.background_executor().timer(std::time::Duration::from_millis(800)).await;
                cx.update(|cx| {
                    settings::shoot(&std::path::PathBuf::from(path), cx);
                    cx.quit();
                });
            })
            .detach();
            return;
        }
        if let Some(dir) = std::env::var_os("TABFLIP_DEMO_LOOP") {
            let dir = std::path::PathBuf::from(dir);
            cx.spawn(async move |cx| {
                for n in 0..4 {
                    cx.background_executor().timer(std::time::Duration::from_secs(3)).await;
                    sys::log(format_args!("demo {n}: show"));
                    cx.update(|cx| controller.update(cx, |c, cx| c.demo(cx)));
                    // Two rows down, a moment apart, as Tab does: the slide.
                    for _ in 0..2 {
                        cx.background_executor().timer(std::time::Duration::from_millis(400)).await;
                        cx.update(|cx| controller.update(cx, |c, cx| c.demo_step(cx)));
                    }
                    cx.background_executor().timer(std::time::Duration::from_millis(600)).await;
                    let path = dir.join(format!("demo{n}.png"));
                    cx.update(|cx| {
                        if let Some(panel) = controller.read(cx).panel() {
                            let _ = panel.update(cx, |_, window, _| sys::log(format_args!("demo {n}: shot {:?}", sys::shoot_window(window, &path))));
                        }
                        controller.update(cx, |c, cx| c.demo_close(cx));
                    });
                }
                cx.update(|cx| cx.quit());
            })
            .detach();
            return;
        }
        if let Some(path) = std::env::var_os("TABFLIP_SHOT") {
            shot(controller, path.into(), cx);
            return;
        }

        controller.update(cx, |c, cx| c.listen(cx));
        // A newer release looked for once a day, while that is on.
        let c = controller.clone();
        cx.spawn(async move |cx| loop {
            c.update(cx, |c, cx| c.check_updates(true, cx));
            cx.background_executor().timer(std::time::Duration::from_secs(60 * 60)).await;
        })
        .detach();
        if !controller.read(cx).listening {
            // Settings, on the section where Accessibility is given: Allow
            // there asks the system's own question, and the list is
            // watched for the answer.
            settings::show(Some("general"), cx);
            let mut changes = sys::trust_changes();
            let c = controller.clone();
            cx.spawn(async move |cx| {
                use futures::StreamExt as _;
                while changes.next().await.is_some() {
                    if c.update(cx, |c, cx| {
                        c.listen_soon(cx);
                        c.listening
                    }) {
                        return;
                    }
                }
            })
            .detach();
        }
    });
}

/// The system's ⌘Tab put back when tabflip is ended by a signal or a
/// panic, as at quit.
fn restore_on_exit() {
    extern "C" fn on_signal(sig: i32) {
        sys::private::set_system_switcher(true);
        extern "C" {
            fn _exit(code: i32) -> !;
        }
        unsafe { _exit(128 + sig) }
    }
    extern "C" {
        fn signal(sig: i32, handler: extern "C" fn(i32)) -> usize;
    }
    const SIGHUP: i32 = 1;
    const SIGINT: i32 = 2;
    const SIGTERM: i32 = 15;
    for sig in [SIGHUP, SIGINT, SIGTERM] {
        unsafe { signal(sig, on_signal) };
    }
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        sys::private::set_system_switcher(true);
        previous(info);
    }));
}

/// `TABFLIP_SHOT=<file.png>`: the panel shown with sample rows, written
/// to the file once it has come in, and the app quits.
fn shot(controller: Entity<controller::Controller>, path: std::path::PathBuf, cx: &mut App) {
    controller.update(cx, |c, cx| c.demo(cx));
    cx.spawn(async move |cx| {
        cx.background_executor().timer(std::time::Duration::from_millis(800)).await;
        cx.update(|cx| {
            if let Some(panel) = controller.read(cx).panel() {
                let _ = panel.update(cx, |_, window, _| match sys::shoot_window(window, &path) {
                    Ok(()) => println!("{}", path.display()),
                    Err(e) => eprintln!("tabflip: {e}"),
                });
            }
            cx.quit();
        });
    })
    .detach();
}

/// `tabflip --try-update <bundle>`: the update done to a throwaway bundle.
fn try_update(bundle: &std::path::Path) {
    use tabflip_core::update;
    let latest = match update::latest() {
        Ok(Some(v)) => v,
        Ok(None) => return eprintln!("tabflip: nothing is published"),
        Err(e) => return eprintln!("tabflip: {e}"),
    };
    println!("latest {latest}");
    let file = match update::download(&latest, &|done, total| println!("  {done} of {}", total.map(|t| t.to_string()).unwrap_or("?".into()))) {
        Ok(f) => f,
        Err(e) => return eprintln!("tabflip: {e}"),
    };
    println!("downloaded {}", file.display());
    match update::install_into(&file, bundle, false) {
        Ok(()) => println!("swapped into {}", bundle.display()),
        Err(e) => eprintln!("tabflip: {e}"),
    }
}

/// `tabflip --list`: the windows in the switcher's order, one a line.
fn list() {
    use tabflip_core::label::{labels, Titled};
    use tabflip_core::order::{order, Recent};
    if !sys::ax::trusted() {
        eprintln!("tabflip: Accessibility is not granted to this terminal; titles will be missing");
    }
    let apps = sys::windows::apps();
    let stack = sys::windows::on_screen();
    let windows = sys::windows::list(&apps, &stack);
    let ix = order(&windows, |w| w.id, &stack, &Recent::default());
    let ordered: Vec<_> = ix.into_iter().map(|i| &windows[i]).collect();
    let titled: Vec<Titled> = ordered.iter().map(|w| Titled { app_name: &w.app_name, bundle_id: &w.bundle_id, title: &w.title }).collect();
    for (w, label) in ordered.iter().zip(labels(&titled)) {
        let mark = if w.minimized { " (minimized)" } else if w.app_hidden { " (hidden)" } else { "" };
        println!("{label:<26} {:<18} {:>6}  {}{mark}", w.app_name, w.id.map(|n| n.to_string()).unwrap_or_default(), w.title);
    }
}
