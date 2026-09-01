//! The legend for a visual language that had none.
//!
//! The viewer draws four glyphs on a stack, three states of badge under a cell,
//! two overlay colours and a border round the focused pane, and says what none
//! of them mean anywhere a person can read. `Overlay::label` has existed since
//! the overlays were written and is called by a test alone.

use eframe::egui;

use crate::decoder::overlays::Overlay;
use crate::organize::group::Kind;
use crate::view::grid_view::cell::Badges;
use crate::view::stacks;

/// Draws the legend window.
pub fn ui(ctx: &egui::Context, open: &mut bool) {
    let shown = egui::Window::new("What the marks mean")
        .open(open)
        .default_width(540.0)
        .default_height(520.0)
        .show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.heading("On a stack");
                ui.label(
                    "A cell standing for a run of frames wears a glyph saying what kind of \
                     run it is, and the count beside it.",
                );
                ui.add_space(4.0);

                for kind in Kind::ALL {
                    row(ui, stacks::glyph(*kind), kind.label(), sentence(*kind));
                }

                ui.add_space(10.0);
                ui.heading("Under a cell");
                ui.label("What the strip under a thumbnail shows. The key cycles the three.");
                ui.add_space(4.0);

                for badges in Badges::ALL {
                    row(ui, "", badges_label(*badges), badges_sentence(*badges));
                }

                ui.add_space(10.0);
                ui.heading("On the photograph");
                ui.label(
                    "The overlay paints over the picture rather than beside it, so what it \
                     marks is where it is.",
                );
                ui.add_space(4.0);

                for overlay in Overlay::ALL {
                    row(ui, "", overlay.label(), overlay_sentence(*overlay));
                }

                ui.add_space(4.0);
                swatch(ui, (255, 40, 40), "Blown", "Nothing left in the highlight: it has gone to white and no development brings it back.");
                swatch(ui, (60, 120, 255), "Crushed", "Nothing left in the shadow: it has gone to black.");
                swatch(ui, (120, 255, 90), "In focus", "Where the edges are sharpest, which is where the focus fell.");

                ui.add_space(10.0);
                ui.heading("Round a pane");
                ui.label(
                    "While photographs are pinned side by side, one of them has the keyboard. \
                     Its pane carries a border, and the keys act on that one.",
                );

                ui.add_space(10.0);
                ui.heading("In the status bar");
                ui.label(
                    "A star is a rating, a flag is a keep or a reject, and a coloured square \
                     is a colour label. Each of them can be clicked.",
                );
            });
        });

    crate::utils::in_front(ctx, shown.as_ref());
}

/// One row: a glyph, its name, and what it means.
fn row(ui: &mut egui::Ui, glyph: &str, name: &str, meaning: &str) {
    ui.horizontal(|ui| {
        ui.add_sized(
            egui::Vec2::new(24.0, ui.spacing().interact_size.y),
            egui::Label::new(egui::RichText::new(glyph).size(16.0)),
        );
        ui.add_sized(
            egui::Vec2::new(110.0, ui.spacing().interact_size.y),
            egui::Label::new(egui::RichText::new(name).strong()),
        );
        ui.label(meaning);
    });
}

/// One row whose glyph is a colour rather than a shape.
fn swatch(ui: &mut egui::Ui, (r, g, b): (u8, u8, u8), name: &str, meaning: &str) {
    ui.horizontal(|ui| {
        ui.add_sized(
            egui::Vec2::new(24.0, ui.spacing().interact_size.y),
            egui::Label::new(
                egui::RichText::new("■")
                    .size(16.0)
                    .color(egui::Color32::from_rgb(r, g, b)),
            ),
        );
        ui.add_sized(
            egui::Vec2::new(110.0, ui.spacing().interact_size.y),
            egui::Label::new(egui::RichText::new(name).strong()),
        );
        ui.label(meaning);
    });
}

fn sentence(kind: Kind) -> &'static str {
    match kind {
        Kind::Hdr => "One view at several exposures, to be merged later",
        Kind::FocusStack => "One view focused at several distances, to be merged later",
        Kind::Timelapse => "Frames at a fixed interval, from a camera on a timer",
        Kind::Series => "A burst: several frames of the same moment",
    }
}

fn badges_label(badges: Badges) -> &'static str {
    match badges {
        Badges::None => "Nothing",
        Badges::Marks => "Marks",
        Badges::Full => "Marks and name",
    }
}

fn badges_sentence(badges: Badges) -> &'static str {
    match badges {
        Badges::None => "The picture alone",
        Badges::Marks => "Stars, flag and colour label",
        Badges::Full => "Those, and the file name",
    }
}

fn overlay_sentence(overlay: Overlay) -> &'static str {
    match overlay {
        Overlay::Off => "The photograph as it is",
        Overlay::Clipping => {
            "Marks what has gone to pure white or pure black, in red and blue. \
             What is marked has nothing left in it: no development recovers a highlight \
             that was never recorded."
        }
        Overlay::Peaking => {
            "Marks where the edges are sharpest, in green. It says where the focus fell, \
             not whether the frame is sharp enough — that is what magnifying is for."
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every glyph the viewer draws is in the legend, and says something.
    #[test]
    fn every_stack_glyph_is_explained() {
        for kind in Kind::ALL {
            assert!(!stacks::glyph(*kind).is_empty());
            assert!(!sentence(*kind).is_empty());
        }
    }

    #[test]
    fn every_overlay_is_explained() {
        for overlay in Overlay::ALL {
            assert!(!overlay.label().is_empty());
            assert!(!overlay_sentence(*overlay).is_empty());
        }
    }

    #[test]
    fn every_badge_state_is_explained() {
        for badges in Badges::ALL {
            assert!(!badges_label(*badges).is_empty());
            assert!(!badges_sentence(*badges).is_empty());
        }
    }
}
