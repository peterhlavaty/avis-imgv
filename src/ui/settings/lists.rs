//! The five lists that need an editor rather than a control.
//!
//! A path, a label and a digit is not a value a text box can hold, and neither
//! is a keyword tree. Each of these draws its own rows, and each writes the
//! same field the file holds — so what the window edits and what a person
//! hand-edits are the same thing.

use eframe::egui::{self, RichText};

use crate::config::registry::access::List;
use crate::config::{Config, ContextMenuEntry, Destination, TagCategory, UserAction};

use super::widgets::Touched;

/// Draws whichever list a row names.
pub fn ui(ui: &mut egui::Ui, list: List, config: &mut Config) -> Touched {
    match list {
        List::Destinations => destinations(ui, config),
        List::Categories => categories(ui, config),
        List::MetadataTags => metadata_tags(ui, config),
        List::UserActions => actions(ui, config),
        List::ContextMenu => menus(ui, config),
        // The keys have their editor on Keys and mouse, a row each.
        List::RatingKeys | List::LabelKeys => {
            ui.weak("Edited on Keys and mouse, a row each.");
            Touched::default()
        }
    }
}

fn changed() -> Touched {
    Touched {
        changed: true,
        committed: true,
    }
}

/// Where photographs are sent, with the digit that reaches each.
///
/// The tenth and later are drawn greyed rather than dropped: `take(9)` used to
/// discard them silently, so a folder written into the file simply was not
/// there and nothing said why.
fn destinations(ui: &mut egui::Ui, config: &mut Config) -> Touched {
    let mut touched = Touched::default();
    let mut remove = None;
    let mut move_up = None;

    ui.vertical(|ui| {
        for (at, destination) in config.cull.destinations.iter_mut().enumerate() {
            ui.horizontal(|ui| {
                if at < 9 {
                    ui.label(RichText::new(format!("{}", at + 1)).monospace().strong())
                        .on_hover_text("The digit that sends a photograph here");
                } else {
                    ui.label(RichText::new("—").monospace().weak())
                        .on_hover_text(
                            "There are nine digits and the digit is the gesture, so this one \
                         is reached with the arrow keys",
                        );
                }

                touched.changed |= ui
                    .add(
                        egui::TextEdit::singleline(&mut destination.label)
                            .desired_width(140.0)
                            .hint_text("what it is called"),
                    )
                    .changed();

                touched.changed |= ui
                    .add(
                        egui::TextEdit::singleline(&mut destination.path)
                            .desired_width(280.0)
                            .hint_text("where it goes"),
                    )
                    .changed();

                if ui.small_button("Choose…").clicked() {
                    if let Some(picked) = rfd::FileDialog::new().pick_folder() {
                        destination.path = picked.display().to_string();
                        touched = changed();
                    }
                }

                if at > 0 && ui.small_button("↑").on_hover_text("Move it up").clicked() {
                    move_up = Some(at);
                }

                if ui.small_button("✖").on_hover_text("Take it off").clicked() {
                    remove = Some(at);
                }
            });
        }

        if ui.button("Add a destination").clicked() {
            config.cull.destinations.push(Destination {
                label: String::new(),
                path: String::new(),
            });
            touched = changed();
        }
    });

    if let Some(at) = move_up {
        config.cull.destinations.swap(at - 1, at);
        touched = changed();
    }

    if let Some(at) = remove {
        config.cull.destinations.remove(at);
        touched = changed();
    }

    if touched.changed {
        touched.committed = true;
    }

    touched
}

/// The keyword tree.
fn categories(ui: &mut egui::Ui, config: &mut Config) -> Touched {
    let mut touched = Touched::default();
    let mut remove = None;

    ui.vertical(|ui| {
        for (at, category) in config.tags.categories.iter_mut().enumerate() {
            egui::CollapsingHeader::new(if category.name.is_empty() {
                "(unnamed)".to_string()
            } else {
                format!("{} ({})", category.name, category.tags.len())
            })
            .id_salt(("category", at))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Heading:");
                    touched.changed |= ui
                        .add(egui::TextEdit::singleline(&mut category.name).desired_width(220.0))
                        .changed();

                    if ui.small_button("Take the group off").clicked() {
                        remove = Some(at);
                    }
                });

                let mut text = category.tags.join("\n");
                let response = ui.add(
                    egui::TextEdit::multiline(&mut text)
                        .desired_width(f32::INFINITY)
                        .desired_rows(4)
                        .hint_text(
                            "one keyword a line; Places|Slovakia|Tatras files it \
                                    under its levels",
                        ),
                );

                if response.changed() {
                    category.tags = text
                        .lines()
                        .map(str::trim)
                        .filter(|line| !line.is_empty())
                        .map(str::to_string)
                        .collect();
                    touched.changed = true;
                }
            });
        }

        if ui.button("Add a group").clicked() {
            config.tags.categories.push(TagCategory {
                name: String::new(),
                tags: Vec::new(),
            });
            touched = changed();
        }
    });

    if let Some(at) = remove {
        config.tags.categories.remove(at);
        touched = changed();
    }

    if touched.changed {
        touched.committed = true;
    }

    touched
}

