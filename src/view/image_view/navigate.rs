//! Moving through the folder.
//!
//! Every way in and out of a picture goes through `select`, which is where the
//! zoom and position of the one being left are put away and the one arrived at
//! is put back where it was.

use std::path::{Path, PathBuf};

use eframe::epaint::Vec2;

use crate::actions::Callback;
use crate::cache::StoreStats;
use crate::config::{ImageViewConfig, Motion, SlideshowConfig};
use crate::metadata::Metadata;

use super::slideshow::Slideshow;
use super::viewports::Place;
use super::{zoom, ImageView};

impl ImageView {
    /// Opens a new collection, optionally starting on a specific image.
    pub fn set_images(&mut self, paths: Vec<PathBuf>, selected: Option<&Path>) {
        let selected = selected
            .and_then(|path| paths.iter().position(|candidate| candidate == path))
            .unwrap_or(0);

        self.store.set_paths(paths);
        self.viewports.clear();
        self.select(selected);
    }

    /// Starts or stops the slideshow.
    ///
    /// Starting it fills the screen and hides the status bar; stopping it puts
    /// the picture back where the user had it, because a drifting zoom is not
    /// somewhere anybody chose to be.
    pub fn set_slideshow(&mut self, running: bool, config: &SlideshowConfig) {
        if running == self.slideshow.is_some() {
            return;
        }

        self.slideshow_config = config.clone();
        self.slideshow = running.then(|| Slideshow::new(config));
        self.viewport.maximize = running && config.motion != Motion::Still;
        self.viewport.maximized = false;
        self.frame.enabled = running && config.start_with_frame_enabled;

        if !running {
            zoom::fit(&mut self.viewport);
            self.viewport.pan = Vec2::ZERO;
        }
    }

    /// Takes a changed configuration, for when the keyboard map is edited.
    pub fn set_config(&mut self, config: ImageViewConfig) {
        self.frame.relative_size = config.frame_size_relative_to_image;
        self.config = config;
    }

    pub fn selected_index(&self) -> usize {
        self.cursor
    }

    pub fn active_path(&self) -> Option<PathBuf> {
        self.store.path(self.cursor).map(Path::to_path_buf)
    }

    pub fn active_metadata(&self) -> Option<&Metadata> {
        self.store.metadata(self.cursor)
    }

    /// Metadata read from the whole of the active file, once it is decoded.
    pub fn active_decoded_metadata(&self) -> Option<&Metadata> {
        self.store.decoded_metadata(self.cursor)
    }

    pub fn stats(&self) -> StoreStats {
        self.store.stats()
    }

    pub fn take_callback(&mut self) -> Option<Callback> {
        self.callback.take()
    }

    /// Moves to `index`, which is where the caches centre themselves.
    ///
    /// The image being left keeps its zoom and pan, and the one arrived at
    /// gets whatever it was left at, so a folder can be walked without losing
    /// the place found in each picture.
    pub fn select(&mut self, index: usize) {
        let previous = self.cursor;
        self.cursor = if self.store.is_empty() {
            0
        } else {
            index.min(self.store.len() - 1)
        };

        self.store.set_cursor(self.cursor);

        if self.cursor == previous {
            return;
        }

        // Cloned rather than borrowed: the viewports outlive the borrow of
        // the store that produced the path.
        if let Some(path) = self.store.path(previous).map(Path::to_path_buf) {
            self.viewports.save(&path, &self.viewport);
        }

        // Remembered even when it was not worth an entry of its own: repeating
        // a plain fitted view onto the next picture is still a thing to ask
        // for, and it is what undoes an accidental repeat.
        self.previous_place = Place::of(&self.viewport);

        self.viewport.reset_for_new_image();
        if let Some(path) = self.active_path() {
            self.viewports.restore(&path, &mut self.viewport);
        }

        if let Some(slideshow) = &mut self.slideshow {
            slideshow.restart();
        }
    }

    pub fn select_path(&mut self, path: &Path) {
        if let Some(index) = self.store.index_of(path) {
            self.select(index);
        }
    }

    /// Removes an image from the collection, staying on the same position.
    pub fn pop(&mut self, path: &Path) {
        let Some(index) = self.store.index_of(path) else {
            return;
        };

        self.store.remove(index);
        self.viewports.forget(path);
        self.select(self.cursor.min(self.store.len().saturating_sub(1)));
    }

    pub fn reload(&mut self, path: &Path) {
        if let Some(index) = self.store.index_of(path) {
            self.store.reload(index);
        }
    }

    pub fn next_image(&mut self) {
        if self.store.is_empty() || self.should_wait() {
            return;
        }

        self.select((self.cursor + 1) % self.store.len());
    }

    pub fn previous_image(&mut self) {
        if self.store.is_empty() {
            return;
        }

        let last = self.store.len() - 1;
        self.select(if self.cursor == 0 {
            last
        } else {
            self.cursor - 1
        });
    }
}
