//! Which of the six things the viewer does is on screen, and the switching
//! between them.
//!
//! A mode change is more than a different panel: the folder jobs read the
//! folder when they are opened, the slideshow takes the whole screen and hands
//! it back, and the gallery lands on the picture that was being looked at.

use eframe::egui;

use crate::ui::empty::Nothing;
use crate::view::image_view::bottom_bar::{Flags, Marks};
use crate::view::organize::Done;

use super::{App, Mode};

/// What the strip is given when somebody asks for it and it has no height.
///
/// Tall enough to read a thumbnail at, short enough not to take the
/// photograph's room.
const DEFAULT_FILMSTRIP_HEIGHT: f32 = 96.0;

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

        // Every file, not only the browsed half of each pair: a bulk rename
        // that renamed the JPEG and left the raw behind would break the very
        // pairing it depends on, and a list that hides half the folder is a
        // poor thing to check a rename against.
        let everything = self.all_paths();
        if mode.is_folder_job() && !self.organize_view.holds(&everything) {
            self.organize_view.set_images(everything);
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
            self.ensure_marks();
            // Built only on the frames it is drawn on: the sentences are
            // allocated, and this runs once a frame for every folder.
            let nothing = if self.grid_view.shows_nothing() {
                self.nothing_to_show()
            } else {
                Nothing::default()
            };

            self.grid_view
                .ui(ctx, &self.marks, self.stacking.stacks(), &nothing);

            if let Some(asked) = self.grid_view.take_asked() {
                self.run_asked(asked);
            }

            if let Some(path) = self.grid_view.take_selected() {
                self.image_view.select_path(&path);
                self.set_mode(Mode::Image);
            }

            if let Some(callback) = self.grid_view.take_callback() {
                self.execute_callback(callback);
            }

            if let Some((verb, path)) = self.grid_view.take_verb() {
                self.run_verb(verb, path);
            }

            return;
        }

        // Before the image view, so the strip claims its band of the window
        // and the photograph is fitted to what is left rather than being drawn
        // under it.
        self.show_filmstrip(ctx);

        let showing = self.image_view.active_path();
        let marks = showing
            .as_ref()
            .and_then(|path| self.annotations.peek(path).map(Marks::of))
            .unwrap_or_default();

        let paired = showing
            .as_ref()
            .is_some_and(|path| !self.pairs.partners_of(path).is_empty());

        let nothing = if self.image_view.shows_nothing() {
            self.nothing_to_show()
        } else {
            Nothing::default()
        };

        self.image_view.ui(
            ctx,
            Flags {
                flattened: self.flattened,
                watching: self.watcher.is_active(),
                advancing: self.advancing,
                paired,
                place: self
                    .stacking
                    .stacks()
                    .place_of(self.image_view.selected_index()),
                ..Default::default()
            },
            marks,
            &nothing,
            self.mode,
            self.notices.unseen(),
        );

        if let Some(asked) = self.image_view.take_asked() {
            self.run_asked(asked);
        }

        if let Some(callback) = self.image_view.take_callback() {
            self.execute_callback(callback);
        }

        if let Some((verb, path)) = self.image_view.take_verb() {
            self.run_verb(verb, path);
        }

        for action in self.image_view.take_bar_actions() {
            self.run_bar_action(action);
        }
    }

    /// Draws the strip of thumbnails under the photograph, if it is up.
    ///
    /// From the contact sheet's own store, whose textures are resident
    /// whichever view is on screen — a strip with a cache of its own would
    /// decode the folder a second time.
    fn show_filmstrip(&mut self, ctx: &egui::Context) {
        let height = self.settings.grid_view.filmstrip_height;
        if !self.filmstrip_visible || height <= 0.0 {
            return;
        }

        let cursor = self.image_view.selected_index();
        let (opened, dragged) = self.grid_view.filmstrip(ctx, cursor, height);

        if let Some(path) = opened {
            self.image_view.select_path(&path);
        }

        // Through the field the settings window reads, so dragging the strip's
        // edge is a change that is still there on the next launch.
        if let Some(height) = dragged {
            self.settings.grid_view.filmstrip_height = height;
            self.grid_view.set_config(self.settings.grid_view.clone());
            self.save_settings();
        }
    }

    /// Shows or hides the strip, saying so when it cannot.
    ///
    /// A command advertised in the editor and on the cheat sheet that silently
    /// does nothing is the worst kind of dead end: the key is documented, the
    /// person presses it, and the program's answer is no pixel changing. The
    /// strip's height and its visibility used to be one number, so on a fresh
    /// install — where the default height is zero — the key did exactly that.
    /// The two are separate fields now; this is the sentence for whatever the
    /// next one is.
    pub(super) fn toggle_filmstrip(&mut self) {
        self.filmstrip_visible = !self.filmstrip_visible;
        self.settings.grid_view.filmstrip_visible = self.filmstrip_visible;

        if self.filmstrip_visible && self.settings.grid_view.filmstrip_height <= 0.0 {
            // Given a height rather than only complained about: the person
            // asked for the strip, and a strip of no height is not an answer.
            self.settings.grid_view.filmstrip_height = DEFAULT_FILMSTRIP_HEIGHT;
            self.notices.say(format!(
                "The strip had no height; it is {DEFAULT_FILMSTRIP_HEIGHT:.0} points now.                  Drag its top edge to change it."
            ));
        }

        self.save_settings();
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

        // The one follow-on that is not handed in as an argument: this view
        // wants the collection, and the collection does not exist until the
        // walk finishes.
        if let Some(opening) = &mut self.opening {
            opening.tell_organize = true;
        }
    }
}
