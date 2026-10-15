//! The switcher at work: keys in from the tap, the core's states, and what
//! each effect does on the Mac. One per app, held as a global entity that
//! the panel and Settings both read.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use gpui::*;
use tabflip_core::config::Config;
use tabflip_core::label::{self, Titled};
use tabflip_core::order::{self, Recent};
use tabflip_core::shortcut::{Invalid, Shortcut};
use tabflip_core::switcher::{Effect, Switcher};

use crate::panel::Panel;
use crate::sys::{self, tap::Key, windows::Window as Win};

/// One row of the list.
#[derive(Clone)]
pub struct Row {
    pub label: SharedString,
    pub app: SharedString,
    pub icon: Option<Arc<Image>>,
}

pub struct Controller {
    pub config: Config,
    switcher: Switcher,
    windows: Vec<Win>,
    pub rows: Vec<Row>,
    recent: Recent,
    icons: HashMap<i32, Option<Arc<Image>>>,
    panel: Option<WindowHandle<Panel>>,
    /// The tap is running: Accessibility is granted and the keys are seen.
    pub listening: bool,
    /// Settings is waiting for a shortcut, and what was wrong with the last
    /// one pressed.
    pub recording: bool,
    pub record_error: Option<Invalid>,
    /// The system's ⌘Tab could not be switched off.
    pub system_switcher_stuck: bool,
}

pub struct Global_(pub Entity<Controller>);

impl Global for Global_ {}

pub fn get(cx: &App) -> Entity<Controller> {
    cx.global::<Global_>().0.clone()
}

impl Controller {
    pub fn new() -> Self {
        let config = Config::load();
        sys::tap::set_shortcut(config.shortcut());
        Self {
            config,
            switcher: Switcher::default(),
            windows: Vec::new(),
            rows: Vec::new(),
            recent: Recent::default(),
            icons: HashMap::new(),
            panel: None,
            listening: false,
            recording: false,
            record_error: None,
            system_switcher_stuck: false,
        }
    }

    pub fn selected(&self) -> Option<usize> {
        self.switcher.selected()
    }

    pub fn set_panel(&mut self, panel: WindowHandle<Panel>) {
        self.panel = Some(panel);
    }

    /// Start seeing the keyboard, once Accessibility allows it. Keys are
    /// handled as they come, for the app's life.
    pub fn listen(&mut self, cx: &mut Context<Self>) {
        if self.listening || !sys::ax::trusted() {
            return;
        }
        let Some(mut keys) = sys::tap::start() else { return };
        self.listening = true;
        self.apply_shortcut();
        cx.notify();
        cx.spawn(async move |this, cx| {
            use futures::StreamExt as _;
            while let Some(key) = keys.next().await {
                if this.update(cx, |c, cx| c.key(key, cx)).is_err() {
                    break;
                }
            }
        })
        .detach();
    }

    /// The system's ⌘Tab off while it is the shortcut, on otherwise.
    fn apply_shortcut(&mut self) {
        let s = self.config.shortcut();
        sys::tap::set_shortcut(s);
        let ok = sys::private::set_system_switcher(!s.is_command_tab());
        self.system_switcher_stuck = s.is_command_tab() && !ok;
    }

    fn key(&mut self, key: Key, cx: &mut Context<Self>) {
        let effect = match key {
            Key::Trigger { back } => self.switcher.trigger(back),
            Key::Step(by) => self.switcher.step(by),
            Key::Release => self.switcher.release(),
            Key::Escape => self.switcher.cancel(),
            Key::Recorded { flags, key } => {
                self.recorded(flags, key, cx);
                None
            }
        };
        self.effect(effect, cx);
    }

    pub fn hover(&mut self, i: usize, cx: &mut Context<Self>) {
        let e = self.switcher.hover(i);
        self.effect(e, cx);
    }

    pub fn click(&mut self, i: usize, cx: &mut Context<Self>) {
        let e = self.switcher.click(i);
        self.effect(e, cx);
    }

    fn effect(&mut self, effect: Option<Effect>, cx: &mut Context<Self>) {
        match effect {
            None => {}
            Some(Effect::Begin(round)) => self.begin(round, cx),
            Some(Effect::Show) => self.show(cx),
            Some(Effect::Redraw) => cx.notify(),
            Some(Effect::Focus(i)) => {
                self.close(cx);
                if let Some(w) = self.windows.get(i).cloned() {
                    if let Some(id) = w.id {
                        self.recent.touch(id);
                    }
                    cx.background_spawn(async move { sys::focus::focus(&w) }).detach();
                }
            }
            Some(Effect::Close) => self.close(cx),
        }
    }

    /// A round begins: the window in front is remembered as the latest
    /// used, the windows are listed off the main thread, and the delay
    /// before the list shows starts.
    fn begin(&mut self, round: u64, cx: &mut Context<Self>) {
        if let Some(id) = sys::windows::focused() {
            self.recent.touch(id);
        }
        let apps = sys::windows::apps();
        let listing = cx.background_spawn(async move {
            let stack = sys::windows::on_screen();
            let windows = sys::windows::list(&apps, &stack);
            (windows, stack)
        });
        cx.spawn(async move |this, cx| {
            let (windows, stack) = listing.await;
            let _ = this.update(cx, |c, cx| c.listed(round, windows, &stack, cx));
        })
        .detach();
        let delay = Duration::from_millis(self.config.appear_delay_ms());
        cx.spawn(async move |this, cx| {
            cx.background_executor().timer(delay).await;
            let _ = this.update(cx, |c, cx| {
                let e = c.switcher.delay_passed(round);
                c.effect(e, cx);
            });
        })
        .detach();
    }

