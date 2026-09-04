//! Taking photographs out of the folder.
//!
//! The verb a culling tool is for and the one this viewer had no answer to:
//! everything else here decides what a picture *is*, and this decides whether
//! it stays. Kept apart from the rest of the wiring for the same reason the
//! tagging is — it is one of the two places that touch the user's files.

mod asking;
mod bin;
mod destinations;

pub use asking::{Pending, Sends};
pub use bin::Leaving;

use std::path::{Path, PathBuf};

use crate::history::{Deed, Step};
use crate::metadata::xmp::Flag;
use crate::organize::{bin as folder_bin, files};

use super::App;

impl App {
    /// Sends what is being looked at — or what is picked out — to the bin, or
    /// off the disk entirely.
    ///
    /// One photograph to the bin needs no asking, because the bin is the
    /// asking. Deleting for good does, and so does taking a whole selection:
    /// the cost of a wrong keystroke there is a folder rather than a frame.
    pub(super) fn delete_open_image(&mut self, permanent: bool) {
        let shown = self.marked_paths();
        if shown.is_empty() {
            return;
        }

        // Both halves of a raw+JPEG pair go, because they are one photograph:
        // deleting the JPEG and leaving the raw is how a frame somebody threw
        // out comes back on the next card read.
        let paths: Vec<PathBuf> = shown
            .iter()
            .flat_map(|path| self.with_partners(path))
            .collect();

        let pending = Pending {
            sends: self.sends(permanent),
            paths,
            photographs: shown.len(),
        };

        // `cull.confirm.bin_several` is the half that can be switched off, and
        // until now it was a row in the registry nothing ever read: the count
        // was compared against one in the code and the answer ignored.
        let several = pending.photographs > 1 && self.settings.cull.confirm.bin_several;

        if pending.asks_first(several) {
            self.pending_delete = Some(pending);
            return;
        }

        self.carry_out(pending);
    }

    /// Where a deletion of what is on screen should send it.
    ///
    /// Inside the bin the ordinary delete *is* the permanent one: there is
    /// nowhere further to send something that is already in the bin, and
    /// deleting out of a bin is what Delete does in every file manager there
    /// is. It still asks, because everything permanent does.
    fn sends(&self, permanent: bool) -> Sends {
        if permanent || self.in_the_bin() {
            return Sends::ForGood;
        }

        match self.bin_folder() {
            Some(root) => Sends::ToTheFolder(root),
            None => Sends::ToTheSystemBin,
        }
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

        let photographs = paths.len();
        let paths: Vec<PathBuf> = paths
            .iter()
            .flat_map(|path| self.with_partners(path))
            .collect();

        let pending = Pending {
            paths,
            sends: self.sends(false),
            photographs,
        };

        // The whole sweep behind one question, however few it found, because
        // it is a bulk verb: `cull.confirm.empty_rejects` is the row that
        // governs it, and it too was never read.
        if pending.asks_first(self.settings.cull.confirm.empty_rejects) {
            self.pending_delete = Some(pending);
            return;
        }

        self.carry_out(pending);
    }

    /// Nothing is being asked any more.
    pub(in crate::app) fn close_modal(&mut self) {
        self.pending_delete = None;
        self.asking = None;
    }

    /// Does it, and says what happened.
    fn carry_out(&mut self, pending: Pending) {
        if let Sends::EmptyingTheBin(root) = &pending.sends {
            self.empty_the_bin(&root.clone());
            return;
        }

        let root = match &pending.sends {
            Sends::ToTheFolder(root) => Some(root.clone()),
            _ => None,
        };

        let mut gone = 0usize;
        let mut failed = 0usize;
        // The platform's bin remembers where things came from itself; the
        // viewer's own has to be told, and both halves of that are the same
        // list read the two ways round.
        let mut binned: Vec<PathBuf> = Vec::new();
        let mut arrivals: Vec<(PathBuf, PathBuf)> = Vec::new();
        let mut interred: Vec<(PathBuf, PathBuf)> = Vec::new();

        for path in &pending.paths {
            let outcome = match (&root, pending.sends.permanent()) {
                (Some(root), _) => folder_bin::room_for(root, path)
                    .and_then(|inside| files::move_file(path, &inside).map(|()| Some(inside))),
                (None, true) => files::delete(path).map(|()| None),
                (None, false) => files::to_bin(path).map(|()| None),
            };

            match outcome {
                Ok(landed) => {
                    gone += 1;
                    self.forget(path);

                    match landed {
                        Some(inside) => {
                            arrivals.push((path.clone(), inside.clone()));
                            interred.push((inside, path.clone()));
                        }
                        None => binned.push(path.clone()),
                    }
                }
                Err(e) => {
                    failed += 1;
                    tracing::error!("{e}");
                    self.notices.say(format!("{e}"));
                }
            }
        }

        // Written once for the batch rather than once a file: it is one
        // document, and a folder of two hundred rejects would otherwise
        // rewrite it two hundred times.
        if let Some(root) = &root {
            if let Err(e) = folder_bin::note(root, &arrivals) {
                self.notices.warn(format!("{e}"));
            }
        }

        // Only a bin can be undone: what was deleted for good is gone, and the
        // history must not suggest otherwise.
        match &pending.sends {
            Sends::ToTheFolder(_) => {
                self.history.record(Deed::Files(Step::Interred(interred)));
            }
            Sends::ToTheSystemBin => {
                self.history.record(Deed::Files(Step::Binned(binned)));
            }
            Sends::ForGood | Sends::EmptyingTheBin(_) => {}
        }

        if gone > 0 {
            let where_to = if pending.sends.permanent() {
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

        if pending.photographs > 1 {
            self.grid_view.clear_selection();
        }
    }

    /// Takes a photograph out of the open collection.
    ///
    /// The cursor stays where it is rather than following the picture that has
    /// gone, so what it lands on is the next one — which is the single most
    /// complained about detail of culling in Lightroom.
    pub(super) fn forget(&mut self, path: &Path) {
        self.pairs.forget(path);

        if let Some(index) = self.paths.iter().position(|candidate| candidate == path) {
            self.drop_mark(index);
            self.paths.remove(index);
            // The runs hold store positions, so everything below the gap has
            // just moved. Nothing else notices: the caches shift themselves
            // and the detector only runs on a folder change, so a stack left
            // untold goes on naming frames that are one photograph away.
            self.stacking.remove_shifting(index);
        }

        self.image_view.pop(path);
        self.grid_view.pop(path);
        self.annotations.forget(path);
    }
}
