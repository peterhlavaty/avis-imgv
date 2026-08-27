//! Rating and tagging: the panel, and what clicking in it does.
//!
//! Kept apart from the rest of the application because it is the one part that
//! writes to the user's files.

use std::path::Path;

use eframe::egui;

use crate::metadata::xmp::Xmp;
use crate::ui::tag_panel::{self, Action};

use super::App;

impl App {
    /// Puts `stars` on the image on screen.
    pub(super) fn rate(&mut self, stars: u8) {
        let Some(path) = self.image_view.active_path() else {
            return;
        };

        self.load_annotations(&path);
        self.annotations.set_rating(&path, stars);
    }

    /// Draws the rating and tagging panel and applies what was clicked.
    pub(super) fn show_tag_panel(&mut self, ctx: &egui::Context) {
        let Some(path) = self.image_view.active_path() else {
            return;
        };

        self.load_annotations(&path);

        let actions = {
            // Tags typed on other images of this folder are offered again
            // without having to be configured.
            let seen = self.annotations.known_tags();
            let empty = Xmp::default();
            let source = tag_panel::Source {
                annotations: self.annotations.peek(&path).unwrap_or(&empty),
                catalog: &self.catalog,
                recent: &self.recent_tags,
                seen: &seen,
            };

            tag_panel::ui(
                ctx,
                self.tag_panel_visible,
                self.tag_config.panel_width,
                &mut self.tag_panel,
                &source,
            )
        };

        for action in actions {
            match action {
                Action::SetRating(stars) => {
                    self.annotations.set_rating(&path, stars);
                }
                Action::AddTag(tag) => {
                    if self.annotations.add_tag(&path, &tag) {
                        self.recent_tags.remember(tag);
                    }
                }
                Action::RemoveTag(tag) => {
                    self.annotations.remove_tag(&path, &tag);
                }
            }
        }

        self.recent_tags.save_if_changed();
    }

    /// Loads the annotations for `image`, seeding them from what the file
    /// itself carries.
    ///
    /// Does nothing until the image has been decoded, because that is when its
    /// embedded rating becomes known; caching an empty entry before then would
    /// hide a rating set elsewhere.
    fn load_annotations(&mut self, image: &Path) {
        if self.annotations.peek(image).is_some() {
            return;
        }

        let Some(embedded) = self.image_view.active_metadata().map(|m| m.xmp.clone()) else {
            return;
        };

        self.annotations.get(image, Some(&embedded));
    }
}
