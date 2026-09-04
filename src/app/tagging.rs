//! Rating and tagging: the panel, and what clicking in it does.
//!
//! Kept apart from the rest of the application because it is the one part that
//! writes to the user's files.

use std::path::{Path, PathBuf};

use eframe::egui;

use crate::annotations::AnnotationStore;
use crate::history::{Deed, Step};
use crate::metadata::xmp::{Flag, Label, Xmp};
use crate::metadata::Orientation;
use crate::ui::tag_panel::{self, Action};

use super::{App, Mode};

impl App {
    /// Puts `stars` on whatever is being marked.
    pub(super) fn rate(&mut self, stars: u8) {
        self.mark(move |store, path| {
            store.set_rating(path, stars);
        });
    }

    /// Keeps what is being marked, throws it out, or takes the mark back off.
    ///
    /// Pressing the key of the mark it already carries takes that mark off,
    /// which is what every other program does with these keys. Over a
    /// selection the first photograph decides for the rest: a toggle applied
    /// one at a time would leave half the set flagged and half not, which is
    /// never what pressing one key over two hundred frames meant.
    pub(super) fn flag(&mut self, flag: Flag) {
        let wanted = match flag {
            Flag::Unflagged => Flag::Unflagged,
            _ if self.first_marked_carries(|xmp| xmp.flag() == flag) => Flag::Unflagged,
            _ => flag,
        };

        // Taken before the mark rather than after it, because carrying it out
        // is what changes what the answer would be.
        let marked = self.marked_paths();

        self.mark(move |store, path| {
            store.set_flag(path, wanted);
        });

        self.rejected_leave_the_comparison(wanted, &marked);
    }

    /// Flags one named photograph, whatever the keys are about.
    ///
    /// The icons over a pane and the two verbs on its menu are the only things
    /// in the program that mark a photograph other than the one being looked
    /// at — which is the whole point of them, because with four side by side
    /// the one being looked at is one in four.
    pub(super) fn flag_one(&mut self, index: usize, flag: Flag) {
        let Some(path) = self.paths.get(index).cloned() else {
            return;
        };

        let carried = self
            .annotations
            .peek(&path)
            .map(|xmp| xmp.flag())
            .unwrap_or_default();

        // The same toggle the key has: pressing the mark a photograph already
        // carries takes it off.
        let wanted = match carried == flag {
            true => Flag::Unflagged,
            false => flag,
        };

        let paths = [path];
        let files = self.each(&paths, |store, path| {
            store.set_flag(path, wanted);
        });
        self.as_photographs(files, &paths);
        self.refresh_marks(&paths);

        self.rejected_leave_the_comparison(wanted, &paths);
    }

    /// Takes every rejected photograph out of a comparison of the picked-out
    /// ones.
    ///
    /// Judging a row of frames is deciding which of them to be rid of, so the
    /// ones decided against should go rather than sit there being decided
    /// again. Only rejection does this: keeping a photograph is a reason to go
    /// on looking at it beside the others.
    ///
    /// Tied to the deed rather than watched as state, deliberately: a
    /// photograph that was already rejected when it was picked out is one
    /// somebody chose to look at again, and a rule reading the state would
    /// throw it out of the comparison before they had seen it.
    fn rejected_leave_the_comparison(&mut self, applied: Flag, rejected: &[PathBuf]) {
        if applied != Flag::Rejected || !self.image_view.is_comparing_selection() {
            return;
        }

        let going: Vec<usize> = rejected
            .iter()
            .filter_map(|path| self.paths.iter().position(|held| held == path))
            .collect();

        if going.is_empty() {
            return;
        }

        // The focus moves on before the frame goes, so that judging a row is a
        // run of keystrokes rather than a keystroke and a click each time.
        // Taking a photograph out by hand is the other thing and leaves nothing
        // current, because there the person said what they did not want rather
        // than what they wanted next.
        if self
            .image_view
            .focused()
            .is_some_and(|focused| going.contains(&focused))
        {
            self.image_view.focus_next_pane();
        }

        for index in going {
            self.grid_view.unpick(index);
        }
    }

    /// Puts a colour label on what is being marked, or takes it off again.
    pub(super) fn label(&mut self, index: usize) {
        let Some(label) = Label::CHOICES.get(index).copied() else {
            return;
        };

        let wanted =
            (!self.first_marked_carries(|xmp| xmp.known_label() == Some(label))).then_some(label);

        self.mark(move |store, path| {
            store.set_label(path, wanted);
        });
    }

    /// Turns whatever is being marked a quarter.
    pub(super) fn turn(&mut self, clockwise: bool) {
        self.turn_by(Orientation::quarter(clockwise));
    }

