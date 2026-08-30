//! Rating and tagging: the panel, and what clicking in it does.
//!
//! Kept apart from the rest of the application because it is the one part that
//! writes to the user's files.

use std::path::Path;

use eframe::egui;

use crate::annotations::AnnotationStore;
use crate::metadata::xmp::{Flag, Label, Xmp};
use crate::ui::tag_panel::{self, Action};

use super::App;

impl App {
    /// Puts `stars` on the image on screen.
    pub(super) fn rate(&mut self, stars: u8) {
        self.mark(|store, path| {
            store.set_rating(path, stars);
        });
    }

    /// Keeps the image on screen, throws it out, or takes the mark back off.
    ///
    /// Pressing the key of the mark it already carries takes that mark off,
    /// which is what every other program does with these keys.
    pub(super) fn flag(&mut self, flag: Flag) {
        self.mark(|store, path| {
            if flag == Flag::Unflagged {
                store.set_flag(path, flag);
            } else {
                store.toggle_flag(path, flag);
            }
        });
    }

    /// Puts a colour label on the image on screen, or takes it off again.
    pub(super) fn label(&mut self, index: usize) {
        let Some(label) = Label::CHOICES.get(index).copied() else {
            return;
        };

        self.mark(|store, path| {
            store.toggle_label(path, label);
        });
    }

    /// Applies a mark to the image on screen.
    ///
    /// The annotations are read from disk first, because marking an image
    /// nobody has read is how the keywords on it get lost.
    fn mark(&mut self, apply: impl FnOnce(&mut AnnotationStore, &Path)) {
        let Some(path) = self.image_view.active_path() else {
            return;
        };

        self.load_annotations(&path);
        apply(&mut self.annotations, &path);
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
                Action::SetFlag(flag) => {
                    self.annotations.set_flag(&path, flag);
                }
                Action::SetLabel(index) => match index.and_then(|i| Label::CHOICES.get(i)) {
                    Some(label) => {
                        self.annotations.toggle_label(&path, *label);
                    }
                    None => {
                        self.annotations.clear_label(&path);
                    }
                },
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

        let Some(embedded) = self
            .image_view
            .active_decoded_metadata()
            .map(|m| m.xmp.clone())
        else {
            return;
        };

        self.annotations.get(image, Some(&embedded));
    }
}
