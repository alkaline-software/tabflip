//! tabflip: switch between windows, not apps.
//!
//! An agent app (no Dock icon) with an item in the menu bar. Hold the
//! shortcut (⌘Tab by default) and a list of every window shows; let go
//! and the selected one comes to the front. `tabflip --list` prints the
//! windows as the switcher would show them.

mod controller;
mod panel;
mod settings;
mod sys;

use gpui::*;

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

    gpui_platform::application().run(|cx: &mut App| {
        gpui_component::init(cx);
        let controller = cx.new(|_| controller::Controller::new());
        cx.set_global(controller::Global_(controller.clone()));
        panel::install(cx);

        cx.on_app_quit(|_| {
            sys::private::set_system_switcher(true);
            async {}
        })
        .detach();

        if let Some(mut menu) = sys::status_item::install() {
            cx.spawn(async move |cx| {
                use futures::StreamExt as _;
                while let Some(command) = menu.next().await {
                    cx.update(|cx| match command {
                        sys::status_item::Command::Settings => settings::show(cx),
                        sys::status_item::Command::Quit => cx.quit(),
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
            settings::show(cx);
            cx.spawn(async move |cx| {
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
        if !controller.read(cx).listening {
            // The system's own prompt the first time, and Settings, which
            // says what to do and notices when it is done.
            sys::ax::ask_trust();
            settings::show(cx);
            let mut changes = sys::trust_changes();
            let c = controller.clone();
            cx.spawn(async move |cx| {
                use futures::StreamExt as _;
                while changes.next().await.is_some() {
                    // The list's notice comes a moment before the answer
                    // to "am I trusted" changes.
                    for _ in 0..10 {
                        cx.background_executor().timer(std::time::Duration::from_millis(250)).await;
                        if c.update(cx, |c, cx| {
                            c.listen(cx);
                            c.listening
                        }) {
                            return;
                        }
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
