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
    /// Sends the photograph on screen to the bin, or off the disk entirely.
    ///
    /// The bin needs no asking, because the bin is the asking; deleting for
    /// good does.
    pub(super) fn delete_open_image(&mut self, permanent: bool) {
        let Some(path) = self.image_view.active_path() else {
            return;
        };

        if permanent {
            self.pending_delete = Some(Pending {
                paths: vec![path],
                permanent,
            });
            return;
        }

        self.carry_out(Pending {
            paths: vec![path],
            permanent,
        });
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
        });

        // Escape is the answer people reach for without thinking, and the safe
        // one is the one it should give.
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            answered = Some(false);
        }

        match answered {
            Some(true) => {
                self.pending_delete = None;
                self.carry_out(pending);
            }
            Some(false) => self.pending_delete = None,
            None => {}
        }
    }

    /// Does it, and says what happened.
    fn carry_out(&mut self, pending: Pending) {
        let mut gone = 0usize;
        let mut failed = 0usize;

        for path in &pending.paths {
            let outcome = if pending.permanent {
                files::delete(path)
            } else {
                files::to_bin(path)
            };

            match outcome {
                Ok(()) => {
                    gone += 1;
                    self.forget(path);
                }
                Err(e) => {
                    failed += 1;
                    tracing::error!("{e}");
                    self.notices.say(format!("{e}"));
                }
            }
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
    }

    /// Takes a photograph out of the open collection.
    ///
    /// The cursor stays where it is rather than following the picture that has
    /// gone, so what it lands on is the next one — which is the single most
    /// complained about detail of culling in Lightroom.
    fn forget(&mut self, path: &Path) {
        self.paths.retain(|candidate| candidate != path);
        self.image_view.pop(path);
        self.grid_view.pop(path);
        self.annotations.forget(path);
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
