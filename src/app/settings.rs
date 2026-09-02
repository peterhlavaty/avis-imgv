//! The menu, and the windows it opens.
//!
//! Everything here is the same shape: draw a window, take what it changed, and
//! hand it to whichever part of the viewer holds a copy. The configuration is
//! written out as soon as anything in it moves, so a key changed in the middle
//! of a session survives the end of it.

use eframe::egui::{self, ViewportCommand};

use crate::actions::reveal;
use crate::config::load::Save;
use crate::config::Config;
use crate::formats;
use crate::ui::{keys, legend, notice, placeholders, settings};

use super::about;
use super::conflict;
use super::panels::MenuAction;
use super::stores;
use super::App;

/// Where the manual lives. The README, which is what the program has.
const MANUAL: &str = "https://github.com/hats-np/avis-imgv#readme";

impl App {
    /// Carries out whatever the menu bar was asked for.
    pub(super) fn handle_menu(&mut self, action: MenuAction) {
        let dialog = rfd::FileDialog::new().set_directory(&self.base_path);

        match action {
            MenuAction::OpenFolder => {
                if let Some(folder) = dialog.pick_folder() {
                    self.open_directory(&folder, None);
                }
            }
            MenuAction::OpenFiles => {
                let picked = dialog
                    .add_filter("image", &formats::supported_extensions())
                    .pick_files();

                if let Some(files) = picked {
                    let first = files.first().cloned();
                    if let Some(parent) = first.as_ref().and_then(|f| f.parent()) {
                        self.open_directory(parent, first.as_deref());
                    }
                }
            }
            MenuAction::BinRejected => self.bin_rejected(),
            MenuAction::OpenBin => self.open_bin(),
            MenuAction::EmptyBin => self.ask_to_empty_the_bin(),
            MenuAction::Mode(mode) => self.set_mode(mode),
            MenuAction::AllSettings => self.open_settings(),
            MenuAction::Keyboard => self.keys_visible = true,
            // A deep link to one of the eleven pages rather than a window of
            // its own. The window drew three of the five slideshow fields and
            // omitted the other two; the page draws all five.
            MenuAction::Slideshow => self.open_settings_at("slideshow.seconds_per_image"),
            MenuAction::CheatSheet => {
                self.cheat_sheet_visible = true;
                self.cheat_sheet_opened = true;
            }
            MenuAction::MarksLegend => self.legend_visible = true,
            MenuAction::Placeholders => self.placeholders_visible = true,
            MenuAction::Messages => self.messages_visible = true,
            MenuAction::About => self.about_visible = true,
            MenuAction::OpenConfigFile => self.open_named_file(Config::path(), "configuration"),
            MenuAction::OpenLogFile => self.open_named_file(crate::logging::path(), "log"),
            MenuAction::OpenManual => {
                if !reveal::with_the_system(std::path::Path::new(MANUAL)) {
                    self.notices.fail("Could not open the manual.");
                }
            }
        }
    }

    /// Opens one of the viewer's own files with whatever the system uses.
    fn open_named_file(&mut self, path: Option<std::path::PathBuf>, name: &str) {
        let Some(path) = path else {
            self.notices.fail(format!(
                "There is no configuration directory, so no {name} file."
            ));
            return;
        };

        if !reveal::with_the_system(&path) {
            self.notices.fail(format!(
                "Could not open the {name} file at {}.",
                path.display()
            ));
        }
    }

    /// Draws the keyboard editor and applies whatever it changed.
    ///
    /// The views hold their own copies of the configuration, so a changed key
    /// has to be handed to each of them; writing the file is what makes it
    /// survive the session.
    /// Draws the windows the Help menu opens.
    pub(super) fn show_help_windows(&mut self, ctx: &egui::Context) {
        if self.about_visible {
            about::ui(ctx, &mut self.about_visible, &self.about);
        }

        if self.legend_visible {
            legend::ui(ctx, &mut self.legend_visible);
        }

        if self.placeholders_visible {
            placeholders::ui(ctx, &mut self.placeholders_visible);
        }

        if self.messages_visible {
            match notice::history_window(ctx, &mut self.messages_visible, &mut self.notices) {
                Some(notice::Asked::OpenLog) => self.open_named_file(crate::logging::path(), "log"),
                Some(notice::Asked::Keys) => {
                    self.messages_visible = false;
                    self.keys_visible = true;
                }
                None => {}
            }
        }
    }

