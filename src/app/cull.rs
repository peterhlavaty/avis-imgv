//! Taking photographs out of the folder.
//!
//! The verb a culling tool is for and the one this viewer had no answer to:
//! everything else here decides what a picture *is*, and this decides whether
//! it stays. Kept apart from the rest of the wiring for the same reason the
//! tagging is — it is one of the two places that touch the user's files.

use std::path::{Path, PathBuf};

use eframe::egui;

use crate::metadata::xmp::Flag;
use crate::organize::files;
use crate::organize::journal::Step;
use crate::ui::destinations::{self, Answer, Asking, Errand, Slot};
use crate::utils;

use super::App;

/// A deletion the user has been asked about but has not answered.
///
/// Only the permanent kind, and only the ones that take more than one
/// photograph: sending a single frame to the bin is answerable with the bin
/// itself, and a dialogue in the middle of a cull is what people complain
/// about most in the tools that have one.
#[derive(Debug, Clone)]
pub struct Pending {
    pub paths: Vec<PathBuf>,
    /// Whether they go to the bin or straight off the disk.
    pub permanent: bool,
}

impl Pending {
    fn question(&self) -> String {
        let count = self.paths.len();
        let what = if count == 1 {
            self.paths
                .first()
                .and_then(|path| path.file_name())
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_else(|| "this photograph".to_string())
        } else {
            format!("{count} photographs")
        };

        if self.permanent {
            format!("Delete {what} for good? This cannot be undone.")
        } else {
            format!("Send {what} to the bin?")
        }
    }
}

impl App {
    /// Sends what is being looked at — or what is picked out — to the bin, or
    /// off the disk entirely.
    ///
    /// One photograph to the bin needs no asking, because the bin is the
    /// asking. Deleting for good does, and so does taking a whole selection:
    /// the cost of a wrong keystroke there is a folder rather than a frame.
    pub(super) fn delete_open_image(&mut self, permanent: bool) {
        let paths = self.marked_paths();
        if paths.is_empty() {
            return;
        }

        let pending = Pending { permanent, paths };

        if permanent || pending.paths.len() > 1 {
            self.pending_delete = Some(pending);
            return;
        }

        self.carry_out(pending);
    }

    /// Asks about every photograph in the folder that has been rejected.
    ///
    /// The second half of a first pass: mark the ones that are not staying,
    /// then be rid of all of them at once. The sidecars are read here rather
    /// than waited for, because a folder's worth of them is a few milliseconds
    /// and the alternative is only knowing about the frames already looked at.
    pub(super) fn bin_rejected(&mut self) {
        let paths: Vec<PathBuf> = self
            .paths
            .clone()
            .into_iter()
            .filter(|path| self.annotations.get(path, None).flag() == Flag::Rejected)
            .collect();

        if paths.is_empty() {
            self.notices.say("Nothing in this folder is rejected");
            return;
        }

        self.pending_delete = Some(Pending {
            paths,
            permanent: false,
        });
    }

