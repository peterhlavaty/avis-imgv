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
use crate::ui::{keys, legend, notice, placeholders};

use super::about;
use super::conflict;
use super::panels::{self, MenuAction};
use super::{App, Mode};

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
            MenuAction::Keyboard => self.keys_visible = true,
            MenuAction::Slideshow => self.slideshow_visible = true,
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
    /// Draws the slideshow settings and hands changes to the view.
    pub(super) fn show_slideshow_settings(&mut self, ctx: &egui::Context) {
        let mut open = self.slideshow_visible;
        let changed = panels::slideshow_settings(ctx, &mut open, &mut self.settings.slideshow);
        self.slideshow_visible = open;

        if !changed {
            return;
        }

        // Restarted so a changed interval or motion takes effect at once
        // rather than at the next picture.
        if self.mode == Mode::Slideshow {
            self.image_view
                .set_slideshow(false, &self.settings.slideshow);
            self.image_view
                .set_slideshow(true, &self.settings.slideshow);
        }

        self.save_settings();
    }

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
        self.config = self.settings.general.clone();
        self.tag_config = self.settings.tags.clone();
        self.image_view.set_config(self.settings.image_view.clone());
        self.grid_view.set_config(self.settings.grid_view.clone());
    }
}
