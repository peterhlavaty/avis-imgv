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
            notice::history_window(ctx, &mut self.messages_visible, &mut self.notices);
        }
    }

    /// Sends whatever fullscreen change a mode asked for.
    pub(super) fn apply_fullscreen(&mut self, ctx: &egui::Context) {
        if let Some(wanted) = self.pending_fullscreen.take() {
            ctx.send_viewport_cmd(ViewportCommand::Fullscreen(wanted));
        }
    }

    pub(super) fn show_keyboard(&mut self, ctx: &egui::Context) {
        let mut open = self.keys_visible;
        let outcome = keys::show(ctx, &mut open, &mut self.keys, &mut self.settings);
        self.keys_visible = open;

        // While a row is armed the viewer has to stop reading its own keys, or
        // the key being captured also does whatever it does the rest of the
        // time: arming a row and pressing Delete sent the photograph on screen
        // to the bin, and the capture failed as well. Only ever cleared by
        // whoever set it, so this does not un-mute a question that is up.
        if self.keys.is_listening() {
            crate::utils::set_mute_state(ctx, true);
            self.muted_for_keys = true;
        } else if std::mem::take(&mut self.muted_for_keys) {
            crate::utils::set_mute_state(ctx, false);
        }

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
                self.apply_settings();
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

    /// Hands the configuration to everything holding a copy of part of it.
    pub(super) fn apply_settings(&mut self) {
        self.pending_theme = Some(self.settings.general.theme == "light");
        crate::annotations::sidecar::name_like_adobe(
            self.settings.tags.sidecar_naming == "replacing",
        );
        self.config = self.settings.general.clone();
        self.tag_config = self.settings.tags.clone();
        self.image_view.set_config(self.settings.image_view.clone());
        self.image_view
            .set_backdrop(&self.settings.general.backdrop);
        self.grid_view.set_config(self.settings.grid_view.clone());
        self.grid_view.set_backdrop(&self.settings.general.backdrop);
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

        if let Some(run) = outcome.run {
            self.run_settings_button(run, ctx);
        }

        if !outcome.changed {
            return;
        }

        self.apply_settings();

        // Written on every gesture, which is what makes the window the thing
        // that decides rather than a form to be submitted.
        self.save_settings();
    }

    /// Opens the settings window on the page it was last left on.
    pub(super) fn open_settings(&mut self) {
        if self.settings_state.page.is_none() {
            self.settings_state.page = Some(page_named(&self.settings.general.last_settings_page));
        }

        self.settings_state.problems = self.settings.check();
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
        self.apply_settings();
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