    /// Turns whatever is being marked, by any of the eight.
    ///
    /// Through the same path as a star or a flag, so it takes the selection
    /// with it, writes to both halves of a raw-and-JPEG pair, and is one step
    /// of the history however many photographs it touched.
    pub(super) fn turn_by(&mut self, extra: Orientation) {
        let paths = self.marked_paths();

        self.mark(move |store, path| {
            store.turn_by(path, extra);
        });

        // What is on the card is turned as well, so the picture moves now
        // rather than when it is next decoded. The sidecar is the lasting
        // answer and the decoder reads it; this is the same answer, applied
        // without waiting three hundred milliseconds for a raw file.
        for path in &paths {
            for path in self.with_partners(path) {
                self.image_view.turn_by(&path, extra);
                self.grid_view.turn_by(&path, extra);
            }
        }
    }

    /// Whether the photograph the others follow already carries something.
    ///
    /// Reads from the disk if it has to, because the answer decides what the
    /// whole selection ends up as, and guessing it from an unread entry would
    /// put a mark on where the user was trying to take one off.
    fn first_marked_carries(&mut self, holds: impl Fn(&Xmp) -> bool) -> bool {
        let Some(first) = self.marked_paths().into_iter().next() else {
            return false;
        };

        self.load_annotations(&first);
        holds(self.annotations.get(&first, None))
    }

    /// Applies a mark to everything that is being marked.
    ///
    /// The annotations are read from disk first, because marking an image
    /// nobody has read is how the keywords on it get lost. What changed is
    /// recorded as one step however many photographs it touched, so undoing a
    /// selection is the same single keystroke that made it.
    fn mark(&mut self, apply: impl FnMut(&mut AnnotationStore, &Path)) {
        let paths = self.marked_paths();
        if paths.is_empty() {
            return;
        }

        let files = self.each(&paths, apply);
        let changed = self.as_photographs(files, &paths);
        self.refresh_marks(&paths);

        // Said out loud only when it was not one photograph, because a mark on
        // the picture on screen announces itself by changing what is drawn and
        // a mark on two hundred of them does not.
        if paths.len() > 1 {
            self.notices
                .say(format!("{changed} of {} photographs changed", paths.len()));
        }
    }

    /// Draws the rating and tagging panel and applies what was clicked.
    pub(super) fn show_tag_panel(&mut self, ctx: &egui::Context) {
        let Some(path) = self.marked_path() else {
            // The panel used to return before drawing anything, so the key
            // that opens it changed no pixel on an empty folder and looked
            // broken.
            let (size, width, _) = self.tag_panel_size.asked_for();
            self.tag_panel_size = size;
            tag_panel::nothing_open(ctx, self.tag_panel_visible, width);
            return;
        };

        self.load_annotations(&path);
        let applies_to = self.marked_paths().len();

        // One frame, and only when the window is what changed it: forcing it
        // every frame would take the edge away from the pointer.
        let (size, panel_width, forced) = self.tag_panel_size.asked_for();
        self.tag_panel_size = size;

        self.refresh_seen_tags();

        let actions = {
            // Tags typed on other images of this folder are offered again
            // without having to be configured.
            let seen: Vec<&str> = self.seen_tags.1.iter().map(String::as_str).collect();
            let empty = Xmp::default();
            let source = tag_panel::Source {
                annotations: self.annotations.peek(&path).unwrap_or(&empty),
                catalog: &self.catalog,
                recent: &self.recent_tags,
                seen: &seen,
                applies_to: applies_to.max(1),
            };

            tag_panel::ui(
                ctx,
                self.tag_panel_visible,
                panel_width,
                forced,
                &mut self.tag_panel,
                &source,
            )
        };

        // The panel shows one photograph and applies to all of them: clicking
        // a keyword with two hundred frames picked out is the whole reason to
        // have picked two hundred frames out.
        let targets = self.marked_paths();

        for action in actions {
            match action {
                Action::SetRating(stars) => {
                    self.each(&targets, |store, path| {
                        store.set_rating(path, stars);
                    });
                }
                Action::SetFlag(flag) => {
                    self.each(&targets, |store, path| {
                        store.set_flag(path, flag);
                    });
                }
                Action::SetLabel(index) => {
                    let label = index.and_then(|i| Label::CHOICES.get(i)).copied();
                    self.each(&targets, |store, path| {
                        store.set_label(path, label);
                    });
                }
                Action::AddTag(tag) => {
                    let mut added = false;
                    self.each(&targets, |store, path| {
                        added |= store.add_tag(path, &tag);
                    });

                    if added {
                        self.recent_tags.remember(tag);
                    }
                }
                Action::RemoveTag(tag) => {
                    self.each(&targets, |store, path| {
                        store.remove_tag(path, &tag);
                    });
                }
                // Through the same field the settings window writes, so a
                // dragged edge survives the session.
                Action::ShowOnlyStars(stars) => {
                    self.show_only(crate::app::verbs::Narrow::Stars(stars))
                }
                Action::ShowOnlyFlag(flag) => self.show_only(crate::app::verbs::Narrow::Flag(flag)),
                Action::ShowOnlyLabel(index) => {
                    if let Some(label) = Label::CHOICES.get(index).copied() {
                        self.show_only(crate::app::verbs::Narrow::Label(label));
                    }
                }
                Action::ShowOnlyKeyword(keyword) => self.show_only_keyword(&keyword),
                Action::Settings(path) => self.open_settings_at(path),
                Action::PanelWidth(width) => {
                    self.tag_config.panel_width = width;
                    self.settings.tags.panel_width = width;
                    self.tag_panel_size = self.tag_panel_size.dragged_to(width);
                    self.save_settings();
                }
            }
        }

        self.refresh_marks(&targets);
        self.recent_tags.save_if_changed();
    }