    /// Sends whatever fullscreen change a mode asked for.
    pub(super) fn apply_fullscreen(&mut self, ctx: &egui::Context) {
        if let Some(wanted) = self.pending_fullscreen.take() {
            ctx.send_viewport_cmd(ViewportCommand::Fullscreen(wanted));
        }
    }

    /// Draws the keyboard editor and applies whatever it changed.
    ///
    /// The whole editor takes the keyboard, not only an armed row: a key
    /// pressed here is a key being bound, and one the viewer also read would
    /// both rebind and fire. Arming a row and pressing Delete used to send the
    /// photograph on screen to the bin, and the capture failed as well.
    pub(super) fn show_keyboard(&mut self, ctx: &egui::Context) {
        let mut open = self.keys_visible;
        let outcome = keys::show(ctx, &mut open, &mut self.keys, &mut self.settings);
        self.keys_visible = open;

        if outcome.is_none() {
            return;
        }

        self.apply_settings();
        self.save_settings();
    }

    /// Writes the configuration out, telling the user when it could not be.
    ///
    /// The interesting failure is a file that was only partly understood on
    /// the way in: writing it back would replace whatever the viewer could not
    /// read with the defaults that stood in for it, so it is refused, and the
    /// person who just changed a key needs to know their change is not being
    /// kept.
    pub(super) fn save_settings(&mut self) {
        // Every deliberate change to the configuration ends here, which is what
        // makes this the honest place to tell the history to look. Comparing
        // the hundred and eighty registry rows on every frame instead cost ten
        // microseconds of every one of them, measured, for an answer that is
        // "nothing moved" all but a handful of times in a session.
        self.settings_touched = true;

        match self.settings.save() {
            Ok(Save::Written) => {}
            // Somebody has edited the file since it was read. Writing would
            // throw their edit away, so the question is asked instead.
            Ok(Save::Refused) => self.conflict_visible = true,
            Err(e) => {
                tracing::error!("Could not write the configuration: {e}");
                self.notices
                    .say(format!("Could not write the configuration: {e}"));
            }
        }
    }

    /// Draws the question about an edited file and does what it was told.
    pub(super) fn show_conflict(&mut self, ctx: &egui::Context) {
        if !self.conflict_visible {
            return;
        }

        let mut open = true;
        match conflict::ask(ctx, &mut open) {
            conflict::Answer::Waiting => {}
            conflict::Answer::Reread => {
                self.settings = Config::new();
                self.commit_settings();
                self.notices.say("Read the configuration file again.");
            }
            conflict::Answer::Overwrite => {
                if let Err(e) = self.settings.save_over() {
                    tracing::error!("Could not write the configuration: {e}");
                    self.notices
                        .say(format!("Could not write the configuration: {e}"));
                } else {
                    self.notices.say("Wrote over the configuration file.");
                }
            }
        }

        self.conflict_visible = open;
    }

    /// Writes back what the keys have nudged.
    ///
    /// Seven values a key changes for the session and the configuration also
    /// holds: the overlay's corner, how many photographs are side by side,
    /// how many thumbnails are across, what is drawn under them, whether
    /// marking advances, whether the strip of thumbnails is up, and whether
    /// the history panel is. Once the
    /// configuration is authoritative these have to be written, or the next
    /// save from the settings window snaps the view back to whatever the file
    /// still says — and the key's effect is lost at the next launch besides.
    ///
    /// Preferences go to the configuration file; only *position* stays in the
    /// session file, because that is what a session is for.
    pub(super) fn remember_runtime(&mut self) {
        let mut moved = false;

        let corner = self.image_view.overlay_corner();
        if self.settings.image_view.overlay_corner != corner {
            self.settings.image_view.overlay_corner = corner;
            moved = true;
        }

        let opens = self.image_view.opens();
        if self.settings.image_view.opening != opens.at {
            self.settings.image_view.opening = opens.at;
            moved = true;
        }

        let keeping = self.image_view.keeping();
        if self.settings.image_view.keep_zoom != keeping.zoom
            || self.settings.image_view.keep_pan != keeping.pan
        {
            self.settings.image_view.keep_zoom = keeping.zoom;
            self.settings.image_view.keep_pan = keeping.pan;
            moved = true;
        }

        let shown = self.image_view.images_shown();
        if self.settings.image_view.nr_images_shown != shown {
            self.settings.image_view.nr_images_shown = shown;
            moved = true;
        }

        let columns = self.grid_view.columns();
        if self.settings.grid_view.images_per_row != columns {
            self.settings.grid_view.images_per_row = columns;
            moved = true;
        }

        let badges = self.grid_view.badges();
        if self.settings.grid_view.badges != badges {
            self.settings.grid_view.badges = badges.to_string();
            moved = true;
        }

        if self.settings.tags.advance_after_marking != self.advancing {
            self.settings.tags.advance_after_marking = self.advancing;
            self.tag_config.advance_after_marking = self.advancing;
            moved = true;
        }

        if self.settings.grid_view.filmstrip_visible != self.filmstrip_visible {
            self.settings.grid_view.filmstrip_visible = self.filmstrip_visible;
            moved = true;
        }

        if self.settings.history.panel_visible != self.history_panel_visible {
            self.settings.history.panel_visible = self.history_panel_visible;
            moved = true;
        }

        if moved {
            self.save_settings();
        }
    }

