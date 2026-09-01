//! What a deletion is, and the question it asks before it happens.
//!
//! Split from the doing next door because the two answer different people:
//! this is everything the user sees before anything reaches the disk — where
//! the photographs are going, how that is worded, and which of the four
//! answers switches the question off.

use std::path::PathBuf;

use eframe::egui;

use crate::app::App;

/// Where a deletion sends the photographs.
///
/// Three answers rather than a boolean, because the viewer's own bin is a
/// third thing and not a shade of either: it can be taken back like the
/// platform's, and it is a folder on a disk like a move. The question, the
/// window's title and what is written to the history all read off this, so
/// there is one place that decides and nowhere for the three to disagree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Sends {
    /// To the platform's bin.
    ToTheSystemBin,
    /// To the folder the viewer keeps, which can be opened and looked in.
    ToTheFolder(PathBuf),
    /// Off the disk.
    ForGood,
    /// Everything in the viewer's own bin, off the disk, folder and all.
    ///
    /// No file list: emptying is one `remove_dir_all` and takes whatever is in
    /// there, including anything dragged in by hand.
    EmptyingTheBin(PathBuf),
}

impl Sends {
    /// Whether this is the answer nobody can take back.
    pub(super) fn permanent(&self) -> bool {
        matches!(self, Sends::ForGood | Sends::EmptyingTheBin(_))
    }

    /// What the window asking about it is called.
    fn title(&self) -> &'static str {
        match self {
            Sends::EmptyingTheBin(_) => "Empty the bin",
            Sends::ForGood => "Delete for good",
            _ => "Move to the bin",
        }
    }
}

/// A deletion the user has been asked about but has not answered.
///
/// Not every deletion becomes one: sending a single frame to the bin is
/// answerable with the bin itself, and a dialogue in the middle of a cull is
/// what people complain about most in the tools that have one. What is asked
/// about is anything permanent, anything taking a whole selection, and
/// emptying the bin — see [`Pending::asks_first`].
#[derive(Debug, Clone)]
pub struct Pending {
    /// Every file that will go, both halves of a pair included.
    pub paths: Vec<PathBuf>,
    /// Where they are going.
    pub sends: Sends,
    /// How many *photographs* that is, which is what the question says: a
    /// raw+JPEG pair is two files and one picture.
    pub photographs: usize,
}

impl Pending {
    fn question(&self) -> String {
        let count = self.photographs;

        if let Sends::EmptyingTheBin(root) = &self.sends {
            return format!(
                "Empty the bin? {} in {} will be deleted for good, and this cannot \
                 be undone.",
                match count {
                    1 => "The photograph".to_string(),
                    many => format!("All {many} photographs"),
                },
                root.display()
            );
        }

        let what = if count == 1 {
            self.paths
                .first()
                .and_then(|path| path.file_name())
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_else(|| "this photograph".to_string())
        } else {
            format!("{count} photographs")
        };

        if self.sends.permanent() {
            format!("Delete {what} for good? This cannot be undone.")
        } else {
            format!("Send {what} to the bin?")
        }
    }

    /// Whether this is asked about before it is carried out.
    ///
    /// `configured` is whichever row of `cull.confirm` governs the caller, and
    /// it can only ever switch off a question about something reversible:
    /// anything permanent asks whatever it says. A confirmation is not a
    /// substitute for reversibility, and switching one off is not a substitute
    /// for either.
    pub(super) fn asks_first(&self, configured: bool) -> bool {
        self.sends.permanent() || configured
    }
}

impl App {
    /// Draws the question, if there is one outstanding.
    pub(in crate::app) fn show_pending_delete(&mut self, ctx: &egui::Context) {
        let Some(pending) = self.pending_delete.clone() else {
            return;
        };

        let mut answered = None;

        let shown = egui::Window::new(pending.sends.title())
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
                    egui::RichText::new(if pending.sends.permanent() {
                        "Y to delete · Escape to leave them alone"
                    } else {
                        "Enter or Y to send them · Escape to leave them alone"
                    })
                    .weak(),
                );
            });

        crate::utils::in_front(ctx, shown.as_ref());

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
            yes | (!pending.sends.permanent()
                && i.consume_key(egui::Modifiers::NONE, egui::Key::Enter))
        });

        if said_no {
            answered = Some(false);
        } else if said_yes {
            answered = Some(true);
        }

        match answered {
            Some(true) => {
                self.close_modal();
                self.carry_out(pending);
            }
            Some(false) => self.close_modal(),
            None => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pending(names: &[&str], permanent: bool) -> Pending {
        Pending {
            paths: names.iter().map(PathBuf::from).collect(),
            sends: if permanent {
                Sends::ForGood
            } else {
                Sends::ToTheSystemBin
            },
            photographs: names.len(),
        }
    }

    /// A raw and a JPEG shot together are two files and one picture, and the
    /// question has to say so or it reads as though twice as much is going.
    #[test]
    fn a_pair_is_counted_as_one_photograph() {
        let pending = Pending {
            paths: vec![
                PathBuf::from("/photos/IMG_1.JPG"),
                PathBuf::from("/photos/IMG_1.CR2"),
            ],
            sends: Sends::ToTheSystemBin,
            photographs: 1,
        };

        assert!(
            pending.question().contains("IMG_1.JPG"),
            "{}",
            pending.question()
        );
        assert!(!pending.question().contains('2'), "{}", pending.question());
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

    /// The viewer's own bin is still a bin: the question says so, and Enter
    /// answers it, because it can be taken back.
    #[test]
    fn the_viewers_own_bin_asks_the_same_question_as_the_platforms() {
        let pending = Pending {
            paths: vec![PathBuf::from("/photos/a.jpg")],
            sends: Sends::ToTheFolder(PathBuf::from("/data/bin")),
            photographs: 1,
        };

        assert!(!pending.sends.permanent());
        assert_eq!(pending.sends.title(), "Move to the bin");
        assert!(pending.question().contains("bin"), "{}", pending.question());
    }

    /// A confirmation that can be switched off may only ever be one about
    /// something that can be taken back.
    #[test]
    fn nothing_permanent_can_be_made_to_happen_without_a_question() {
        for sends in [
            Sends::ForGood,
            Sends::EmptyingTheBin(PathBuf::from("/data/bin")),
        ] {
            let pending = Pending {
                paths: vec![PathBuf::from("a.jpg")],
                sends,
                photographs: 1,
            };

            assert!(pending.asks_first(false), "{:?}", pending.sends);
        }
    }

    /// And one about something that can is the setting's to make.
    #[test]
    fn a_reversible_deletion_asks_only_where_the_setting_says_so() {
        let pending = pending(&["a.jpg", "b.jpg"], false);

        assert!(pending.asks_first(true));
        assert!(!pending.asks_first(false));
    }

    /// Emptying names the folder, says how much is going, and is one of the
    /// two answers Enter must not give.
    #[test]
    fn emptying_names_the_folder_and_the_count() {
        let pending = Pending {
            paths: Vec::new(),
            sends: Sends::EmptyingTheBin(PathBuf::from("/data/bin")),
            photographs: 12,
        };

        let question = pending.question();

        assert!(pending.sends.permanent());
        assert_eq!(pending.sends.title(), "Empty the bin");
        assert!(question.contains("12"), "{question}");
        assert!(question.contains("bin"), "{question}");
        assert!(question.contains("cannot be undone"), "{question}");
    }
}
