//! The status bar under the image: position, name, zoom.

use eframe::egui::{self, Sense};
use eframe::epaint::Vec2;

use crate::metadata::xmp::{Flag, Label, Xmp};

use super::input::Command;

/// Zoom levels offered in the magnification context menu.
const PERCENTAGES: &[f32] = &[200., 100., 75., 50., 25.];

/// What the user has said about the photograph on screen.
///
/// Drawn in the bar so that rating, flagging or labelling with the panel shut
/// is not a keystroke that appears to do nothing.
#[derive(Debug, Clone, Default)]
pub struct Marks {
    pub stars: u8,
    pub flag: Flag,
    pub label: Option<Label>,
    /// Kept here as well as in the annotation store, because the filter asks
    /// about every photograph in the folder at once and a lookup per file per
    /// keystroke is the thing this list exists to avoid.
    pub keywords: Vec<String>,
}

impl Marks {
    pub fn of(annotations: &Xmp) -> Marks {
        Marks {
            stars: annotations.stars(),
            flag: annotations.flag(),
            label: annotations.known_label(),
            keywords: annotations.keywords.clone(),
        }
    }
}

/// Modes worth telling the user about.
#[derive(Debug, Clone, Copy, Default)]
pub struct Flags {
    pub flattened: bool,
    pub watching: bool,
    pub filling: bool,
    /// Whether a mark moves on to the next photograph by itself.
    pub advancing: bool,
}

/// Everything the bar draws, borrowed from the view.
pub struct Status<'a> {
    pub jump_to: &'a mut String,
    pub zoom: &'a mut f32,
    /// One based, as shown to the user.
    pub position: usize,
    pub total: usize,
    /// How many the filter is holding back, so a shorter collection is not a
    /// mystery.
    pub hidden: usize,
    pub name: String,
    pub percentage_zoom: f32,
    pub marks: Marks,
    pub flags: Flags,
}

/// What the user asked for by clicking in the bar.
#[derive(Debug, Default)]
pub struct Outcome {
    pub commands: Vec<Command>,
    /// A zero based index typed into the jump field.
    pub jump_to: Option<usize>,
}

/// Draws the bar and reports the interactions it produced.
pub fn ui(ctx: &egui::Context, status: &mut Status<'_>) -> Outcome {
    let mut outcome = Outcome::default();

    egui::TopBottomPanel::bottom("image_view_bottom_bar")
        .show_separator_line(false)
        .show(ctx, |ui| {
            ui.horizontal_centered(|ui| {
                outcome.jump_to = jump_field(ui, status);

                let counted = match status.hidden {
                    0 => format!("{}/{}", status.position, status.total),
                    hidden => format!("{}/{} (+{hidden})", status.position, status.total),
                };

                ui.add_sized(
                    Vec2::new(
                        if status.hidden == 0 { 45. } else { 90. },
                        ui.available_height(),
                    ),
                    egui::Label::new(counted),
                )
                .on_hover_text(match status.hidden {
                    0 => String::new(),
                    hidden => format!("{hidden} more are hidden by the filter"),
                });

                for (active, label) in [
                    (status.flags.flattened, "Flattened"),
                    (status.flags.watching, "Watching"),
                    (status.flags.filling, "Filling"),
                    (status.flags.advancing, "Advancing"),
                ] {
                    if active {
                        ui.label(label);
                    }
                }

                marks(ui, &status.marks);

                // Leave room for the zoom controls pinned to the right.
                let name_width = (ui.available_width() - 245.).max(20.);
                ui.add_sized(
                    Vec2::new(name_width, ui.available_height()),
                    egui::Label::new(status.name.clone()).truncate(),
                );

                ui.with_layout(
                    egui::Layout::right_to_left(eframe::emath::Align::Max),
                    |ui| {
                        ui.add_sized(
                            Vec2::new(200., ui.available_height()),
                            egui::Slider::new(status.zoom, 0.1..=10.0).text("🔎"),
                        );

                        outcome
                            .commands
                            .extend(zoom_label(ui, status.percentage_zoom));
                    },
                );
            });
        });

    outcome
}

/// The three marks, drawn only when there is something to draw: the bar is a
/// summary, not a control, and an unmarked photograph should say nothing.
fn marks(ui: &mut egui::Ui, marks: &Marks) {
    if marks.flag != Flag::Unflagged {
        let colour = match marks.flag {
            Flag::Rejected => egui::Color32::from_rgb(219, 96, 96),
            _ => ui.visuals().text_color(),
        };

        ui.label(egui::RichText::new(marks.flag.glyph()).color(colour));
    }

    if let Some(label) = marks.label {
        let (r, g, b) = label.colour();
        ui.label(egui::RichText::new("■").color(egui::Color32::from_rgb(r, g, b)))
            .on_hover_text(label.name());
    }

    if marks.stars > 0 {
        ui.label(stars(marks.stars));
    }
}

/// A rating as filled stars, without the empty ones.
fn stars(rating: u8) -> String {
    "★".repeat(rating as usize)
}

fn jump_field(ui: &mut egui::Ui, status: &mut Status<'_>) -> Option<usize> {
    let response = ui.add_sized(
        Vec2::new(65., ui.available_height()),
        egui::TextEdit::singleline(status.jump_to),
    );

    if !(response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))) {
        return None;
    }

    let typed = status.jump_to.parse::<usize>().ok();
    status.jump_to.clear();

    // The field is one based; positions outside the collection are ignored.
    typed
        .filter(|position| (1..=status.total).contains(position))
        .map(|position| position - 1)
}

fn zoom_label(ui: &mut egui::Ui, percentage_zoom: f32) -> Vec<Command> {
    let mut commands = Vec::new();

    let response = ui.add_sized(
        Vec2::new(45., ui.available_height()),
        egui::Label::new(format!("{percentage_zoom:.1}%")).sense(Sense::click()),
    );

    response.context_menu(|ui| {
        for (label, command) in [
            ("Fit to screen", Command::Fit),
            ("Fill screen", Command::Fill),
            ("Fit horizontal", Command::FitHorizontal),
            ("Fit vertical", Command::FitVertical),
        ] {
            if ui.button(label).clicked() {
                commands.push(command);
                ui.close();
            }
        }

        ui.separator();

        for percentage in PERCENTAGES {
            if ui.button(format!("{percentage:.0}%")).clicked() {
                commands.push(Command::ZoomToPercent(*percentage));
                ui.close();
            }
        }
    });

    commands
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_rating_is_shown_as_filled_stars() {
        assert_eq!(stars(0), "");
        assert_eq!(stars(3), "★★★");
        assert_eq!(stars(5).chars().count(), 5);
    }
}