    /// Hands the configuration to everything holding a copy of part of it,
    /// and builds the caches again where it has to.
    ///
    /// For the one-shot changes — a file re-read, an import, a reset — where
    /// there is no gesture to wait for the end of.
    pub(super) fn commit_settings(&mut self) {
        self.apply_settings();
        self.rebuild_stores();
    }

    /// Builds both stores again if anything they were built from has moved.
    ///
    /// Seventeen fields at once, and the reason none of them needs a restart:
    /// the two budgets and their two GPU halves, the preload radii, the decode
    /// ceiling, the thumbnail resolution, the camera-thumbnail count, all five
    /// raw settings and the screen profile are read exactly once, when a store
    /// is built. So the way to apply them is to build the store again.
    ///
    /// Called when a gesture *ends* rather than while it moves: a rail on true
    /// per-frame apply would empty and refill the cache sixty times a second.
    /// Both stores compare what they are running on against what the
    /// configuration now says, so a commit that changed something else costs
    /// two comparisons.
    pub(super) fn rebuild_stores(&mut self) {
        let profile: std::sync::Arc<str> =
            std::sync::Arc::from(self.settings.general.output_icc_profile.as_str());

        let images = stores::image_store(
            &self.settings.cache,
            &self.settings.image_view,
            &self.settings.raw,
        );
        let thumbnails = stores::thumbnail_store(&self.settings.cache, &self.settings.grid_view);

        let rebuilt = self
            .image_view
            .rebuild_store(images, std::sync::Arc::clone(&profile));
        let sheet = self.grid_view.rebuild_store(thumbnails, profile);

        if rebuilt || sheet {
            // Said out loud because it is work: everything decoded under the
            // old settings has just been thrown away, and a folder of raws
            // will take a moment to come back.
            self.notices
                .say("Filling the cache again on the new settings.");
        }
    }

    /// Hands the configuration to everything holding a copy of part of it.
    pub(super) fn apply_settings(&mut self) {
        self.pending_theme = Some(self.settings.general.theme == "light");

        // Applied on the next frame because it wants the context, and from
        // the base the styles were at when the viewer started, which is what
        // makes calling it again safe.
        self.pending_text_size = true;
        crate::annotations::sidecar::name_like_adobe(
            self.settings.tags.sidecar_naming == "replacing",
        );
        crate::ui::surface::show_settings_rows(self.settings.menus.settings_rows);
        crate::ui::slider::travels(self.settings.mouse.slider_travel);
        // Before the copies are replaced, because the question is whether the
        // window moved them rather than what they now say.
        self.forced_panel_width |=
            (self.tag_config.panel_width - self.settings.tags.panel_width).abs() > 0.5;
        self.forced_filmstrip_height |=
            (self.grid_view.filmstrip_height() - self.settings.grid_view.filmstrip_height).abs()
                > 0.5;

        self.config = self.settings.general.clone();
        self.tag_config = self.settings.tags.clone();

        // The keyword list, the file it is merged with and how many recent
        // keywords are offered were all read once at startup, so editing a
        // keyword list meant restarting the viewer to see it. The catalogue is
        // a pure function of the configuration; the recent list is in memory
        // and only its length changed, so it is trimmed rather than re-read.
        self.catalog = crate::annotations::Catalog::configured(&self.settings.tags);
        self.recent_tags.set_limit(self.settings.tags.recent_tags);
        self.image_view.set_config(self.settings.image_view.clone());
        self.image_view.set_mouse(self.settings.mouse.clone());
        self.image_view
            .set_backdrop(&self.settings.general.backdrop);
        self.grid_view.set_config(self.settings.grid_view.clone());
        self.grid_view.set_backdrop(&self.settings.general.backdrop);

        // Moving the bin can make the folder already on screen the bin, or
        // stop it being one, and the two menus read this rather than asking
        // per frame.
        let in_the_bin = self.in_the_bin();
        self.image_view.in_the_bin = in_the_bin;
        self.grid_view.in_the_bin = in_the_bin;
        // Shortening it takes effect on the frame it is shortened, rather than
        // at the next deed: a limit that has not bitten yet is a limit nobody
        // can tell is working.
        self.history.set_remember(self.settings.history.remember);

        // The other half of the two the panels need. `remember_runtime` writes
        // the live flag into the file so a key press survives the next launch;
        // without this, that was the only direction, and the tick in the
        // settings window was overwritten on the same frame by the flag it had
        // just been asked to change. "Show the strip" had never worked.
        self.filmstrip_visible = self.settings.grid_view.filmstrip_visible;
        self.history_panel_visible = self.settings.history.panel_visible;
    }
}

