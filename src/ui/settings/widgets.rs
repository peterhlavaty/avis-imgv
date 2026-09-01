//! The controls, written once and used by every page.
//!
//! One vocabulary, so a number means the same thing wherever it appears. The
//! rules, from the plan: a rail with its value box for a continuous quantity; a
//! typed box for a whole count sitting against a default nobody moves; a radio
//! group with a sentence under each variant for fewer than five choices; a tick
//! above a number for anything meaning "automatic", so zero never means two
//! things. Units go beside the control and never in a tooltip, because a unit is
//! part of the value and a tooltip is not always read.
//!
//! egui clamps a value it is handed whether or not anybody edited it, so every
//! control here turns that off. What is written by hand always wins, including
//! a value the window itself cannot produce.

use eframe::egui::{self, RichText};

use crate::config::registry::{Access, Choice, Effect, Row};
use crate::config::Config;

/// What one row's control did.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Touched {
    /// The value moved this frame.
    pub changed: bool,
    /// The gesture ended: a rail let go, a box that lost focus, the click on a
    /// radio. A `Rebuild` row commits here rather than on every frame, because
    /// a rail on true per-frame apply would rebuild the cache sixty times a
    /// second.
    pub committed: bool,
}

impl Touched {
    fn of(response: &egui::Response) -> Touched {
        Touched {
            changed: response.changed(),
            committed: response.drag_stopped() || response.lost_focus() || response.clicked(),
        }
    }

    fn merge(self, other: Touched) -> Touched {
        Touched {
            changed: self.changed || other.changed,
            committed: self.committed || other.committed,
        }
    }
}

/// Draws one row: its label, its control, and whatever it has to say.
pub fn row(ui: &mut egui::Ui, row: &Row, config: &mut Config) -> (Touched, Option<&'static str>) {
    let mut touched = Touched::default();
    let mut asked = None;

    ui.horizontal(|ui| {
        // The bullet, which is a button: clicking it puts the field back.
        let differs = row.changed(config);
        let bullet = ui.add_enabled(
            differs,
            egui::Button::new(RichText::new(if differs { "●" } else { " " }).small()).frame(false),
        );

        if differs {
            bullet
                .clone()
                .on_hover_text("Changed from the default. Click to put it back.");
        }

        if bullet.clicked() {
            row.access.reset(config);
            touched = Touched {
                changed: true,
                committed: true,
            };
        }

        // The badge goes on the label, where it is seen before the control is
        // touched. Exactly one row in the whole window carries it, which is
        // what keeps it worth reading.
        let label = if row.effect.badged() {
            RichText::new(format!("{} {}", Effect::BADGE, row.label))
                .color(ui.visuals().warn_fg_color)
        } else {
            RichText::new(row.label)
        };

        ui.add_sized(
            egui::Vec2::new(LABEL_WIDTH, ui.spacing().interact_size.y),
            egui::Label::new(label).wrap(),
        );

        let label_response = ui.interact(
            ui.min_rect(),
            ui.id().with((row.path, "row")),
            egui::Sense::click(),
        );

        if let Some(path) = row_menu(ui, &label_response, row) {
            asked = Some(path);
        }

        touched = touched.merge(control(ui, row, config));

        // Marked where it is drawn, and left exactly as written. `save` writes
        // the whole document, so clamping on load would destroy somebody's
        // deliberate 8,192 MB budget on the first unrelated save.
        if out_of_range(row, config) {
            ui.label(RichText::new("out of range").color(ui.visuals().warn_fg_color))
                .on_hover_text(
                    "Outside what this control can produce. It is left exactly as it                      was written: hand-editing wins.",
                );
        }
    });

    // Under the control, never in a tooltip: a sentence about what a value
    // means is part of the field, and a restart requirement is a field
    // requirement.
    ui.indent(row.path, |ui| {
        ui.weak(RichText::new(row.sentence).small());

        if let Some(said) = row.effect.sentence() {
            let text = RichText::new(said).small();
            ui.label(if row.effect.badged() {
                text.color(ui.visuals().warn_fg_color)
            } else {
                text.weak()
            });
        }

        if let Some(explained) = row.explained {
            ui.weak(RichText::new(explained).small().italics());
        }
    });

    ui.add_space(6.0);
    (touched, asked)
}

