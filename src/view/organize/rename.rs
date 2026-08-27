//! The bulk rename panel.

use eframe::egui;

use crate::organize::rename::{self, Extension, Planned, PLACEHOLDERS};

use super::table::{self, Row};
use super::{Done, OrganizeView};

const TEMPLATE_WIDTH: f32 = 320.0;

pub fn show(ui: &mut egui::Ui, view: &mut OrganizeView) -> Option<Done> {
    template(ui, &mut view.rename);
    counter(ui, &mut view.rename);

    let planned = rename::plan(&view.selection, &view.rename);
    let done = actions(ui, view, &planned);

    ui.add_space(6.0);
    table::show(ui, ("Now", "Would become"), &rows(&planned));

    done
}

fn template(ui: &mut egui::Ui, options: &mut rename::Options) {
    ui.horizontal(|ui| {
        ui.label("Name:");
        ui.add(
            egui::TextEdit::singleline(&mut options.template)
                .desired_width(TEMPLATE_WIDTH)
                .hint_text("{date}_{counter}"),
        )
        .on_hover_ui(placeholders);

        ui.menu_button("Insert…", |ui| {
            for (placeholder, meaning) in PLACEHOLDERS {
                // The literal brace entry is a note rather than something to
                // insert; inserting it would be inserting two braces.
                if placeholder.starts_with("{{") {
                    continue;
                }

                if ui.button(format!("{placeholder}  —  {meaning}")).clicked() {
                    options.template.push_str(first_form(placeholder));
                    ui.close();
                }
            }
        });
    });
}

fn counter(ui: &mut egui::Ui, options: &mut rename::Options) {
    ui.horizontal(|ui| {
        ui.label("Counter starts at:");
        ui.add(egui::DragValue::new(&mut options.counter_start).range(0..=1_000_000));

        ui.label("steps by:");
        ui.add(egui::DragValue::new(&mut options.counter_step).range(1..=1000));

        ui.label("digits:");
        ui.add(egui::DragValue::new(&mut options.counter_digits).range(1..=12));

        ui.label("Extension:");
        egui::ComboBox::from_id_salt("rename extension")
            .selected_text(options.extension.label())
            .show_ui(ui, |ui| {
                for choice in Extension::CHOICES {
                    ui.selectable_value(&mut options.extension, *choice, choice.label());
                }
            });
    });
}

fn actions(ui: &mut egui::Ui, view: &mut OrganizeView, planned: &[Planned]) -> Option<Done> {
    let changing = planned.iter().filter(|plan| plan.changes()).count();
    let problems = planned.iter().filter(|plan| plan.problem.is_some()).count();

    let mut done = None;

    ui.add_space(6.0);
    ui.horizontal(|ui| {
        let button = egui::Button::new(format!("Rename {changing} file(s)"));

        if ui.add_enabled(changing > 0, button).clicked() {
            let outcome = rename::apply(planned);

            view.status = outcome.summary();
            for (path, problem) in &outcome.failed {
                tracing::warn!("Could not rename {}: {problem}", path.display());
            }

            done = Some(Done::Renamed);
        }

        if problems > 0 {
            ui.label(
                egui::RichText::new(format!("{problems} cannot be renamed"))
                    .color(egui::Color32::from_rgb(220, 120, 120)),
            );
        }

        if !view.status.is_empty() {
            ui.weak(&view.status);
        }
    });

    done
}

fn rows(planned: &[Planned]) -> Vec<Row> {
    planned
        .iter()
        .map(|plan| Row {
            before: plan
                .from
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned(),
            after: plan.new_name(),
            problem: plan.problem.map(|problem| problem.message().to_string()),
            changes: plan.changes(),
        })
        .collect()
}

/// A placeholder entry can list several forms; inserting takes the first.
fn first_form(placeholder: &str) -> &str {
    placeholder.split_whitespace().next().unwrap_or(placeholder)
}

fn placeholders(ui: &mut egui::Ui) {
    ui.label("Anything outside braces is written as it is.");
    ui.add_space(4.0);

    egui::Grid::new("rename placeholders")
        .num_columns(2)
        .spacing([12.0, 2.0])
        .show(ui, |ui| {
            for (placeholder, meaning) in PLACEHOLDERS {
                ui.monospace(*placeholder);
                ui.label(*meaning);
                ui.end_row();
            }
        });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::organize::Entry;
    use std::path::PathBuf;

    fn entry(name: &str) -> Entry {
        Entry::new(PathBuf::from("/photos").join(name))
    }

    #[test]
    fn a_row_shows_the_name_before_and_after() {
        let planned = rename::plan(
            &[entry("a.jpg")],
            &rename::Options {
                template: "b".into(),
                ..Default::default()
            },
        );

        let rows = rows(&planned);
        assert_eq!(rows[0].before, "a.jpg");
        assert_eq!(rows[0].after, "b.jpg");
        assert!(rows[0].changes);
        assert!(rows[0].problem.is_none());
    }

    #[test]
    fn a_row_that_cannot_happen_says_why() {
        let planned = rename::plan(
            &[entry("a.jpg"), entry("b.jpg")],
            &rename::Options {
                template: "same".into(),
                ..Default::default()
            },
        );

        let rows = rows(&planned);
        assert!(rows.iter().all(|row| row.problem.is_some()));
        assert!(rows.iter().all(|row| !row.changes));
    }

    #[test]
    fn inserting_a_placeholder_takes_only_its_first_form() {
        assert_eq!(first_form("{year} {month} {day}"), "{year}");
        assert_eq!(first_form("{counter}"), "{counter}");
    }
}