    fn listed(&mut self, round: u64, windows: Vec<Win>, stack: &[u32], cx: &mut Context<Self>) {
        if round != self.switcher.round() {
            return;
        }
        let ix = order::order(&windows, |w| w.id, stack, &self.recent);
        let mut ordered: Vec<Win> = ix.into_iter().map(|i| windows[i].clone()).collect();
        // A window an app lists twice is one row.
        let mut seen = std::collections::HashSet::new();
        ordered.retain(|w| w.id.is_none_or(|id| seen.insert(id)));
        let titled: Vec<Titled> = ordered.iter().map(|w| Titled { app_name: &w.app_name, bundle_id: &w.bundle_id, title: &w.title }).collect();
        let labels = label::labels(&titled);
        let rows = ordered
            .iter()
            .zip(labels)
            .map(|(w, label)| Row { label: label.into(), app: w.app_name.clone().into(), icon: self.icon(w.pid) })
            .collect();
        self.windows = ordered;
        self.rows = rows;
        let e = self.switcher.listed(round, self.windows.len());
        self.effect(e, cx);
    }

    fn icon(&mut self, pid: i32) -> Option<Arc<Image>> {
        self.icons.entry(pid).or_insert_with(|| sys::windows::icon_png(pid, 32.).map(|png| Arc::new(Image::from_bytes(ImageFormat::Png, png)))).clone()
    }

    fn show(&mut self, cx: &mut Context<Self>) {
        cx.notify();
        let Some(panel) = self.panel else { return };
        let rows = self.rows.len();
        // Deferred, as is putting it away: a click on the panel arrives
        // while its window is busy with the click.
        cx.defer(move |cx| {
            let _ = panel.update(cx, |p, window, cx| p.appear(rows, window, cx));
        });
    }

    fn close(&mut self, cx: &mut Context<Self>) {
        sys::tap::closed();
        cx.notify();
        if let Some(panel) = self.panel {
            cx.defer(move |cx| {
                let _ = panel.update(cx, |p, window, cx| p.leave(window, cx));
            });
        }
    }

    /// Note which window is in front, each time another app comes forward.
    pub fn app_activated(&mut self) {
        if let Some(id) = sys::windows::focused() {
            self.recent.touch(id);
        }
    }

    /// `TABFLIP_SHOT`: the list as it shows for a few windows, with no
    /// Accessibility needed, for a picture of the panel.
    pub fn demo(&mut self, cx: &mut Context<Self>) {
        const SAMPLES: [(&str, &str, &str); 6] = [
            ("com.google.Chrome", "Google Chrome", "Inbox (3) - john@gmail.com - Gmail"),
            ("com.google.Chrome", "Google Chrome", "(2) Facebook"),
            ("dev.zed.Zed", "Zed", "tabflip — main.rs"),
            ("com.apple.finder", "Finder", "Downloads"),
            ("com.apple.Terminal", "Terminal", "jhelvy — -zsh — 80×24"),
            ("com.apple.Safari", "Safari", "Pull requests · alkaline-software/emaki"),
        ];
        let apps = sys::windows::apps();
        let titled: Vec<Titled> = SAMPLES.iter().map(|(bundle_id, app_name, title)| Titled { app_name, bundle_id, title }).collect();
        let labels = label::labels(&titled);
        self.rows = SAMPLES
            .iter()
            .zip(labels)
            .map(|((bundle, app, _), label)| {
                let icon = match apps.iter().find(|a| a.bundle_id == *bundle) {
                    Some(a) => self.icon(a.pid),
                    None => sys::windows::bundle_icon_png(bundle, 32.).map(|png| Arc::new(Image::from_bytes(ImageFormat::Png, png))),
                };
                Row { label: label.into(), app: (*app).into(), icon }
            })
            .collect();
        let round = self.switcher.round() + 1;
        self.switcher.trigger(false);
        self.switcher.listed(round, self.rows.len());
        let e = self.switcher.delay_passed(round);
        self.effect(e, cx);
    }

    pub fn panel(&self) -> Option<WindowHandle<Panel>> {
        self.panel
    }

    // Settings.

    pub fn start_recording(&mut self, cx: &mut Context<Self>) {
        if !self.listening {
            return;
        }
        self.recording = true;
        self.record_error = None;
        sys::tap::set_recording(true);
        cx.notify();
    }

    pub fn stop_recording(&mut self, cx: &mut Context<Self>) {
        self.recording = false;
        self.record_error = None;
        sys::tap::set_recording(false);
        cx.notify();
    }

    fn recorded(&mut self, flags: u64, key: u16, cx: &mut Context<Self>) {
        if !self.recording {
            return;
        }
        // Esc alone gives up.
        if key == tabflip_core::shortcut::ESCAPE && flags == 0 {
            self.stop_recording(cx);
            return;
        }
        match Shortcut::new(flags, key) {
            Ok(s) => {
                self.set_shortcut(s);
                self.stop_recording(cx);
            }
            Err(e) => {
                self.record_error = Some(e);
                cx.notify();
            }
        }
    }

    pub fn set_shortcut(&mut self, s: Shortcut) {
        self.config.set_shortcut(s);
        if let Err(e) = self.config.save() {
            eprintln!("tabflip: could not save settings: {e}");
        }
        self.apply_shortcut();
    }
}