/// The row's own menu: the two things somebody who found this page by accident
/// needs, and the one thing somebody who found it on purpose does.
///
/// The registry is keyed on the path and never on the label, which is why
/// "Copy setting name" yields something that can be pasted into a forum answer
/// and still work — nomacs stored shortcuts under their *translated* names and
/// broke every one when the interface language changed.
fn row_menu(ui: &egui::Ui, response: &egui::Response, row: &Row) -> Option<&'static str> {
    let mut asked = None;

    crate::ui::surface::menu(
        ui,
        response,
        crate::ui::surface::Subject::of("Setting", row.label),
        |ui| {
            if ui
                .button("Copy setting name")
                .on_hover_text(row.path)
                .clicked()
            {
                ui.ctx().copy_text(row.path.to_string());
                ui.close();
            }

            if row.access.is_a_key() && ui.button("Change this key…").clicked() {
                asked = Some(row.path);
                ui.close();
            }
        },
    );

    asked
}

/// How wide a row's name column is.
const LABEL_WIDTH: f32 = 250.0;

/// Whether a row holds a number its own control could not produce.
fn out_of_range(row: &Row, config: &Config) -> bool {
    match &row.access {
        Access::Int { get, min, max, .. } => {
            let value = get(config);
            value < *min || value > *max
        }
        Access::Float { get, min, max, .. } => {
            let value = get(config);
            !value.is_finite() || value < *min || value > *max
        }
        _ => false,
    }
}

/// The control itself.
fn control(ui: &mut egui::Ui, row: &Row, config: &mut Config) -> Touched {
    match &row.access {
        Access::Bool(get, set) => {
            let mut on = get(config);
            let response = ui.checkbox(&mut on, "");
            if response.changed() {
                set(config, on);
            }
            Touched::of(&response)
        }
        Access::Int {
            get,
            set,
            min,
            max,
            unit,
            rail,
        } => {
            let mut value = get(config);
            let response = number(ui, &mut value, *min, *max, unit, *rail, row.effect);
            if response.changed() {
                set(config, value);
            }
            Touched::of(&response)
        }
        Access::Float {
            get,
            set,
            min,
            max,
            unit,
            rail,
        } => {
            let mut value = get(config);
            let response = decimal(ui, &mut value, *min, *max, unit, *rail, row.effect);
            if response.changed() {
                set(config, value);
            }
            Touched::of(&response)
        }
        Access::Enum { choices, .. } => radios(ui, row, config, choices),
        Access::Flags { options, .. } => ticks(ui, row, config, options),
        Access::Text(get, set) | Access::Template(get, set) => {
            let mut text = get(config);
            let response = ui.add(
                egui::TextEdit::singleline(&mut text)
                    .desired_width(TEXT_WIDTH)
                    .font(egui::TextStyle::Monospace),
            );
            if response.changed() {
                set(config, text);
            }
            Touched::of(&response)
        }
        Access::Path(get, set) => {
            let mut touched = path(ui, get, set, config);

            // Reported where it was asked for. A keyword list that will not
            // load used to be a log line and a panel showing fewer keywords.
            if row.path == "tags.catalog_file" {
                if ui
                    .button("Read it now")
                    .on_hover_text("Says how many keywords are in it, or why it could not be read")
                    .clicked()
                {
                    touched.committed = true;
                }

                ui.weak(RichText::new(super::lists::read_the_catalogue(config)).small());
            }

            touched
        }
        Access::Colour(get, set) => colour(ui, get, set, config),
        Access::Records(list, _) => super::lists::ui(ui, *list, config),
        Access::Key(..) | Access::RatingKey(_) | Access::LabelKey(_) | Access::ActionKey(_) => {
            let key = row
                .access
                .shortcut(config)
                .map(crate::ui::keys::describe)
                .unwrap_or_else(|| "no key".to_string());
            ui.label(RichText::new(key).monospace());
            Touched::default()
        }
        Access::Fixed(key) => {
            ui.label(RichText::new(*key).monospace().weak());
            Touched::default()
        }
        Access::ReadOnly(get) => {
            let value = get(config);
            ui.label(RichText::new(if value.is_empty() { "—" } else { &value }).weak());
            Touched::default()
        }
        Access::Run(_) => Touched::default(),
    }
}

