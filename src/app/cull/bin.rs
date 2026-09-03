//! The viewer's own bin, from the program's side.
//!
//! [`crate::organize::bin`] is the folder and the note it keeps; this is what
//! the viewer does with them — where the bin is, whether the folder on screen
//! *is* it, opening it, putting a photograph back, emptying it, and the one
//! question asked on the way out.
//!
//! The last of those is the reason a folder bin is worth having at all. A bin
//! that fills up quietly and is never looked in is a folder of photographs
//! nobody will ever decide about, so the viewer says on the way out that there
//! is something in it, and offers to be rid of it. It is a question and not a
//! rule: somebody who uses the bin as a holding folder turns it off, and
//! emptying is confirmed either way.

use std::path::{Path, PathBuf};

use eframe::egui;

use crate::app::App;
use crate::history::{Deed, Step};
use crate::organize::{bin, files};

use super::{Pending, Sends};

/// How far the viewer has got with being closed.
///
/// Closing is not one event: the request arrives, the question goes up, the
/// answer comes back and only then may the window actually go. Without
/// somewhere to write that down the question would be asked again on the frame
/// after it was answered, because the close it sends is a close request like
/// any other.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum Leaving {
    /// Nothing has asked to close.
    #[default]
    No,
    /// The question is up, about this bin and this many photographs.
    Asking(PathBuf, usize),
    /// Answered. The next close goes through.
    Yes,
}

impl App {
    /// Where the viewer's own bin is, whether or not it is the one in use.
    ///
    /// The folder named in the configuration, or the one beside the viewer's
    /// own files when it names none. Always absolute: a path taken against the
    /// open folder would be a different bin in every shoot, and the question
    /// asked on the way out would be about whichever one happened to be open.
    /// A relative one is refused there and complained about at load.
    pub(in crate::app) fn bin_root(&self) -> Option<PathBuf> {
        bin::root_from(self.settings.cull.bin_folder.as_deref())
    }

    /// The bin, when it is the one the delete key means.
    pub(in crate::app) fn bin_folder(&self) -> Option<PathBuf> {
        (self.settings.cull.bin == "folder")
            .then(|| self.bin_root())
            .flatten()
    }

    /// Whether the folder on screen is the bin, or somewhere inside it.
    ///
    /// Asked of the configured folder whichever bin is in use: a folder full
    /// of things this viewer threw out is the bin whether or not the *next*
    /// thing thrown out would go there.
    pub(in crate::app) fn in_the_bin(&self) -> bool {
        self.bin_root()
            .is_some_and(|root| bin::is_inside(&root, &self.base_path))
    }

    /// Opens the bin like any other folder, which is the whole idea of it.
    pub(in crate::app) fn open_bin(&mut self) {
        let Some(root) = self.bin_root() else {
            self.notices
                .warn("This machine has nowhere for the viewer to keep a bin folder.");
            return;
        };

        // Made rather than reported missing: a bin nothing has been thrown
        // into yet is an empty bin, and an empty folder is what that looks
        // like.
        if let Err(e) = std::fs::create_dir_all(&root) {
            self.notices
                .warn(format!("Could not make {}: {e}", root.display()));
            return;
        }

        self.notices.say(format!("Opening {}", root.display()));
        self.open_directory(&root, None);
    }

    /// Takes what is on screen out of the bin and puts it back where it came
    /// from.
    ///
    /// Recorded as an ordinary move, because that is what it is. The note the
    /// bin keeps is left alone rather than tidied: a row whose file has gone is
    /// invisible to everything that reads it and live again the moment the file
    /// returns, so undoing this puts the photograph back in the bin with its
    /// origin still known and nothing had to be told.
    pub(in crate::app) fn put_back(&mut self) {
        let Some(root) = self.bin_root() else {
            return;
        };

        let shown = self.marked_paths();
        if shown.is_empty() {
            return;
        }

        // The raw goes with its JPEG here as everywhere else: they went in
        // together and half a photograph coming back is worse than none.
        let paths: Vec<PathBuf> = shown
            .iter()
            .flat_map(|path| self.with_partners(path))
            .collect();

        let mut moved: Vec<(PathBuf, PathBuf)> = Vec::new();
        let mut failed = 0usize;
        let mut unknown = 0usize;

        for path in &paths {
            let Some(home) = bin::came_from(&root, path) else {
                unknown += 1;
                continue;
            };

            match files::move_file(path, &home) {
                Ok(()) => {
                    moved.push((home, path.clone()));
                    self.forget(path);
                }
                Err(e) => {
                    failed += 1;
                    self.notices.say(format!("{e}"));
                }
            }
        }

        self.history.record(Deed::Files(Step::Moved(moved.clone())));

        match moved.len() {
            0 => {}
            1 => self.notices.say(format!(
                "Put back in {}",
                folder_of(&moved[0].0).unwrap_or_else(|| "its folder".to_string())
            )),
            many => self.notices.say(format!("Put {many} photograph(s) back")),
        }

        if unknown > 0 {
            self.notices.say(match paths.len() {
                1 => "The bin does not say where this one came from".to_string(),
                _ => format!("The bin does not say where {unknown} of them came from"),
            });
        }

        if failed > 0 {
            self.notices
                .say(format!("{failed} could not be, and are still in the bin"));
        }

        if shown.len() > 1 {
            self.grid_view.clear_selection();
        }
    }