impl App {
    /// Draws the settings window and applies whatever it changed.
    ///
    /// One commit model across every row: no OK, no Cancel, no Apply, which is
    /// what the program already does and what Microsoft carves out for a
    /// property inspector. The one qualification is arithmetic — the stores are
    /// pure functions of the configuration, so a rail on true per-frame apply
    /// would rebuild the cache sixty times a second. A row whose effect is a
    /// rebuild waits for the gesture to end.
    pub(super) fn show_settings(&mut self, ctx: &egui::Context) {
        // The one field in the whole window that cannot take effect while the
        // window is open. The pool is spawned once, each thread is expected to
        // exist, and one loader is shared by both views; draining a running
        // pool mid-session is a larger job than this deserves, and pretending
        // otherwise would be worse than saying so.
        self.settings_state.waiting_on_a_restart = (self.settings.cache.decode_threads
            != self.threads_at_start)
            .then_some("decode threads");

        let mut open = self.settings_visible;
        let outcome = settings::show(ctx, &mut open, &mut self.settings_state, &mut self.settings);
        self.settings_visible = open;

        if let Some(page) = self.settings_state.page {
            // Through the registry into `config.json`, not into the session
            // file: `on_exit` returns early when `restore_session` is off, so a
            // preference kept there is one some people silently do not have.
            let remembered = format!("{page:?}");
            if self.settings.general.last_settings_page != remembered {
                self.settings.general.last_settings_page = remembered;
            }
        }

        if let Some(path) = self.settings_state.arm_key.take() {
            self.arm_key(path);
        }

        if let Some(run) = outcome.run {
            self.run_settings_button(run, ctx);
        }

        if outcome.committed {
            self.rebuild_stores();
        }

        if !outcome.changed {
            return;
        }

        self.apply_settings();

        // Written on every gesture, which is what makes the window the thing
        // that decides rather than a form to be submitted.
        self.save_settings();
    }

    /// What a slider's own menu asked for, taken on the frame it was asked.
    ///
    /// A rail is drawn in four subsystems and not one of them has the
    /// configuration in hand, so the menu leaves its ask in `ui::slider` and it
    /// is collected here. Through the same two calls the settings window makes,
    /// so a travel picked from a menu and a travel typed into the window are
    /// the same deed — including in the history, which watches the registry
    /// whenever `save_settings` says something moved.
    pub(super) fn take_slider_ask(&mut self) {
        match crate::ui::slider::asked() {
            Some(crate::ui::slider::Ask::Travel(travel)) => {
                if (self.settings.mouse.slider_travel - travel).abs() < 0.01 {
                    return;
                }

                self.settings.mouse.slider_travel = travel;
                self.apply_settings();
                self.save_settings();

                // Said out loud because the change is only felt on the next
                // drag, and a menu that closes having apparently done nothing
                // is a menu nobody trusts again.
                self.notices
                    .say(if travel <= crate::ui::slider::drag::BOUND {
                        "Sliders follow the pointer.".to_string()
                    } else {
                        format!("The pointer now moves {travel:.0}× the rail to cross a slider.")
                    });
            }
            Some(crate::ui::slider::Ask::Settings) => {
                self.open_settings_at("mouse.slider_travel");
            }
            None => {}
        }
    }