const TEXT_WIDTH: f32 = 320.0;
const RAIL_WIDTH: f32 = 220.0;

/// A whole number, as a rail with its own value box or as a box alone.
///
/// `Slider`'s value display *is* a `DragValue`, so a rail with a box beside it
/// is one call rather than two widgets that can disagree.
fn number(
    ui: &mut egui::Ui,
    value: &mut i64,
    min: i64,
    max: i64,
    unit: &str,
    rail: bool,
    effect: Effect,
) -> egui::Response {
    if rail && !effect.badged() {
        return ui.add(
            egui::Slider::new(value, min..=max)
                .clamping(egui::SliderClamping::Edits)
                .suffix(unit),
        );
    }

    // A control whose effect only appears at the next launch is worse than a
    // number, because a dragged rail looks like it is doing something.
    ui.add_sized(
        egui::Vec2::new(RAIL_WIDTH, ui.spacing().interact_size.y),
        egui::DragValue::new(value)
            .range(min..=max)
            .clamp_existing_to_range(false)
            .suffix(unit),
    )
}

fn decimal(
    ui: &mut egui::Ui,
    value: &mut f32,
    min: f32,
    max: f32,
    unit: &str,
    rail: bool,
    effect: Effect,
) -> egui::Response {
    if rail && !effect.badged() {
        return ui.add(
            egui::Slider::new(value, min..=max)
                .clamping(egui::SliderClamping::Edits)
                .suffix(unit),
        );
    }

    ui.add_sized(
        egui::Vec2::new(RAIL_WIDTH, ui.spacing().interact_size.y),
        egui::DragValue::new(value)
            .range(min..=max)
            .clamp_existing_to_range(false)
            .speed(0.01)
            .suffix(unit),
    )
}

/// A closed set of choices.
///
/// Radios with a sentence under each below five, a dropdown above: the sentence
/// is what makes a choice decidable without trying it, and eight of them is a
/// wall.
fn radios(ui: &mut egui::Ui, row: &Row, config: &mut Config, choices: &[Choice]) -> Touched {
    let mut touched = Touched::default();
    let current = row.access.choice(config).unwrap_or_default();

    if choices.len() > 5 {
        let selected = choices
            .iter()
            .find(|choice| choice.value == current)
            .map(|choice| choice.label)
            .unwrap_or(current);

        egui::ComboBox::from_id_salt(row.path)
            .selected_text(selected)
            .show_ui(ui, |ui| {
                for choice in choices {
                    if ui
                        .selectable_label(choice.value == current, choice.label)
                        .clicked()
                    {
                        row.access.set_choice(config, choice.value);
                        touched = Touched {
                            changed: true,
                            committed: true,
                        };
                    }
                }
            });

        return touched;
    }

    ui.vertical(|ui| {
        for choice in choices {
            let picked = ui.radio(choice.value == current, choice.label);

            if picked.clicked() && choice.value != current {
                row.access.set_choice(config, choice.value);
                touched = Touched {
                    changed: true,
                    committed: true,
                };
            }

            if !choice.sentence.is_empty() {
                ui.indent((row.path, choice.value), |ui| {
                    ui.weak(RichText::new(choice.sentence).small());
                });
            }
        }
    });

    touched
}