    /// Rebuilds the keyword list the panel offers, if it would have changed.
    ///
    /// A walk over every entry in the folder, sorted and deduplicated, which
    /// was being done on every frame the panel was open — on a folder of two
    /// thousand rated photographs that is real work to arrive back where it
    /// started, since keywords change only when somebody types one.
    fn refresh_seen_tags(&mut self) {
        let revision = self.annotations.revision();
        if self.seen_tags.0 == Some(revision) {
            return;
        }

        self.seen_tags = (
            Some(revision),
            self.annotations
                .known_tags()
                .into_iter()
                .map(str::to_string)
                .collect(),
        );
    }

    /// Applies something to every photograph in `targets`, recording it as one
    /// step, and reports how many it actually changed.
    ///
    /// A raw and a JPEG shot together are one photograph, so a mark goes on
    /// both: each keeps its own sidecar, and a pair whose two halves disagreed
    /// about a rating would be a pair that had to be culled twice.
    fn each(
        &mut self,
        targets: &[PathBuf],
        mut apply: impl FnMut(&mut AnnotationStore, &Path),
    ) -> usize {
        let mut steps = Vec::new();
        let targets: Vec<PathBuf> = targets
            .iter()
            .flat_map(|path| self.with_partners(path))
            .collect();

        for path in &targets {
            self.load_annotations(path);

            // Recorded before the change rather than after it: undoing a mark
            // is writing back what was there, and what was there is only
            // knowable now.
            let before = self.annotations.get(path, None).clone();

            apply(&mut self.annotations, path);

            // Both halves, so the mark can be put back *and* put on again:
            // a rating undone is a rating that redo has to be able to restore.
            match self.annotations.peek(path) {
                Some(after) if after != &before => steps.push(Step::Marked {
                    image: path.clone(),
                    before: Box::new(before),
                    after: Box::new(after.clone()),
                }),
                _ => {}
            }
        }

        let changed = steps.len();
        match changed {
            0 => {}
            1 => {
                self.history.record(Deed::Files(steps.remove(0)));
            }
            _ => {
                self.history.record(Deed::Files(Step::Many(steps)));
            }
        }

        changed
    }

    /// How many photographs a count of changed *files* stands for.
    ///
    /// Marking a raw+JPEG pair writes two sidecars and is one photograph, and
    /// saying "2 of 1 photographs changed" would be nonsense.
    fn as_photographs(&self, files: usize, shown: &[PathBuf]) -> usize {
        let per = shown
            .iter()
            .map(|path| self.with_partners(path).len())
            .max()
            .unwrap_or(1)
            .max(1);

        files.div_ceil(per)
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

    /// The photograph every panel is about.
    ///
    /// One answer, so a link one panel draws leads where another panel is
    /// looking. The metadata panel read the image view and the keyword panel
    /// read the sheet's cursor, so with both open the two described different
    /// photographs and neither said which.
    pub(super) fn current_photograph(&self) -> Option<PathBuf> {
        self.marked_path()
    }

    /// Every photograph a command applies to.
    ///
    /// The selection when the contact sheet has one, and the photograph being
    /// looked at when it has not. One rule, read by marking, tagging, moving
    /// and deleting alike, so that picking frames out changes what every
    /// command means without any of those commands having to know that a
    /// selection exists.
    pub(super) fn marked_paths(&self) -> Vec<PathBuf> {
        // No mode gate. It used to read the selection only in the contact
        // sheet, so opening one of two hundred picked-out photographs silently
        // reduced the selection to one — and putting the sheet back did not
        // bring it back, because the next command had already acted.
        // While the picked-out photographs are side by side, a command is
        // about the one being looked at rather than about all of them. That is
        // the inversion of the rule below and it is what makes the comparison
        // a place to work rather than only to look: rating one of five, tagging
        // one of five, throwing one of five out. Everything else about the set
        // is still there — it is on the strip — and closing the comparison
        // puts the set back in charge.
        if self.image_view.is_comparing_selection() {
            return self.marked_path().into_iter().collect();
        }

        if self.grid_view.selected_count() > 0 {
            return self.grid_view.selected_paths();
        }

        self.marked_path().into_iter().collect()
    }

    /// Loads the annotations for `image`, seeding them from what the file
    /// itself carries.
    ///
    /// What the file itself carries is only used when the image view is on
    /// that very photograph, because that is the only one whose decoded
    /// metadata is to hand. Marking from the contact sheet reads the sidecar
    /// and nothing else, which is what the store does on a miss anyway.
    pub(super) fn load_annotations(&mut self, image: &Path) {
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
