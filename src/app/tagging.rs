//! Rating and tagging: the panel, and what clicking in it does.
//!
//! Kept apart from the rest of the application because it is the one part that
//! writes to the user's files.

use std::path::{Path, PathBuf};

use eframe::egui;

use crate::annotations::AnnotationStore;
use crate::metadata::xmp::{Flag, Label, Xmp};
use crate::organize::journal::Step;
use crate::ui::tag_panel::{self, Action};

use super::{App, Mode};

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
        let Some(path) = self.marked_path() else {
            return;
        };

        self.load_annotations(&path);

        // Recorded before the change rather than after it: undoing a mark is
        // writing back what was there, and what was there is only knowable
        // now.
        let before = self.annotations.get(&path, None).clone();

        apply(&mut self.annotations, &path);

        if self.annotations.peek(&path) != Some(&before) {
            self.journal.record(Step::Marked {
                image: path.clone(),
                before: Box::new(before),
            });
        }

        self.refresh_mark(&path);
    }

    /// Draws the rating and tagging panel and applies what was clicked.
    pub(super) fn show_tag_panel(&mut self, ctx: &egui::Context) {
        let Some(path) = self.marked_path() else {
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

        self.refresh_mark(&path);
        self.recent_tags.save_if_changed();
    }

    /// The photograph a mark applies to.
    ///
    /// Whichever one is being looked at, which in the contact sheet is the one
    /// under the keyboard rather than the one the image view was left on:
    /// rating from the sheet used to rate whatever had last been open in the
    /// other view.
    pub(super) fn marked_path(&self) -> Option<PathBuf> {
        match self.mode {
            Mode::Grid => self.grid_view.cursor_path(),
            _ => self.image_view.active_path(),
        }
    }

    /// Loads the annotations for `image`, seeding them from what the file
    /// itself carries.
    ///
    /// What the file itself carries is only used when the image view is on
    /// that very photograph, because that is the only one whose decoded
    /// metadata is to hand. Marking from the contact sheet reads the sidecar
    /// and nothing else, which is what the store does on a miss anyway.
    fn load_annotations(&mut self, image: &Path) {
        if self.annotations.peek(image).is_some() {
            return;
        }

        let embedded = (self.image_view.active_path().as_deref() == Some(image))
            .then(|| {
                self.image_view
                    .active_decoded_metadata()
                    .map(|m| m.xmp.clone())
            })
            .flatten();

        self.annotations.get(image, embedded.as_ref());
    }
}
