//! Which of the six things the viewer does is on screen, and the switching
//! between them.
//!
//! A mode change is more than a different panel: the folder jobs read the
//! folder when they are opened, the slideshow takes the whole screen and hands
//! it back, and the gallery lands on the picture that was being looked at.

use eframe::egui;

use crate::view::image_view::bottom_bar::{Flags, Marks};
use crate::view::organize::Done;

use super::{App, Mode};

impl App {
    pub(super) fn set_mode(&mut self, mode: Mode) {
        let arriving = mode != self.mode;
        let leaving = self.mode;
        self.mode = mode;

        if arriving {
            self.change_screen(leaving, mode);
        }

        // Opening the gallery lands on the image that was on screen; after
        // that the scroll position is the user's.
        if arriving && mode == Mode::Grid {
            self.grid_view.focus_on(self.image_view.selected_index());
        }

        if mode.is_folder_job() && !self.organize_view.holds(&self.paths) {
            self.organize_view.set_images(self.paths.clone());
        }
    }

    /// Starts and stops what a mode takes over when it is entered.
    ///
    /// The slideshow fills the screen and runs its own clock; leaving it puts
    /// the window back the way it was found and the picture back where the
    /// user had it.
    pub(super) fn change_screen(&mut self, leaving: Mode, arriving: Mode) {
        if leaving.is_fullscreen() == arriving.is_fullscreen() {
            return;
        }

        let fullscreen = arriving.is_fullscreen() || self.was_fullscreen;
        self.pending_fullscreen = Some(fullscreen);

        self.image_view
            .set_slideshow(arriving == Mode::Slideshow, &self.settings.slideshow);
    }

    /// Draws whichever view is on screen, and keeps the others' caches
    /// filling in behind it.
    pub(super) fn show_views(&mut self, ctx: &egui::Context) {
        // A folder job draws no images, so both caches carry on filling in
        // behind it and the viewer is ready the moment it is left.
        let warmed = match self.mode {
            Mode::Image => self.grid_view.warm(self.image_view.selected_index()),
            _ => self.image_view.warm(),
        };

        if warmed {
            ctx.request_repaint();
        }

        if self.mode.is_folder_job() {
            if let Some(done) = self.organize_view.ui(ctx, self.mode) {
                self.finish_folder_job(done);
            }

            return;
        }

        if self.mode == Mode::Grid {
            self.grid_view.ui(ctx);

            if let Some(path) = self.grid_view.take_selected() {
                self.image_view.select_path(&path);
                self.mode = Mode::Image;
            }

            if let Some(callback) = self.grid_view.take_callback() {
                self.execute_callback(callback);
            }

            return;
        }

        let marks = self
            .image_view
            .active_path()
            .and_then(|path| self.annotations.peek(&path).map(Marks::of))
            .unwrap_or_default();

        self.image_view.ui(
            ctx,
            Flags {
                flattened: self.flattened,
                watching: self.watcher.is_active(),
                advancing: self.advancing,
                ..Default::default()
            },
            marks,
        );

        if let Some(callback) = self.image_view.take_callback() {
            self.execute_callback(callback);
        }
    }

    /// Picks the folder up again after a job has changed it.
    ///
    /// A rename moves every path the caches are keyed by, and a time shift
    /// changes what the metadata says, so in both cases the collection is read
    /// again from scratch. It costs a folder's worth of decoding, and it
    /// happens once, when the user asked for something that changed the files.
    pub(super) fn finish_folder_job(&mut self, done: Done) {
        tracing::info!("Folder job finished: {done:?}");

        let base = self.base_path.clone();
        let selected = match done {
            // The name it had is gone, so there is nothing to return to.
            Done::Renamed => None,
            Done::Shifted => self.image_view.active_path(),
        };

        self.annotations.forget_all();
        self.open_directory(&base, selected.as_deref());
        self.organize_view.set_images(self.paths.clone());
    }
}