    /// Asks before emptying the bin. Every emptying is asked about.
    pub(in crate::app) fn ask_to_empty_the_bin(&mut self) {
        let Some(root) = self.bin_root() else {
            return;
        };

        let held = bin::count(&root);
        if held == 0 {
            self.notices.say("The bin is empty");
            return;
        }

        self.pending_delete = Some(Pending {
            paths: Vec::new(),
            sends: Sends::EmptyingTheBin(root),
            photographs: held,
        });
    }

    /// Empties it, and puts the view right if it was looking at it.
    pub(in crate::app) fn empty_the_bin(&mut self, root: &Path) {
        match bin::empty(root) {
            Ok(gone) => {
                self.notices
                    .say(format!("Emptied the bin: {gone} photograph(s) deleted"));

                // The folder on screen has just lost everything in it, and the
                // caches are keyed by position in the collection.
                if self.in_the_bin() {
                    let base = self.base_path.clone();
                    self.open_directory(&base, None);
                }
            }
            Err(e) => self.notices.warn(format!("{e}")),
        }
    }

    /// Holds the viewer open long enough to ask about a bin with something in
    /// it, once.
    ///
    /// The close is cancelled rather than deferred, and sent again from the
    /// answer: eframe closes on the frame the request arrives unless it is
    /// told not to, and there is no way to hold one pending across the frames
    /// a question takes to answer.
    pub(in crate::app) fn consider_leaving(&mut self, ctx: &egui::Context) {
        if self.leaving == Leaving::No {
            if !ctx.input(|i| i.viewport().close_requested()) {
                return;
            }

            let Some(waiting) = self.bin_worth_asking_about() else {
                self.leaving = Leaving::Yes;
                return;
            };

            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.leaving = Leaving::Asking(waiting.0, waiting.1);
        }
    }

    /// The bin and what is in it, where that is worth stopping for.
    ///
    /// Nothing to ask about when the folder bin is not in use, when the
    /// setting says not to ask, when the bin is empty — or when the viewer is
    /// being run to measure itself, which closes on its own and must not stop
    /// to talk to anybody.
    fn bin_worth_asking_about(&self) -> Option<(PathBuf, usize)> {
        if !self.settings.cull.ask_to_empty_the_bin || self.benchmark.is_some() {
            return None;
        }

        let root = self.bin_folder()?;
        let held = bin::count(&root);

        (held > 0).then_some((root, held))
    }

    /// Draws that question, and obeys it.
    pub(in crate::app) fn ask_about_leaving(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        let ctx = &ctx;

        let Leaving::Asking(root, held) = self.leaving.clone() else {
            return;
        };

        // Empty it and close, close and keep it, or neither. Three answers
        // rather than two, because Escape is reached for without thinking and
        // the thing it should undo here is the closing.
        let mut empty_it = None;

        {
            ui.label(format!(
                "{} still in {}. Empty it before closing?",
                match held {
                    1 => "There is 1 photograph".to_string(),
                    many => format!("There are {many} photographs"),
                },
                root.display()
            ));
            ui.add_space(10.0);

            ui.horizontal(|ui| {
                if ui.button("Empty it and close").clicked() {
                    empty_it = Some(true);
                }
                if ui.button("Keep it and close").clicked() {
                    empty_it = Some(false);
                }
                if ui.button("Do not close").clicked() {
                    self.leaving = Leaving::No;
                }
            });

            ui.add_space(6.0);
            ui.label(
                egui::RichText::new("Y to empty it · N to keep it · Escape to stay open").weak(),
            );
        }

        // Consumed rather than read, for the reason the deletion question
        // consumes its keys: the views draw after this and would go on to mean
        // whatever the key means the rest of the time.
        let (escaped, yes, no) = ctx.input_mut(|i| {
            (
                i.consume_key(egui::Modifiers::NONE, egui::Key::Escape),
                i.consume_key(egui::Modifiers::NONE, egui::Key::Y),
                i.consume_key(egui::Modifiers::NONE, egui::Key::N),
            )
        });

        if escaped {
            self.leaving = Leaving::No;
            return;
        }

        if yes {
            empty_it = Some(true);
        } else if no {
            empty_it = Some(false);
        }

        let Some(empty_it) = empty_it else {
            return;
        };

        if empty_it {
            self.empty_the_bin(&root);
        }

        self.leaving = Leaving::Yes;
        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
    }
}

/// What the folder a photograph went back to is called.
fn folder_of(path: &Path) -> Option<String> {
    Some(path.parent()?.file_name()?.to_string_lossy().into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nothing_is_asked_until_something_asks_to_close() {
        assert_eq!(Leaving::default(), Leaving::No);
    }

    #[test]
    fn a_photograph_says_which_folder_it_went_back_to() {
        assert_eq!(
            folder_of(Path::new("/photos/holiday/a.jpg")),
            Some("holiday".to_string())
        );
        assert_eq!(folder_of(Path::new("a.jpg")), None);
    }
}
