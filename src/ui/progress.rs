//! Saying that work is happening, at the foot of the window.
//!
//! Eight things the viewer does take long enough to be noticed and only two of
//! them said so, and the one that said nothing at all was the one that stopped
//! the window repainting. This is the answer to "is it stuck", which is a
//! different question from "how far along is it": most of what happens here has
//! no total to divide by.
//!
//! A determinate bar only where a total exists. `StoreStats` cannot give a
//! folder-wide percentage — `in_ram` is the length of an LRU cache and the
//! preload radius is trimmed to what the budget holds, so `in_ram < total` is
//! permanently true on any large folder and a bar driven by it would never go
//! away. The folder scan and the stack read both count what they are working
//! through, and those get a bar.

use eframe::egui::{self, Align2, RichText};

/// What is happening, in the order it is worth saying.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Doing {
    /// A folder is being walked. The count is what has been found so far;
    /// there is no total until the walk ends, which is the whole reason the
    /// walk is chunked.
    Reading(usize),
    /// Runs of frames are being read out of the folder's metadata.
    Stacking(usize, usize),
    /// Photographs are being decoded, and have been for long enough to be
    /// worth mentioning.
    Decoding(usize),
}

impl Doing {
    fn says(&self) -> String {
        match self {
            Doing::Reading(0) => "Reading the folder…".to_string(),
            Doing::Reading(found) => format!("Reading the folder — {found} photographs so far"),
            Doing::Stacking(done, total) => format!("Reading the times — {done} of {total}"),
            Doing::Decoding(1) => "Decoding one photograph…".to_string(),
            Doing::Decoding(many) => format!("Decoding {many} photographs…"),
        }
    }

    /// How far along, where that can be answered honestly.
    fn fraction(&self) -> Option<f32> {
        match self {
            Doing::Stacking(done, total) if *total > 0 => Some(*done as f32 / *total as f32),
            _ => None,
        }
    }
}

/// Draws the strip, if there is anything to say.
///
/// Anchored to the bottom and untouchable, like the notice band and for the
/// same reason: it is there while somebody is working and must never take the
/// pointer away from what they are working on.
pub fn ui(ctx: &egui::Context, doing: Option<&Doing>) {
    let Some(doing) = doing else {
        return;
    };

    egui::Area::new(egui::Id::new("progress"))
        .anchor(Align2::CENTER_BOTTOM, [0.0, -48.0])
        .interactable(false)
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            egui::Frame::popup(ui.style()).show(ui, |ui| {
                ui.horizontal(|ui| {
                    match doing.fraction() {
                        Some(part) => {
                            ui.add(
                                egui::ProgressBar::new(part)
                                    .desired_width(160.0)
                                    .show_percentage(),
                            );
                        }
                        // Indeterminate, because there is no honest total.
                        None => {
                            ui.add(egui::Spinner::new().size(14.0));
                        }
                    }

                    ui.label(RichText::new(doing.says()).small());
                });
            });
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A walk has no total, so it gets a spinner and a count of what it has
    /// found — never a bar that could only be a guess.
    #[test]
    fn a_walk_has_no_percentage() {
        assert_eq!(Doing::Reading(1200).fraction(), None);
        assert!(Doing::Reading(1200).says().contains("1200"));
    }

    /// The stack read counts what it is working through, so it can say.
    #[test]
    fn the_stack_read_has_one() {
        assert_eq!(Doing::Stacking(50, 200).fraction(), Some(0.25));
    }

    /// And a total of nothing is not a division.
    #[test]
    fn nothing_to_read_is_not_a_percentage() {
        assert_eq!(Doing::Stacking(0, 0).fraction(), None);
    }

    /// One photograph is one photograph.
    #[test]
    fn the_decoder_counts_in_photographs() {
        assert_eq!(Doing::Decoding(1).says(), "Decoding one photograph…");
        assert!(Doing::Decoding(7).says().contains('7'));
    }
}