    /// Draws the question, if there is one outstanding.
    pub(super) fn show_pending_delete(&mut self, ctx: &egui::Context) {
        let Some(pending) = self.pending_delete.clone() else {
            return;
        };

        // A window asking a question owns the keyboard while it is up, or the
        // key that answers it also does whatever it does the rest of the time.
        utils::set_mute_state(ctx, true);

        let mut answered = None;

        egui::Window::new(if pending.permanent {
            "Delete for good"
        } else {
            "Move to the bin"
        })
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.label(pending.question());
            ui.add_space(10.0);

            ui.horizontal(|ui| {
                if ui.button("Yes").clicked() {
                    answered = Some(true);
                }
                if ui.button("Leave them alone").clicked() {
                    answered = Some(false);
                }
            });

            ui.add_space(6.0);
            ui.label(
                egui::RichText::new(if pending.permanent {
                    "Y to delete · Escape to leave them alone"
                } else {
                    "Enter or Y to send them · Escape to leave them alone"
                })
                .weak(),
            );
        });

        // Consumed rather than read. Answering the question un-mutes the
        // keyboard, and the views draw after this does, so a key merely looked
        // at would go on to mean whatever it means the rest of the time:
        // pressing Enter to empty the bin also opened the photograph under the
        // cursor.
        //
        // Escape is the answer people reach for without thinking, and the safe
        // one is the one it should give.
        let said_no = ctx.input_mut(|i| {
            let escaped = i.consume_key(egui::Modifiers::NONE, egui::Key::Escape);
            escaped | i.consume_key(egui::Modifiers::NONE, egui::Key::N)
        });

        // A window that owns the keyboard has to be answerable from it: this
        // one comes up in the middle of a cull, and reaching for the mouse to
        // say yes is the thing the whole keyboard map exists to avoid.
        //
        // Enter says yes to the bin, which can be taken back. It does not say
        // yes to a permanent deletion — that is the one answer nobody can
        // undo, so it costs a key nobody presses by accident.
        let said_yes = ctx.input_mut(|i| {
            let yes = i.consume_key(egui::Modifiers::NONE, egui::Key::Y);
            yes | (!pending.permanent && i.consume_key(egui::Modifiers::NONE, egui::Key::Enter))
        });

        if said_no {
            answered = Some(false);
        } else if said_yes {
            answered = Some(true);
        }

        match answered {
            Some(true) => {
                self.close_modal(ctx);
                self.carry_out(pending);
            }
            Some(false) => self.close_modal(ctx),
            None => {}
        }
    }

    /// Gives the keyboard back once nothing is being asked.
    fn close_modal(&mut self, ctx: &egui::Context) {
        self.pending_delete = None;
        self.asking = None;
        utils::set_mute_state(ctx, false);
    }

    /// Does it, and says what happened.
    fn carry_out(&mut self, pending: Pending) {
        let mut gone = 0usize;
        let mut failed = 0usize;
        let mut binned: Vec<PathBuf> = Vec::new();

        for path in &pending.paths {
            let outcome = if pending.permanent {
                files::delete(path)
            } else {
                files::to_bin(path)
            };

            match outcome {
                Ok(()) => {
                    gone += 1;
                    binned.push(path.clone());
                    self.forget(path);
                }
                Err(e) => {
                    failed += 1;
                    tracing::error!("{e}");
                    self.notices.say(format!("{e}"));
                }
            }
        }

        // Only the bin can be undone: what was deleted for good is gone, and
        // the journal must not suggest otherwise.
        if !pending.permanent {
            self.journal.record(Step::Binned(binned));
        }

        if gone > 0 {
            let where_to = if pending.permanent {
                "deleted"
            } else {
                "sent to the bin"
            };

            self.notices.say(format!("{gone} photograph(s) {where_to}"));
        }

        if failed > 0 && gone > 0 {
            self.notices
                .say(format!("{failed} could not be, and are still there"));
        }

        if pending.paths.len() > 1 {
            self.grid_view.clear_selection();
        }
    }

    /// Takes a photograph out of the open collection.
    ///
    /// The cursor stays where it is rather than following the picture that has
    /// gone, so what it lands on is the next one — which is the single most
    /// complained about detail of culling in Lightroom.
    fn forget(&mut self, path: &Path) {
        if let Some(index) = self.paths.iter().position(|candidate| candidate == path) {
            self.paths.remove(index);
            self.marks.remove(index);
        }

        self.image_view.pop(path);
        self.grid_view.pop(path);
        self.annotations.forget(path);
    }
}

/// Moving photographs somewhere else, and taking it back.
impl App {
    /// Opens the panel that asks where the photograph should go.
    ///
    /// Pressing the same key twice in a row skips the panel and repeats the
    /// last answer, which is the motion every fast viewer has: the panel is
    /// there for the first frame of a shoot and out of the way for the rest.
    pub(super) fn send_somewhere(&mut self, errand: Errand) {
        let count = self.marked_paths().len();
        if count == 0 {
            return;
        }

        let repeat = self.last_errand == Some(errand);
        if let (true, Some(slot)) = (repeat, self.last_destination.clone()) {
            self.carry_errand(errand, &slot);
            return;
        }

        self.last_errand = Some(errand);
        self.asking = Some(Asking {
            errand,
            count,
            slots: self.slots(),
            last: self.last_destination.clone(),
        });
    }

    /// Moves the photograph into the folder for the frames that are not
    /// staying.
    ///
    /// A folder rather than the bin, because a first pass happens on a card or
    /// a network share and neither of those has one.
    pub(super) fn send_to_rejected(&mut self) {
        let folder = self.settings.cull.rejected_folder.clone();
        if folder.trim().is_empty() {
            return;
        }

        let slot = Slot {
            label: folder.clone(),
            path: self.base_path.join(folder),
        };

        self.carry_errand(Errand::Move, &slot);
    }

    /// Draws the panel, if it is open, and does what it was told.
    pub(super) fn show_destinations(&mut self, ctx: &egui::Context) {
        let Some(asking) = self.asking.clone() else {
            return;
        };

        // The digits are the point of this panel, and they are also the star
        // ratings, so it takes the keyboard for as long as it is up.
        utils::set_mute_state(ctx, true);

        let Some(answer) = destinations::ui(ctx, &asking) else {
            return;
        };

        self.close_modal(ctx);

        match answer {
            Answer::Send(slot) => self.carry_errand(asking.errand, &slot),
            Answer::Browse => {
                let picked = rfd::FileDialog::new()
                    .set_directory(&self.base_path)
                    .pick_folder();

                if let Some(folder) = picked {
                    let slot = Slot {
                        label: folder
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .into_owned(),
                        path: folder,
                    };

                    self.carry_errand(asking.errand, &slot);
                }
            }
            Answer::Cancel => self.last_errand = None,
        }
    }

