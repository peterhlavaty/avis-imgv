//! The menu, and the windows it opens.
//!
//! Everything here is the same shape: draw a window, take what it changed, and
//! hand it to whichever part of the viewer holds a copy. The configuration is
//! written out as soon as anything in it moves, so a key changed in the middle
//! of a session survives the end of it.

use eframe::egui::{self, ViewportCommand};

use crate::formats;
use crate::ui::keys;

use super::panels::{self, MenuAction};
use super::{App, Mode};

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
            MenuAction::Mode(mode) => self.set_mode(mode),
            MenuAction::Keyboard => self.keys_visible = true,
            MenuAction::Slideshow => self.slideshow_visible = true,
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

        if let Err(e) = self.settings.save() {
            tracing::error!("Could not write the configuration: {e}");
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

        if outcome.is_none() {
            return;
        }

        self.config = self.settings.general.clone();
        self.tag_config = self.settings.tags.clone();
        self.image_view.set_config(self.settings.image_view.clone());
        self.grid_view.set_config(self.settings.grid_view.clone());

        if let Err(e) = self.settings.save() {
            tracing::error!("Could not write the configuration: {e}");
        }
    }
}
