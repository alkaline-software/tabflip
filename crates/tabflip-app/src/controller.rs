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
use tabflip_core::update;

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
    /// The windows as last looked at, shown at once while a fresh look is
    /// taken, and whether one is being taken outside a round.
    known: Vec<Win>,
    refreshing: bool,
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
    /// Allow was pressed in this launch: the system asks only once in an
    /// app's life, so the next press opens Privacy & Security itself.
    pub asked_trust: bool,
    /// How many times the language changed while running: the window
    /// fades into each new one.
    pub lang_serial: u32,
    /// The section Settings open on: where they were left.
    pub section: &'static str,
    pub update: UpdateView,
}

/// What Settings say of updates: a check under way, and what the last
/// one found.
#[derive(Default)]
pub struct UpdateView {
    pub checking: bool,
    /// A check asked for by the button has come back.
    pub answered: bool,
    /// A newer published version.
    pub available: Option<String>,
    /// The last check could not reach GitHub.
    pub failed: bool,
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
            known: Vec::new(),
            refreshing: false,
            recent: Recent::default(),
            icons: HashMap::new(),
            panel: None,
            listening: false,
            recording: false,
            record_error: None,
            system_switcher_stuck: false,
            asked_trust: false,
            lang_serial: 0,
            section: "general",
            update: UpdateView::default(),
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
        let Some(mut keys) = sys::tap::start() else {
            sys::log(format_args!("listen: the key tap was refused"));
            return;
        };
        sys::log(format_args!("listen: key tap on"));
        self.listening = true;
        self.apply_shortcut();
        self.refresh(cx);
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
        sys::log(format_args!("key {key:?}"));
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
        if let Some(e) = effect {
            sys::log(format_args!("effect {e:?} ({} rows)", self.rows.len()));
        }
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
    /// used, and the delay before the list shows starts. The windows last
    /// looked at are listed at once, those still on screen or put away,
    /// and a fresh look is taken off the main thread to replace them: an
    /// app slow to answer never keeps the list from showing.
    fn begin(&mut self, round: u64, cx: &mut Context<Self>) {
        if let Some(id) = sys::windows::focused() {
            self.recent.touch(id);
        }
        if !self.known.is_empty() {
            let stack = sys::windows::on_screen();
            let known = self.known.iter().filter(|w| w.minimized || w.app_hidden || w.id.is_some_and(|n| stack.contains(&n))).cloned().collect();
            self.listed(round, known, &stack, cx);
        }
        let listing = Self::look(cx);
        cx.spawn(async move |this, cx| {
            let (windows, stack) = listing.await;
            let _ = this.update(cx, |c, cx| {
                c.known = windows.clone();
                c.listed(round, windows, &stack, cx);
            });
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

    /// The windows and the on-screen stack, read off the main thread.
    fn look(cx: &mut Context<Self>) -> Task<(Vec<Win>, Vec<u32>)> {
        let apps = sys::windows::apps();
        cx.background_spawn(async move {
            let started = std::time::Instant::now();
            let stack = sys::windows::on_screen();
            let windows = sys::windows::list(&apps, &stack);
            sys::log(format_args!("looked: {} apps, {} on screen, {} windows in {:?}", apps.len(), stack.len(), windows.len(), started.elapsed()));
            (windows, stack)
        })
    }

    /// A fresh look outside a round, for the next one to show at once.
    fn refresh(&mut self, cx: &mut Context<Self>) {
        if self.refreshing || !self.listening {
            return;
        }
        self.refreshing = true;
        let listing = Self::look(cx);
        cx.spawn(async move |this, cx| {
            let (windows, _) = listing.await;
            let _ = this.update(cx, |c, _| {
                c.refreshing = false;
                c.known = windows;
            });
        })
        .detach();
    }

    fn listed(&mut self, round: u64, windows: Vec<Win>, stack: &[u32], cx: &mut Context<Self>) {
        if round != self.switcher.round() || !self.switcher.is_active() {
            return;
        }
        // The window selected in the list being replaced stays selected.
        let was = self.switcher.selected().and_then(|i| self.windows.get(i)).and_then(|w| w.id);
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
        let now = was.and_then(|id| ordered.iter().position(|w| w.id == Some(id)));
        self.windows = ordered;
        self.rows = rows;
        let e = self.switcher.listed(round, self.windows.len());
        if let Some(i) = now {
            self.switcher.keep(i);
        }
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

    /// Note which window is in front, each time another app comes
    /// forward, and look at the windows again for the next round.
    pub fn app_activated(&mut self, cx: &mut Context<Self>) {
        if let Some(id) = sys::windows::focused() {
            self.recent.touch(id);
        }
        self.refresh(cx);
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
            ("com.apple.Safari", "Safari", "Pull requests · rust-lang/rust"),
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

    /// `TABFLIP_DEMO_LOOP`: a row down, as Tab does.
    pub fn demo_step(&mut self, cx: &mut Context<Self>) {
        let e = self.switcher.step(1);
        self.effect(e, cx);
    }

    pub fn demo_close(&mut self, cx: &mut Context<Self>) {
        let e = self.switcher.cancel();
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

    pub fn set_size(&mut self, size: tabflip_core::look::Size, cx: &mut Context<Self>) {
        self.config.set_size(size);
        self.save();
        cx.notify();
    }

    /// A preset's name or a hex colour; false, and nothing kept, for
    /// anything else.
    pub fn set_highlight(&mut self, text: &str, cx: &mut Context<Self>) -> bool {
        if !self.config.set_highlight(text) {
            return false;
        }
        self.save();
        cx.notify();
        true
    }

    /// Show or hide the item in the menu bar.
    pub fn set_menu_bar_icon(&mut self, on: bool, cx: &mut Context<Self>) {
        self.config.menu_bar_icon = on;
        sys::status_item::set(self.config.menu_bar_swap, on);
        self.save();
        cx.notify();
    }

    /// The item's clicks swapped: the menu on a left click, Settings on
    /// a right one.
    pub fn set_menu_bar_swap(&mut self, on: bool, cx: &mut Context<Self>) {
        self.config.menu_bar_swap = on;
        sys::status_item::set(on, self.config.menu_bar_icon);
        self.save();
        cx.notify();
    }

    /// The window's language set to `choice` (`system`, or a code of
    /// `i18n::CHOICES`), at once: the words, the menu bar's menu, the
    /// app's menu, and the app's own language in the system's eyes.
    pub fn set_language(&mut self, choice: &str, cx: &mut Context<Self>) {
        let choice = if choice == "system" || crate::i18n::CHOICES.iter().any(|(c, _, _)| *c == choice) { choice.to_string() } else { "system".to_string() };
        sys::set_own_language((choice != "system").then_some(choice.as_str()));
        crate::i18n::set(&choice);
        sys::status_item::retitle(&crate::menu_lines());
        cx.set_menus(crate::app_menus());
        self.config.language = choice;
        self.lang_serial += 1;
        self.save();
        cx.notify();
    }

    /// Allow pressed: the system's own question the first time in a
    /// launch, which puts tabflip in the list and offers to open it;
    /// after that, Privacy & Security itself, since the system asks only
    /// once in an app's life.
    pub fn ask_trust(&mut self, cx: &mut Context<Self>) {
        if self.asked_trust {
            sys::open_url(sys::ACCESSIBILITY_SETTINGS_URL);
        } else {
            sys::ax::ask_trust();
            self.asked_trust = true;
        }
        cx.notify();
    }

    pub fn set_check_updates(&mut self, on: bool, cx: &mut Context<Self>) {
        self.config.check_updates = on;
        self.save();
        cx.notify();
        if on {
            self.check_updates(true, cx);
        }
    }

    /// Ask GitHub for the newest release, off the main thread. An
    /// automatic check runs once a day, and only while it is switched on;
    /// one asked for by the button always runs.
    pub fn check_updates(&mut self, automatic: bool, cx: &mut Context<Self>) {
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
        if self.update.checking || (automatic && !(self.config.check_updates && update::due(self.config.last_update_check, now))) {
            return;
        }
        self.update.checking = true;
        self.update.answered = false;
        cx.notify();
        let task = cx.background_executor().spawn(async { update::latest() });
        cx.spawn(async move |this, cx| {
            let found = task.await;
            let _ = this.update(cx, |c, cx| {
                c.update.checking = false;
                c.update.answered = !automatic;
                match found {
                    Ok(latest) => {
                        c.update.failed = false;
                        c.update.available = latest.filter(|v| update::is_newer(v, update::current_version()));
                        c.config.last_update_check = now;
                        c.save();
                    }
                    Err(e) => {
                        sys::log(format_args!("update check: {e}"));
                        c.update.failed = true;
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn save(&self) {
        if let Err(e) = self.config.save() {
            eprintln!("tabflip: could not save settings: {e}");
        }
    }

    pub fn set_shortcut(&mut self, s: Shortcut) {
        self.config.set_shortcut(s);
        self.save();
        self.apply_shortcut();
    }
}
