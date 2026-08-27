//! Choosing which files a folder job applies to, and in what order.
//!
//! Shared by both modes because both need it, and because a user who has set
//! up a selection to rename should not have to set it up again to correct the
//! clock on the same files.

use eframe::egui;

use crate::organize::SortKey;

use super::OrganizeView;

/// Width of the small numeric and text boxes, so the rows line up.
const NARROW: f32 = 90.0;
const WIDE: f32 = 180.0;

pub fn show(ui: &mut egui::Ui, view: &mut OrganizeView) {
    let before = (
        view.sort_key.clone(),
        view.sort_tag.clone(),
        view.direction,
        view.filter.clone(),
    );

    sorting(ui, view);
    filtering(ui, view);

    let after = (
        view.sort_key.clone(),
        view.sort_tag.clone(),
        view.direction,
        view.filter.clone(),
    );

    if before != after {
        view.stale = true;
    }
}

fn sorting(ui: &mut egui::Ui, view: &mut OrganizeView) {
    ui.horizontal(|ui| {
        ui.label("Sort by:");

        egui::ComboBox::from_id_salt("organize sort key")
            .selected_text(sort_label(&view.sort_key))
            .show_ui(ui, |ui| {
                for key in SortKey::CHOICES {
                    ui.selectable_value(&mut view.sort_key, key.clone(), key.label());
                }

                ui.selectable_value(
                    &mut view.sort_key,
                    SortKey::Metadata(String::new()),
                    "Other metadata…",
                );
            });

        if matches!(view.sort_key, SortKey::Metadata(_)) {
            ui.add(
                egui::TextEdit::singleline(&mut view.sort_tag)
                    .desired_width(WIDE)
                    .hint_text("tag, such as ISO"),
            );
        }

        if ui.button(view.direction.label()).clicked() {
            view.direction = view.direction.flipped();
        }

        let (selected, total) = view.counts();
        ui.label(egui::RichText::new(format!("· {selected} of {total} files")).weak());
    });
}

fn filtering(ui: &mut egui::Ui, view: &mut OrganizeView) {
    ui.horizontal(|ui| {
        let label = if view.filter.is_empty() {
            "Filter".to_string()
        } else {
            "Filter (on)".to_string()
        };

        ui.toggle_value(&mut view.filter_open, label);

        if !view.filter.is_empty() && ui.button("Clear").clicked() {
            view.filter = crate::organize::Filter::new();
        }
    });

    if !view.filter_open {
        return;
    }

    let filter = &mut view.filter;

    ui.indent("organize filter", |ui| {
        ui.horizontal(|ui| {
            ui.label("Name contains:");
            ui.add(egui::TextEdit::singleline(&mut filter.name_contains).desired_width(WIDE));

            ui.label("Type:");
            ui.add(
                egui::TextEdit::singleline(&mut filter.extensions)
                    .desired_width(WIDE)
                    .hint_text("jpg, cr3"),
            );
        });

        ui.horizontal(|ui| {
            ui.label("Size between:");
            optional_number(ui, &mut filter.min_size, "any");
            ui.label("and");
            optional_number(ui, &mut filter.max_size, "any");
            ui.label("bytes");
        });

        ui.horizontal(|ui| {
            ui.label("Metadata:");
            ui.add(
                egui::TextEdit::singleline(&mut filter.metadata_tag)
                    .desired_width(WIDE)
                    .hint_text("tag, such as Camera Model Name"),
            );
            ui.label("contains");
            ui.add(
                egui::TextEdit::singleline(&mut filter.metadata_contains)
                    .desired_width(WIDE)
                    .hint_text("anything"),
            );
        });

        ui.horizontal(|ui| {
            ui.label("Stars between:");
            ui.add(egui::DragValue::new(&mut filter.min_rating).range(0..=5));
            ui.label("and");
            ui.add(egui::DragValue::new(&mut filter.max_rating).range(0..=5));

            ui.label("Tagged:");
            ui.add(
                egui::TextEdit::singleline(&mut filter.with_any_tag)
                    .desired_width(WIDE)
                    .hint_text("any of these"),
            );

            ui.label("but not:");
            ui.add(
                egui::TextEdit::singleline(&mut filter.without_tags)
                    .desired_width(WIDE)
                    .hint_text("none of these"),
            );
        });
    });
}

/// A number box that can also be empty, meaning "no bound".
fn optional_number(ui: &mut egui::Ui, value: &mut Option<u64>, hint: &str) {
    let mut text = value.map(|number| number.to_string()).unwrap_or_default();

    let response = ui.add(
        egui::TextEdit::singleline(&mut text)
            .desired_width(NARROW)
            .hint_text(hint),
    );

    if response.changed() {
        *value = text.trim().parse().ok();
    }
}

/// What the dropdown shows, which for a typed tag is the tag itself.
fn sort_label(key: &SortKey) -> String {
    match key {
        SortKey::Metadata(_) => "Other metadata…".to_string(),
        other => other.label().to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_dropdown_names_a_typed_tag_by_its_entry_rather_than_its_value() {
        assert_eq!(sort_label(&SortKey::Name), "Name");
        assert_eq!(
            sort_label(&SortKey::Metadata("ISO".into())),
            "Other metadata…"
        );
    }
}