    /// The configured folders, resolved against the folder that is open.
    ///
    /// A relative destination follows the shoot rather than naming one, so a
    /// configured `Selects` means "beside these photographs" and works on
    /// every card that is ever put in.
    fn slots(&self) -> Vec<Slot> {
        self.settings
            .cull
            .destinations
            .iter()
            .filter(|destination| !destination.path.trim().is_empty())
            .map(|destination| {
                let path = PathBuf::from(&destination.path);

                Slot {
                    label: destination.label.clone(),
                    path: if path.is_absolute() {
                        path
                    } else {
                        self.base_path.join(path)
                    },
                }
            })
            .collect()
    }

    /// Sends whatever is being looked at — or picked out — to `slot`, and
    /// records how to take it back.
    ///
    /// The whole errand is one step in the journal however many photographs it
    /// carried, so a selection sent to the wrong folder comes back with one
    /// press rather than two hundred.
    fn carry_errand(&mut self, errand: Errand, slot: &Slot) {
        let paths = self.marked_paths();
        if paths.is_empty() {
            return;
        }

        if let Err(e) = std::fs::create_dir_all(&slot.path) {
            self.notices
                .say(format!("Could not make {}: {e}", slot.path.display()));
            return;
        }

        let mut moved: Vec<(PathBuf, PathBuf)> = Vec::new();
        let mut made: Vec<PathBuf> = Vec::new();
        let mut failed = 0usize;

        for from in &paths {
            let to = slot.path.join(from.file_name().unwrap_or_default());

            match errand {
                Errand::Move => match files::move_file(from, &to) {
                    Ok(()) => {
                        moved.push((to, from.clone()));
                        self.forget(from);
                    }
                    Err(e) => {
                        failed += 1;
                        self.notices.say(format!("{e}"));
                    }
                },
                Errand::Copy => match files::copy_file(from, &to) {
                    Ok(copies) => made.extend(copies),
                    Err(e) => {
                        failed += 1;
                        self.notices.say(format!("{e}"));
                    }
                },
            }
        }

        let carried = match errand {
            Errand::Move => {
                let carried = moved.len();
                self.journal.record(Step::Moved(moved));
                carried
            }
            Errand::Copy => {
                let carried = made.len();
                self.journal.record(Step::Copied(made));
                carried
            }
        };

        if carried > 0 {
            let verb = match errand {
                Errand::Move => "Moved",
                Errand::Copy => "Copied",
            };

            self.notices.say(match paths.len() {
                1 => format!("{verb} to {}", slot.label),
                _ => format!("{verb} {carried} photograph(s) to {}", slot.label),
            });
        }

        if failed > 0 {
            self.notices
                .say(format!("{failed} could not be, and are still there"));
        }

        // The photographs that were picked out have gone somewhere, so what
        // was picked out is no longer a useful thing to be holding.
        if paths.len() > 1 {
            self.grid_view.clear_selection();
        }

        self.last_destination = Some(slot.clone());
        self.last_errand = Some(errand);
    }

    /// Puts back whatever the last thing that touched a file did.
    pub(super) fn undo(&mut self) {
        let Some(step) = self.journal.peek() else {
            self.notices.say("Nothing to undo");
            return;
        };

        let said = step.describe();
        let Some(undone) = self.journal.undo() else {
            return;
        };

        for problem in &undone.failed {
            self.notices.say(format!("Could not undo: {problem}"));
        }

        for path in &undone.remarked {
            self.annotations.forget(path);
            self.refresh_mark(path);
        }

        // Anything that came back or went away changes what the folder holds,
        // and the caches are keyed by position in it.
        if !undone.moved.is_empty() || !undone.removed.is_empty() {
            let base = self.base_path.clone();
            let showing = self.marked_path();
            self.open_directory(&base, showing.as_deref());
        }

        if undone.failed.is_empty() {
            self.notices.say(format!("Undone: {said}"));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pending(names: &[&str], permanent: bool) -> Pending {
        Pending {
            paths: names.iter().map(PathBuf::from).collect(),
            permanent,
        }
    }

    #[test]
    fn one_photograph_is_named_and_several_are_counted() {
        assert!(pending(&["/photos/a.jpg"], false)
            .question()
            .contains("a.jpg"));
        assert!(pending(&["a.jpg", "b.jpg"], false)
            .question()
            .contains("2 photographs"));
    }

    #[test]
    fn deleting_for_good_says_so() {
        assert!(pending(&["a.jpg"], true)
            .question()
            .contains("cannot be undone"));
        assert!(pending(&["a.jpg"], false).question().contains("bin"));
    }
}
