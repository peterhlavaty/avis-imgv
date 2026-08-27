//! The panel that corrects a camera clock.

use eframe::egui;

use crate::organize::timeshift::{self, Planned};

use super::table::{self, Row};
use super::{Done, OrganizeView};

pub fn show(ui: &mut egui::Ui, view: &mut OrganizeView) -> Option<Done> {
    offset(ui, view);
    fields(ui, view);

    let planned = timeshift::plan(&view.selection, &view.chosen_fields, view.offset);
    let done = actions(ui, view, &planned);

    ui.add_space(6.0);
    table::show(ui, ("Taken", "Would become"), &rows(&planned));

    done
}

fn offset(ui: &mut egui::Ui, view: &mut OrganizeView) {
    let offset = &mut view.offset;

    ui.horizontal(|ui| {
        ui.label("Move by:");

        ui.add(egui::DragValue::new(&mut offset.days).range(0..=3650));
        ui.label("days");
        ui.add(egui::DragValue::new(&mut offset.hours).range(0..=23));
        ui.label("hours");
        ui.add(egui::DragValue::new(&mut offset.minutes).range(0..=59));
        ui.label("minutes");
        ui.add(egui::DragValue::new(&mut offset.seconds).range(0..=59));
        ui.label("seconds");

        let label = if offset.forward { "Forward" } else { "Back" };
        if ui.button(label).clicked() {
            offset.forward = !offset.forward;
        }

        ui.label(egui::RichText::new(format!("· {}", offset.describe())).weak());
    })
    .response
    .on_hover_text(
        "Forward when the photographs were taken later than the camera thought \
         — a camera left on winter time in summer needs an hour forward.",
    );
}

fn fields(ui: &mut egui::Ui, view: &mut OrganizeView) {
    let available = timeshift::available_fields(&view.selection);

    if available.is_empty() {
        ui.horizontal(|ui| {
            ui.label("Timestamps:");
            ui.weak("none of the selected files carry one");
        });
        return;
    }

    ui.horizontal_wrapped(|ui| {
        ui.label("Change:");

        for name in &available {
            // Nothing chosen means everything, which is what a user who has
            // not thought about it wants.
            let mut on = view.chosen_fields.is_empty() || view.chosen_fields.contains(name);

            if ui.checkbox(&mut on, name).changed() {
                toggle(&mut view.chosen_fields, &available, name, on);
            }
        }
    });
}

/// Turns one field on or off, expanding the "everything" shorthand first.
///
/// Unticking the first box has to leave the other boxes ticked, and it cannot
/// do that while the set is still empty.
fn toggle(
    chosen: &mut std::collections::BTreeSet<String>,
    available: &[String],
    name: &str,
    on: bool,
) {
    if chosen.is_empty() {
        chosen.extend(available.iter().cloned());
    }

    if on {
        chosen.insert(name.to_string());
    } else {
        chosen.remove(name);
    }
}

fn actions(ui: &mut egui::Ui, view: &mut OrganizeView, planned: &[Planned]) -> Option<Done> {
    let changing = planned.iter().filter(|plan| plan.changes()).count();
    let ready = changing > 0 && !view.offset.is_zero();

    let mut done = None;

    ui.add_space(6.0);
    ui.horizontal(|ui| {
        let button = egui::Button::new(format!("Change {changing} file(s)"));

        if ui.add_enabled(ready, button).clicked() {
            let outcome = timeshift::apply(planned, &view.chosen_fields, view.offset);

            view.status = outcome.summary();
            for (path, problem) in &outcome.failed {
                tracing::warn!("Could not change {}: {problem}", path.display());
            }

            done = Some(Done::Shifted);
        }

        if view.offset.is_zero() {
            ui.weak("Set an offset first.");
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
        .map(|plan| {
            let name = plan
                .path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned();

            let before = match plan.before {
                Some(at) => format!("{name}  ·  {at}"),
                None => format!("{name}  ·  no capture time"),
            };

            Row {
                before,
                after: plan
                    .after
                    .map(|at| at.to_exif())
                    .unwrap_or_else(|| "—".to_string()),
                problem: None,
                changes: plan.changes(),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::organize::Entry;
    use std::collections::BTreeSet;
    use std::path::PathBuf;

    fn available() -> Vec<String> {
        vec!["Date/Time Original".to_string(), "Modify Date".to_string()]
    }

    #[test]
    fn unticking_one_box_leaves_the_others_ticked() {
        let mut chosen = BTreeSet::new();
        let available = available();

        // Nothing chosen yet means every field, so unticking one has to leave
        // the rest behind rather than leaving nothing.
        toggle(&mut chosen, &available, "Modify Date", false);

        assert_eq!(chosen.len(), 1);
        assert!(chosen.contains("Date/Time Original"));
    }

    #[test]
    fn ticking_a_box_back_on_puts_it_back() {
        let mut chosen = BTreeSet::new();
        let available = available();

        toggle(&mut chosen, &available, "Modify Date", false);
        toggle(&mut chosen, &available, "Modify Date", true);

        assert_eq!(chosen.len(), 2);
    }

    #[test]
    fn a_file_with_no_capture_time_says_so_rather_than_showing_a_blank() {
        let planned = timeshift::plan(
            &[Entry::new(PathBuf::from("/photos/a.jpg"))],
            &BTreeSet::new(),
            timeshift::Offset::default(),
        );

        let rows = rows(&planned);
        assert!(rows[0].before.contains("no capture time"));
        assert_eq!(rows[0].after, "—");
        assert!(!rows[0].changes);
    }
}
