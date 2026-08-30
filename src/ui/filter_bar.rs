//! The bar that narrows and orders the open folder.
//!
//! Deliberately one row rather than a panel: it sits above the photographs
//! instead of replacing them, because the whole point is that "show me the
//! three stars and better" should not mean leaving the picture behind.

use eframe::egui::{self, RichText};

use crate::metadata::xmp::{Label, MAX_RATING};
use crate::view::narrow::{FlagRule, LabelRule, Narrowing, Rules, SortBy};

/// Draws the bar, returning whether anything about it changed.
pub fn ui(
    ctx: &egui::Context,
    visible: bool,
    narrowing: &mut Narrowing,
    shown: (usize, usize),
) -> bool {
    let mut changed = false;

    egui::TopBottomPanel::top("filter_bar")
        .show_separator_line(false)
        .show_animated(ctx, visible, |ui| {
            ui.add_space(2.0);

            ui.horizontal_wrapped(|ui| {
                changed |= stars(ui, &mut narrowing.rules);
                ui.separator();

                changed |= flag(ui, &mut narrowing.rules);
                changed |= label(ui, &mut narrowing.rules);
                ui.separator();

                changed |= text_rules(ui, &mut narrowing.rules);
                ui.separator();

                changed |= order(ui, narrowing);
                ui.separator();

                changed |= suspend(ui, narrowing);

                if ui
                    .button("Clear")
                    .on_hover_text("Put every rule back to anything")
                    .clicked()
                {
                    narrowing.rules = Rules::default();
                    narrowing.suspended = false;
                    changed = true;
                }

                counted(ui, narrowing, shown);
            });

            ui.add_space(2.0);
        });

    changed
}

fn stars(ui: &mut egui::Ui, rules: &mut Rules) -> bool {
    let mut changed = false;

    ui.label("Stars");
    changed |= ui
        .add(egui::DragValue::new(&mut rules.min_stars).range(0..=MAX_RATING as u8))
        .on_hover_text("Fewest stars to show")
        .changed();

    ui.label("to");
    changed |= ui
        .add(egui::DragValue::new(&mut rules.max_stars).range(0..=MAX_RATING as u8))
        .on_hover_text("Most stars to show")
        .changed();

    changed
}

fn flag(ui: &mut egui::Ui, rules: &mut Rules) -> bool {
    let mut changed = false;

    egui::ComboBox::from_id_salt("filter_flag")
        .selected_text(rules.flag.label())
        .show_ui(ui, |ui| {
            for wanted in FlagRule::ALL {
                changed |= ui
                    .selectable_value(&mut rules.flag, *wanted, wanted.label())
                    .changed();
            }
        });

    changed
}

fn label(ui: &mut egui::Ui, rules: &mut Rules) -> bool {
    let mut changed = false;

    let selected = match rules.label {
        LabelRule::Any => "Any label".to_string(),
        LabelRule::None => "No label".to_string(),
        LabelRule::One(index) => Label::CHOICES
            .get(index)
            .map(|label| label.name().to_string())
            .unwrap_or_else(|| "Any label".to_string()),
    };

    egui::ComboBox::from_id_salt("filter_label")
        .selected_text(selected)
        .show_ui(ui, |ui| {
            changed |= ui
                .selectable_value(&mut rules.label, LabelRule::Any, "Any label")
                .changed();
            changed |= ui
                .selectable_value(&mut rules.label, LabelRule::None, "No label")
                .changed();

            for (index, known) in Label::CHOICES.iter().enumerate() {
                let (r, g, b) = known.colour();
                let text = RichText::new(known.name()).color(egui::Color32::from_rgb(r, g, b));

                changed |= ui
                    .selectable_value(&mut rules.label, LabelRule::One(index), text)
                    .changed();
            }
        });

    changed
}

fn text_rules(ui: &mut egui::Ui, rules: &mut Rules) -> bool {
    let mut changed = false;

    for (hint, field, width) in [
        ("Name contains", &mut rules.name_contains, 110.0),
        ("Keyword", &mut rules.keyword, 100.0),
        ("Types", &mut rules.extensions, 80.0),
    ] {
        changed |= ui
            .add(
                egui::TextEdit::singleline(field)
                    .hint_text(hint)
                    .desired_width(width),
            )
            .changed();
    }

    changed
}

fn order(ui: &mut egui::Ui, narrowing: &mut Narrowing) -> bool {
    let mut changed = false;

    ui.label("Order by");
    egui::ComboBox::from_id_salt("filter_sort")
        .selected_text(narrowing.sort.label())
        .show_ui(ui, |ui| {
            for wanted in SortBy::ALL {
                changed |= ui
                    .selectable_value(&mut narrowing.sort, *wanted, wanted.label())
                    .changed();
            }
        });

    let arrow = if narrowing.descending { "▼" } else { "▲" };
    if ui
        .button(arrow)
        .on_hover_text(if narrowing.descending {
            "Descending"
        } else {
            "Ascending"
        })
        .clicked()
    {
        narrowing.descending = !narrowing.descending;
        changed = true;
    }

    changed
}

fn suspend(ui: &mut egui::Ui, narrowing: &mut Narrowing) -> bool {
    let response = ui
        .selectable_label(narrowing.suspended, "Show everything")
        .on_hover_text("Set the rules aside without forgetting them");

    if response.clicked() {
        narrowing.suspended = !narrowing.suspended;
        return true;
    }

    false
}

/// How much of the folder is left, which is what tells somebody their rules
/// are doing something.
fn counted(ui: &mut egui::Ui, narrowing: &Narrowing, (shown, total): (usize, usize)) {
    let said = if narrowing.hides_anything() {
        format!("{shown} of {total}")
    } else {
        format!("{total}")
    };

    ui.with_layout(
        egui::Layout::right_to_left(eframe::emath::Align::Center),
        |ui| ui.label(RichText::new(said).weak()),
    );
}