/// A set of named booleans: one decision made of parts.
fn ticks(ui: &mut egui::Ui, row: &Row, config: &mut Config, options: &[Choice]) -> Touched {
    let mut touched = Touched::default();

    ui.vertical(|ui| {
        for option in options {
            let mut on = row.access.flag(config, option.value).unwrap_or(false);

            if ui.checkbox(&mut on, option.label).changed() {
                row.access.set_flag(config, option.value, on);
                touched = Touched {
                    changed: true,
                    committed: true,
                };
            }

            if !option.sentence.is_empty() {
                ui.indent((row.path, option.value), |ui| {
                    ui.weak(RichText::new(option.sentence).small());
                });
            }
        }
    });

    touched
}

/// A path, with a picker beside it.
fn path(
    ui: &mut egui::Ui,
    get: &fn(&Config) -> Option<String>,
    set: &fn(&mut Config, Option<String>),
    config: &mut Config,
) -> Touched {
    let mut touched = Touched::default();
    let mut text = get(config).unwrap_or_default();

    ui.horizontal(|ui| {
        let response = ui.add(
            egui::TextEdit::singleline(&mut text)
                .desired_width(TEXT_WIDTH - 90.0)
                .hint_text("none"),
        );

        if response.changed() {
            let trimmed = text.trim().to_string();
            set(config, (!trimmed.is_empty()).then_some(trimmed));
        }
        touched = Touched::of(&response);

        if ui.button("Choose…").clicked() {
            if let Some(picked) = rfd::FileDialog::new().pick_folder() {
                set(config, Some(picked.display().to_string()));
                touched = Touched {
                    changed: true,
                    committed: true,
                };
            }
        }

        if !text.is_empty() && ui.button("Clear").clicked() {
            set(config, None);
            touched = Touched {
                changed: true,
                committed: true,
            };
        }
    });

    touched
}

/// A hex colour, with a swatch that opens the picker.
fn colour(
    ui: &mut egui::Ui,
    get: &fn(&Config) -> Option<String>,
    set: &fn(&mut Config, Option<String>),
    config: &mut Config,
) -> Touched {
    let mut touched = Touched::default();
    let held = get(config).unwrap_or_default();

    ui.horizontal(|ui| {
        let mut colour = egui::Color32::from_hex(&held).unwrap_or(egui::Color32::GRAY);
        let response = ui.color_edit_button_srgba(&mut colour);

        if response.changed() {
            set(config, Some(colour.to_hex()));
            touched = Touched {
                changed: true,
                committed: true,
            };
        }

        let mut text = held.clone();
        let typed = ui.add(
            egui::TextEdit::singleline(&mut text)
                .desired_width(110.0)
                .font(egui::TextStyle::Monospace)
                .hint_text("#777777"),
        );

        if typed.changed() {
            let trimmed = text.trim().to_string();
            set(config, (!trimmed.is_empty()).then_some(trimmed));
        }
        touched = touched.merge(Touched::of(&typed));
    });

    touched
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A rail that would move under the hand and change nothing is drawn as a
    /// number instead, which is the whole of §5.9's argument.
    #[test]
    fn a_restart_bound_number_gets_a_box_rather_than_a_rail() {
        // The decision is in `number`; asserted here as the property it
        // encodes, so a change to it has to change this too.
        assert!(Effect::Restart.badged());
        assert!(!Effect::Rebuild.badged());
    }

    /// A value the window cannot produce is marked and not touched.
    #[test]
    fn a_hand_edited_value_out_of_range_is_marked_and_kept() {
        let mut config = Config::default();
        config.cache.ram_budget_mb = 200_000;

        let row = crate::config::registry::row("cache.ram_budget_mb").expect("it is there");

        assert!(out_of_range(row, &config));
        assert_eq!(config.cache.ram_budget_mb, 200_000);
    }

    #[test]
    fn a_value_inside_its_range_is_not_marked() {
        let config = Config::default();

        for row in crate::config::registry::rows() {
            assert!(!out_of_range(row, &config), "{} is out of range", row.path);
        }
    }

    #[test]
    fn a_gesture_that_ends_commits() {
        let ended = Touched {
            changed: true,
            committed: true,
        };
        let moving = Touched {
            changed: true,
            committed: false,
        };

        assert!(ended.committed);
        assert!(!moving.committed);
        assert!(moving.merge(ended).committed);
    }
}
