//! The bar that narrows and orders the open folder.
//!
//! Deliberately one row rather than a panel: it sits above the photographs
//! instead of replacing them, because the whole point is that "show me the
//! three stars and better" should not mean leaving the picture behind.

use eframe::egui::{self, RichText};

use crate::metadata::xmp::{Label, MAX_RATING};
use crate::organize::group::Settings;
use crate::view::narrow::{FlagRule, LabelRule, Narrowing, Rules, SortBy};

/// Draws the bar, returning whether anything about it changed.
pub fn ui(
    ctx: &egui::Context,
    visible: bool,
    narrowing: &mut Narrowing,
    shown: (usize, usize),
    stacking: &mut StackState<'_>,
    columns: &mut usize,
) -> (bool, StackOutcome) {
    let mut changed = false;
    let mut stacked = StackOutcome::default();

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
                ui.separator();

                stacked = stacks(ui, stacking);
                ui.separator();

                stacked.columns = size(ui, columns);
                ui.separator();

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

    (changed, stacked)
}

/// What the sheet is stacked into, and how, as the bar shows it.
pub struct StackState<'a> {
    pub on: bool,
    /// How the detector reads the folder. Dragged live, so a photographer can
    /// see a burst split and rejoin rather than guessing at a number.
    pub settings: &'a mut Settings,
    /// How many runs were found, and how many frames are in one.
    pub found: usize,
    pub stacked: usize,
    pub all_collapsed: bool,
    /// How far the reading has got, while it is still going.
    pub reading: Option<(usize, usize)>,
}

/// What the stacking half of the bar reported.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct StackOutcome {
    /// Stacking was turned on or off.
    pub toggled: bool,
    /// The detector was asked to read the folder differently.
    pub retuned: bool,
    /// Every stack was closed, or every one opened.
    pub set_all: Option<bool>,
    /// How many thumbnails across, if the rail moved.
    pub columns: Option<usize>,
    /// A settings row a menu asked for.
    pub settings: Option<&'static str>,
}

/// How many thumbnails fit across the sheet.
///
/// On the bar rather than only in the settings window, which is where Lightroom
/// keeps it: on the Grid toolbar, not in preferences. It writes the
/// configuration field through the same setter, so the value survives the
/// session — which is the thing none of this program's in-view controls used to
/// do.
fn size(ui: &mut egui::Ui, columns: &mut usize) -> Option<usize> {
    ui.label("Across").on_hover_text(
        "How many thumbnails fit in a row, which is what decides how large they are",
    );

    let mut wanted = *columns;
    let moved = ui
        .add(
            egui::Slider::new(&mut wanted, 1..=16)
                .clamping(egui::SliderClamping::Edits)
                .show_value(false),
        )
        .changed();

    if moved {
        *columns = wanted;
        return Some(wanted);
    }

    None
}

/// The stacking controls: on or off, how many runs, and how strictly to read
/// them.
fn stacks(ui: &mut egui::Ui, state: &mut StackState<'_>) -> StackOutcome {
    let toggled = ui
        .selectable_label(state.on, "Stacks")
        .on_hover_text(
            "Show every burst, bracket and timelapse as one cell. Nothing is written to disk.",
        )
        .clicked();

    let mut outcome = StackOutcome {
        toggled,
        ..StackOutcome::default()
    };

    if !state.on {
        return outcome;
    }

    if let Some((done, total)) = state.reading {
        ui.label(RichText::new(format!("reading {done}/{total}")).weak());
        return outcome;
    }

    ui.label(RichText::new(format!("{} stacks · {} frames", state.found, state.stacked)).weak())
        .on_hover_text("How many runs of frames the folder holds, and how many frames are in them");

    let (label, wanted) = if state.all_collapsed {
        ("Open all", false)
    } else {
        ("Fold all", true)
    };

    if ui.button(label).clicked() {
        outcome.set_all = Some(wanted);
    }

    ui.label("Gap");
    let mut seconds = state.settings.max_gap;
    if ui
        .add(
            egui::DragValue::new(&mut seconds)
                .range(1.0..=600.0)
                .clamp_existing_to_range(false)
                .suffix(" s"),
        )
        .on_hover_text("The longest pause between two frames that is still one run")
        .changed()
    {
        state.settings.max_gap = seconds;
        outcome.retuned = true;
    }

    ui.label("Alike");
    let mut tolerance = state.settings.tolerance;
    if ui
        .add(egui::Slider::new(&mut tolerance, 0..=32)
                .clamping(egui::SliderClamping::Edits)
                .show_value(false))
        .on_hover_text(
            "How different two frames may look and still belong together. Drag it and watch the runs join up or come apart.",
        )
        .changed()
    {
        state.settings.tolerance = tolerance;
        outcome.retuned = true;
    }

    outcome
}

fn stars(ui: &mut egui::Ui, rules: &mut Rules) -> bool {
    let mut changed = false;

    ui.label("Stars");
    changed |= ui
        .add(
            egui::DragValue::new(&mut rules.min_stars)
                .range(0..=MAX_RATING as u8)
                .clamp_existing_to_range(false),
        )
        .on_hover_text("Fewest stars to show")
        .changed();

    ui.label("to");
    changed |= ui
        .add(
            egui::DragValue::new(&mut rules.max_stars)
                .range(0..=MAX_RATING as u8)
                .clamp_existing_to_range(false),
        )
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
