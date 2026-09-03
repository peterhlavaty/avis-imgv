//! Sending photographs somewhere else, and taking it back.
//!
//! The other half of a first pass: the frames that are staying go to a folder
//! of selects rather than to the bin, and one keystroke has to carry a raw, its
//! JPEG twin and both their sidecars. Kept apart from the deleting for the
//! reason the whole directory exists — the two are one third of `app/` between
//! them, and only one of them can lose a photograph.

use std::path::PathBuf;

use crate::history::{Deed, Step};
use crate::organize::files;
use crate::ui::destinations::{Answer, Asking, Errand, Slot};

use crate::app::App;

impl App {
    /// Opens the panel that asks where the photograph should go.
    ///
    /// Pressing the same key twice in a row skips the panel and repeats the
    /// last answer, which is the motion every fast viewer has: the panel is
    /// there for the first frame of a shoot and out of the way for the rest.
    pub(in crate::app) fn send_somewhere(&mut self, errand: Errand) {
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
    pub(in crate::app) fn send_to_rejected(&mut self) {
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

    /// Does what the card was told.
    pub(in crate::app) fn carry_destination(&mut self, asking: &Asking, answer: Answer) {
        self.close_modal();

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
            Answer::Remember => {
                let picked = rfd::FileDialog::new()
                    .set_directory(&self.base_path)
                    .pick_folder();

                if let Some(folder) = picked {
                    let label = folder
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .into_owned();

                    self.settings
                        .cull
                        .destinations
                        .push(crate::config::Destination {
                            label: label.clone(),
                            path: folder.display().to_string(),
                        });
                    self.save_settings();

                    let slot = Slot {
                        label,
                        path: folder,
                    };
                    self.notices
                        .say(format!("Kept \"{}\" as a destination.", slot.label));
                    self.carry_errand(asking.errand, &slot);
                }
            }
            Answer::Settings => self.open_settings_at("cull.destinations"),
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
    /// The whole errand is one step in the history however many photographs it
    /// carried, so a selection sent to the wrong folder comes back with one
    /// press rather than two hundred.
    fn carry_errand(&mut self, errand: Errand, slot: &Slot) {
        let shown = self.marked_paths();
        if shown.is_empty() {
            return;
        }

        // The raw goes with its JPEG: a folder of selects holding only half of
        // each pair is a folder somebody has to fix by hand later.
        let paths: Vec<PathBuf> = shown
            .iter()
            .flat_map(|path| self.with_partners(path))
            .collect();

        if let Err(e) = std::fs::create_dir_all(&slot.path) {
            self.notices
                .say(format!("Could not make {}: {e}", slot.path.display()));
            return;
        }

        let mut moved: Vec<(PathBuf, PathBuf)> = Vec::new();
        // Both halves of a copy: the photographs asked for, so it can be made
        // again, and everything that actually appeared, so it can be taken away.
        let mut pairs: Vec<(PathBuf, PathBuf)> = Vec::new();
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
                    Ok(copies) => {
                        pairs.push((from.clone(), to));
                        made.extend(copies);
                    }
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
                self.history.record(Deed::Files(Step::Moved(moved)));
                carried
            }
            Errand::Copy => {
                let carried = made.len();
                self.history
                    .record(Deed::Files(Step::Copied { pairs, made }));
                carried
            }
        };

        if carried > 0 {
            let verb = match errand {
                Errand::Move => "Moved",
                Errand::Copy => "Copied",
            };

            self.notices.say(match shown.len() {
                1 => format!("{verb} to {}", slot.label),
                _ => format!("{verb} {} photograph(s) to {}", shown.len(), slot.label),
            });
        }

        if failed > 0 {
            self.notices
                .say(format!("{failed} could not be, and are still there"));
        }

        // The photographs that were picked out have gone somewhere, so what
        // was picked out is no longer a useful thing to be holding.
        if shown.len() > 1 {
            self.grid_view.clear_selection();
        }

        self.last_destination = Some(slot.clone());
        self.last_errand = Some(errand);
    }
}
