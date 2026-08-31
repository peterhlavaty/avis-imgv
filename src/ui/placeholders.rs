//! The vocabulary a name template is written in.
//!
//! The same grid is drawn twice inside the bulk rename panel and reachable from
//! none of the three configuration fields that use the same grammar. Here it is
//! once, off the Help menu, where somebody writing an overlay format can find
//! it.

use eframe::egui;

use crate::metadata::template::PLACEHOLDERS;

/// Draws the window.
pub fn ui(ctx: &egui::Context, open: &mut bool) {
    egui::Window::new("Template placeholders")
        .open(open)
        .default_width(560.0)
        .default_height(480.0)
        .show(ctx, |ui| {
            ui.label(
                "These may be used in a new file name, in the line drawn over a photograph, \
                 and in the caption under a cell.",
            );
            ui.add_space(6.0);
            ui.weak("Anything the photograph cannot answer expands to nothing.");
            ui.add_space(8.0);

            egui::ScrollArea::vertical().show(ui, |ui| {
                egui::Grid::new("placeholders")
                    .num_columns(2)
                    .spacing([16.0, 4.0])
                    .striped(true)
                    .show(ui, |ui| {
                        for (token, meaning) in PLACEHOLDERS {
                            // Clickable, because the reason somebody opened
                            // this window is to put one of these somewhere
                            // else.
                            if ui
                                .add(
                                    egui::Label::new(egui::RichText::new(*token).monospace())
                                        .sense(egui::Sense::click()),
                                )
                                .on_hover_text("Click to copy")
                                .clicked()
                            {
                                ui.ctx().copy_text((*token).to_string());
                            }

                            ui.label(*meaning);
                            ui.end_row();
                        }
                    });
            });
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_placeholder_says_what_it_is() {
        assert!(!PLACEHOLDERS.is_empty());

        for (token, meaning) in PLACEHOLDERS {
            assert!(!token.is_empty());
            assert!(!meaning.is_empty());
        }
    }
}
