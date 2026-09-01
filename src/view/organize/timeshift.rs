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

        ui.add(
            egui::DragValue::new(&mut offset.days)
                .range(0..=3650)
                .clamp_existing_to_range(false),
        );
        ui.label("days");
        ui.add(
            egui::DragValue::new(&mut offset.hours)
                .range(0..=23)
                .clamp_existing_to_range(false),
        );
        ui.label("hours");
        ui.add(
            egui::DragValue::new(&mut offset.minutes)
                .range(0..=59)
                .clamp_existing_to_range(false),
        );
        ui.label("minutes");
        ui.add(
            egui::DragValue::new(&mut offset.seconds)
                .range(0..=59)
                .clamp_existing_to_range(false),
        );
        ui.label("seconds");

        let label = if offset.forward { "Forward" } else { "Back" };
        if ui.button(label).clicked() {
            offset.forward = !offset.forward;
        }

        ui.label(egui::RichText::new(format!("Â· {}", offset.describe())).weak());
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
    // What the preview shows, so the button and the rows above it cannot
    // disagree about how many files are about to move.
    let changing = planned
        .iter()
        .filter(|plan| !plan.moving.is_empty())
        .count();
    let ready = changing > 0 && !view.offset.is_zero();

    let mut done = None;

    ui.add_space(6.0);
    ui.horizontal(|ui| {
        let button = egui::Button::new(format!("Change {changing} file(s)"));

        if ui
            .add_enabled(ready, button)
            .on_hover_text(
                "Writes the corrected times into the files. This cannot be undone: the \n                 journal covers moves, copies and marks, not folder jobs.",
            )
            .clicked()
        {
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

            // One line per ticked field the file has, rather than the
            // capture time alone: unticking that field while leaving another
            // ticked used to make every row read an em dash beside a button
            // that said it would change four hundred files.
            let before = if plan.moving.is_empty() {
                match plan.before {
                    Some(at) => format!("{name}  Â·  {at}  Â·  nothing ticked is in this file"),
                    None => format!("{name}  Â·  no capture time"),
                }
            } else {
                let fields: Vec<String> = plan
                    .moving
                    .iter()
                    .map(|(field, was, _)| format!("{field} {was}"))
                    .collect();
                format!("{name}  Â·  {}", fields.join("  Â·  "))
            };

            let after = if plan.moving.is_empty() {
                "—".to_string()
            } else {
                plan.moving
                    .iter()
                    .map(|(_, _, becomes)| becomes.to_exif())
                    .collect::<Vec<_>>()
                    .join("  Â·  ")
            };

            Row {
                before,
                after,
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

    /// The preview and the button used to disagree: unticking the capture time
    /// while leaving another field ticked made every row read an em dash while
    /// the button still offered to change four hundred files.
    #[test]
    fn the_preview_shows_every_ticked_field() {
        use crate::organize::timeshift;

        let mut entry = Entry::new(PathBuf::from("/photos/a.jpg"));
        entry.dates = vec![crate::metadata::dates::DateField {
            name: "Date/Time Original",
            offset: 0,
            value: crate::metadata::datetime::Timestamp::parse("2024:01:01 10:00:00").unwrap(),
        }];

        let mut chosen = BTreeSet::new();
        chosen.insert("Date/Time Original".to_string());

        let offset = timeshift::Offset {
            hours: 1,
            forward: true,
            ..timeshift::Offset::default()
        };
        let planned = timeshift::plan(std::slice::from_ref(&entry), &chosen, offset);

        assert_eq!(planned.len(), 1);
        assert_eq!(planned[0].moving.len(), 1);

        let rows = rows(&planned);
        assert!(
            rows[0].after != "—",
            "the preview said nothing would change"
        );
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