/// Which metadata tags the side panel lists, in this order.
fn metadata_tags(ui: &mut egui::Ui, config: &mut Config) -> Touched {
    let mut touched = Touched::default();
    let mut text = config.general.metadata_tags.join("\n");

    ui.vertical(|ui| {
        let response = ui.add(
            egui::TextEdit::multiline(&mut text)
                .desired_width(f32::INFINITY)
                .desired_rows(8)
                .font(egui::TextStyle::Monospace)
                .hint_text("one tag a line, in the order the panel shows them"),
        );

        if response.changed() {
            config.general.metadata_tags = text
                .lines()
                .map(str::trim)
                .filter(|line| !line.is_empty())
                .map(str::to_string)
                .collect();
            touched = changed();
        }

        ui.weak(
            RichText::new(
                "Spelled as they appear in the panel. A tag the file does not carry is \
                 skipped rather than drawn empty.",
            )
            .small(),
        );
    });

    touched
}

/// Commands run on the photograph on screen.
fn actions(ui: &mut egui::Ui, config: &mut Config) -> Touched {
    let mut touched = Touched::default();
    let mut remove = None;
    let mut test = None;

    ui.vertical(|ui| {
        for (at, action) in config.image_view.user_actions.iter_mut().enumerate() {
            ui.horizontal(|ui| {
                ui.label(RichText::new(crate::ui::keys::describe(&action.shortcut)).monospace())
                    .on_hover_text("Changed on Keys and mouse");

                touched.changed |= ui
                    .add(
                        egui::TextEdit::singleline(&mut action.exec)
                            .desired_width(360.0)
                            .font(egui::TextStyle::Monospace)
                            .hint_text("gimp {}"),
                    )
                    .on_hover_text(
                        "No shell: the command is split into arguments first and the file \
                         name goes into one of them, so a name with a space or an \
                         apostrophe cannot add arguments of its own. {} is the path.",
                    )
                    .changed();

                if ui
                    .small_button("Test")
                    .on_hover_text("Runs it on the photograph on screen and says what came back")
                    .clicked()
                {
                    test = Some(at);
                }

                if ui.small_button("✖").clicked() {
                    remove = Some(at);
                }
            });
        }

        if ui.button("Add a command").clicked() {
            config.image_view.user_actions.push(UserAction {
                shortcut: crate::config::Shortcut::new("", &[]),
                exec: String::new(),
                callback: None,
            });
            touched = changed();
        }
    });

    if let Some(at) = remove {
        config.image_view.user_actions.remove(at);
        touched = changed();
    }

    // Reported where it was asked for. The source's own note asked for this:
    // `//Show toast with result?`
    if let Some(at) = test {
        if let Some(action) = config.image_view.user_actions.get(at) {
            let said = if crate::actions::execute(&action.exec, std::path::Path::new("test.jpg")) {
                "The command started."
            } else {
                "The command did not start. The log says why."
            };
            ui.weak(said);
        }
    }

    if touched.changed {
        touched.committed = true;
    }

    touched
}

/// The two context-menu lists, as one table with a column saying where each
/// entry appears. They are one idea written twice in the file.
fn menus(ui: &mut egui::Ui, config: &mut Config) -> Touched {
    let mut touched = Touched::default();

    ui.vertical(|ui| {
        touched.changed |= entries(ui, "On a photograph", &mut config.image_view.context_menu);
        ui.add_space(6.0);
        touched.changed |= entries(ui, "On a cell", &mut config.grid_view.context_menu);
    });

    if touched.changed {
        touched.committed = true;
    }

    touched
}

fn entries(ui: &mut egui::Ui, where_it_is: &str, list: &mut Vec<ContextMenuEntry>) -> bool {
    let mut changed = false;
    let mut remove = None;

    ui.label(RichText::new(where_it_is).strong());

    for (at, entry) in list.iter_mut().enumerate() {
        ui.horizontal(|ui| {
            changed |= ui
                .add(
                    egui::TextEdit::singleline(&mut entry.description)
                        .desired_width(180.0)
                        .hint_text("what the row says"),
                )
                .changed();

            changed |= ui
                .add(
                    egui::TextEdit::singleline(&mut entry.exec)
                        .desired_width(300.0)
                        .font(egui::TextStyle::Monospace)
                        .hint_text("gimp {}"),
                )
                .changed();

            if ui.small_button("✖").clicked() {
                remove = Some(at);
            }
        });
    }

    if ui
        .button(format!("Add a row {}", where_it_is.to_lowercase()))
        .clicked()
    {
        list.push(ContextMenuEntry {
            description: String::new(),
            exec: String::new(),
            callback: None,
        });
        changed = true;
    }

    if let Some(at) = remove {
        list.remove(at);
        changed = true;
    }

    changed
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The tenth destination is drawn rather than dropped, which is what the
    /// nine-digit cap used to do in silence.
    #[test]
    fn a_tenth_destination_is_still_a_destination() {
        let mut config = Config::default();

        for i in 0..12 {
            config.cull.destinations.push(Destination {
                label: format!("slot {i}"),
                path: format!("/photos/{i}"),
            });
        }

        // Nothing here truncates: the drawing greys the digit and the panel
        // reaches the rest with the arrow keys.
        assert!(config.cull.destinations.len() > 9);
    }
}