    /// Opens the settings window on the page it was last left on.
    pub(super) fn open_settings(&mut self) {
        if self.settings_state.page.is_none() {
            self.settings_state.page = Some(page_named(&self.settings.general.last_settings_page));
        }

        self.settings_state.problems = self.settings.check();

        // The migration report and the key clashes had six seconds and no way
        // back; here they have a home that does not fade.
        self.settings_state.at_startup = self.startup_notices.clone();
        self.settings_state.just_opened = true;
        self.settings_visible = true;
    }

    /// Opens it on a named row, from a link or a **[Fix]** button.
    pub(super) fn open_settings_at(&mut self, path: &'static str) {
        self.open_settings();

        if let Some(row) = crate::config::registry::row(path) {
            self.settings_state.page = Some(row.page);
            self.settings_state.query.clear();
            self.settings_state.reveal = Some(path);
        }
    }

    fn run_settings_button(&mut self, run: settings::Run, ctx: &egui::Context) {
        if run == settings::Run::Restart {
            self.session.save();
            restart();
            return;
        }

        match crate::ui::settings::footer::carry_out(run, &self.settings) {
            Ok(said) if said.is_empty() => {}
            Ok(said) => self.notices.say(said),
            Err(problem) if problem.is_empty() => self.import_settings(ctx),
            Err(problem) => self.notices.fail(problem),
        }
    }

    /// Reads a saved settings file over this one.
    fn import_settings(&mut self, _ctx: &egui::Context) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("settings", &["json"])
            .pick_file()
        else {
            return;
        };

        let Ok(text) = std::fs::read_to_string(&path) else {
            self.notices
                .fail(format!("Could not read {}.", path.display()));
            return;
        };

        // Merged into what is held rather than replacing it: a bundle is a
        // patch, and the fields it does not name are the fields it does not
        // change.
        let merged = match merge_bundle(&self.settings, &text) {
            Ok(merged) => merged,
            Err(problem) => {
                self.notices.fail(problem);
                return;
            }
        };

        let named = merged.1;
        self.settings = merged.0;
        self.commit_settings();
        self.save_settings();
        self.notices
            .say(format!("Read {named} setting(s) from {}.", path.display()));
    }
}

/// Applies a bundle over a configuration, reporting how many fields it named.
fn merge_bundle(config: &Config, text: &str) -> Result<(Config, usize), String> {
    let bundle: serde_json::Map<String, serde_json::Value> =
        serde_json::from_str(text).map_err(|e| format!("That is not a settings file: {e}"))?;

    let mut document = serde_json::to_value(config)
        .map_err(|e| format!("Could not read what is held: {e}"))?
        .as_object()
        .cloned()
        .ok_or_else(|| "Could not read what is held.".to_string())?;

    let mut named = 0;

    for (section, values) in &bundle {
        let (Some(values), Some(into)) = (
            values.as_object(),
            document.get_mut(section).and_then(|s| s.as_object_mut()),
        ) else {
            continue;
        };

        for (key, value) in values {
            into.insert(key.clone(), value.clone());
            named += 1;
        }
    }

    let json = serde_json::Value::Object(document).to_string();
    let mut merged = Config::from_json(&json);

    // Whatever the bundle did not name stays as it was, including the document
    // this configuration was read from — an import is not a reason to drop
    // somebody's unknown keys.
    merged.document = config.document.clone();

    Ok((merged, named))
}

/// Which page a remembered name is.
fn page_named(name: &str) -> crate::config::registry::Page {
    crate::config::registry::Page::ALL
        .iter()
        .copied()
        .find(|page| format!("{page:?}") == name)
        .unwrap_or(crate::config::registry::Page::OpeningAFolder)
}

/// Saves the session and starts the viewer again.
///
/// A button that does the thing rather than a toast telling somebody to do it:
/// darktable's own fix for this complaint was a toast on closing the dialogue.
fn restart() {
    let Ok(exe) = std::env::current_exe() else {
        return;
    };

    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Err(e) = std::process::Command::new(exe).args(args).spawn() {
        tracing::error!("Could not start the viewer again: {e}");
        return;
    }

    std::process::exit(0);
}
