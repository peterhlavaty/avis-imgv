//! Every key the viewer listens for, with a sentence against each.

use eframe::egui::{self, RichText};

use super::{clash, describe, Outcome, State, CLASH};
use crate::config::bindings::{self, Binding};
use crate::config::Config;

pub(super) fn contents(
    ui: &mut egui::Ui,
    state: &mut State,
    config: &mut Config,
) -> Option<Outcome> {
    let bindings = bindings::all();
    let mut outcome = None;

    ui.label(
        "Click a key to see every key that does it, and to add or take one away. \
         A command can have as many as you like.",
    );
    ui.add_space(6.0);

    ui.horizontal(|ui| {
        ui.add(
            egui::TextEdit::singleline(&mut state.query)
                .hint_text("Search by name, by what it does, or by the key itself")
                .desired_width(360.0),
        );

        ui.checkbox(&mut state.this_mode_only, "Only the keys read everywhere")
            .on_hover_text(
                "The bindings that are live in every mode, which are the ones that can \
                 collide with anything else",
            );
    });

    ui.add_space(6.0);

    let needle = state.query.trim().to_lowercase();

    // No height of its own: the card is the window, so the list is as tall as
    // what is left of it. The 520 points this asked for were the height of a
    // window that had to choose one, and in a card they left two thirds of the
    // screen empty under a scrolling list of ninety rows.
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let mut drawn = 0;

            for section in bindings::SECTIONS {
                let rows: Vec<usize> = bindings
                    .iter()
                    .enumerate()
                    .filter(|(_, binding)| bindings::heading(binding) == *section)
                    .filter(|(_, binding)| !state.this_mode_only || is_everywhere(binding))
                    .filter(|(_, binding)| binding.exists(config))
                    .filter(|(_, binding)| matches(binding, config, &needle))
                    .map(|(index, _)| index)
                    .collect();

                if rows.is_empty() {
                    continue;
                }

                drawn += rows.len();

                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    ui.label(RichText::new(*section).heading());

                    if needle.is_empty()
                        && ui
                            .small_button("Put this section back")
                            .on_hover_text("Only the keys under this heading")
                            .clicked()
                    {
                        for index in &rows {
                            bindings[*index].reset(config);
                        }
                        state.status = format!("Put the {section} keys back");
                        outcome = Some(Outcome::Changed);
                    }
                });

                egui::Grid::new(("keys", section))
                    .num_columns(4)
                    .striped(true)
                    .spacing([14.0, 4.0])
                    .show(ui, |ui| {
                        for index in rows {
                            if row(ui, state, config, &bindings, index) {
                                outcome = Some(Outcome::Changed);
                            }
                        }
                    });
            }

            if drawn == 0 {
                ui.add_space(10.0);
                ui.weak(format!("No key matches \"{}\".", state.query.trim()));
            }

            // Inside the list rather than under it, which is where the
            // settings card puts its own footer and for the same reason: a row
            // of buttons below a scrolling area that fills the card is a row
            // pushed off the bottom of it.
            ui.add_space(12.0);
            ui.separator();
            outcome = footer(ui, state, config, &bindings).or(outcome);
        });

    outcome
}

/// The buttons under the list.
fn footer(
    ui: &mut egui::Ui,
    state: &mut State,
    config: &mut Config,
    bindings: &[Binding],
) -> Option<Outcome> {
    let mut outcome = None;
    // Only the rows this file actually has: nine user-action rows are written
    // into the table and a file with two actions has two of them.
    let editable = bindings
        .iter()
        .filter(|b| b.is_editable() && b.exists(config))
        .count();

    ui.horizontal(|ui| {
        // Named and confirmed. It used to walk every row on one click, with no
        // confirmation of any kind and a label that did not say how many.
        if ui
            .button(format!("Put the {editable} key bindings back"))
            .on_hover_text("Every key, back to what a fresh configuration binds it to")
            .clicked()
        {
            state.confirming_reset = true;
        }

        if !state.status.is_empty() {
            ui.weak(&state.status);
        }
    });

    if state.confirming_reset {
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(format!("Put all {editable} bindings back to the defaults?"))
                    .color(CLASH),
            );

            if ui.button("Yes, all of them").clicked() {
                for binding in bindings {
                    binding.reset(config);
                }
                state.confirming_reset = false;
                state.status = format!("Put {editable} bindings back");
                outcome = Some(Outcome::Changed);
            }

            if ui.button("Leave them").clicked() {
                state.confirming_reset = false;
            }
        });
    }

    outcome
}

/// Whether a binding is read in every mode.
pub(super) fn is_everywhere(binding: &Binding) -> bool {
    binding.scope() == crate::config::registry::Scope::Everywhere
}

/// Whether a row survives the search box.
///
/// Over the keys as well as the name and the sentence: "what is on F3" is
/// asked as often as "what is the key for stacking", and a command's second
/// key is as findable as its first because `describe` names them both.
fn matches(binding: &Binding, config: &Config, needle: &str) -> bool {
    if needle.is_empty() {
        return true;
    }

    let key = binding
        .get(config)
        .map(describe)
        .or_else(|| binding.fixed().map(str::to_string))
        .unwrap_or_default();

    binding.name().to_lowercase().contains(needle)
        || binding.description().to_lowercase().contains(needle)
        || key.to_lowercase().contains(needle)
        || binding.path().contains(needle)
}

fn row(
    ui: &mut egui::Ui,
    state: &mut State,
    config: &mut Config,
    bindings: &[Binding],
    index: usize,
) -> bool {
    let binding = &bindings[index];
    let mut changed = false;

    ui.label(binding.name())
        .on_hover_text(binding.description());

    match binding.fixed() {
        // A key the program reads for itself. Drawn so the clash checker's
        // findings can be understood, and greyed so it is plain it cannot move.
        Some(key) => {
            ui.add_enabled(false, egui::Button::new(RichText::new(key).monospace()))
                .on_disabled_hover_text("The viewer reads this key itself; it cannot be changed");
        }
        None => {
            // Opens the window for this one command rather than arming the row
            // here. A row is one button and a command has as many keys as
            // somebody gave it, so there is nothing here for a second key to
            // be added *to* — and the armed row was invisible whenever the
            // list happened to be scrolled somewhere else.
            let label = binding
                .get(config)
                .map(describe)
                .unwrap_or_else(|| "no key".to_string());

            let button = egui::Button::new(RichText::new(label).monospace());
            if ui
                .add(button)
                .on_hover_text("Every key that does this, to add to or take from")
                .clicked()
            {
                state.arm(binding.path());
            }
        }
    }

    // Per row, so putting one key back does not cost the other sixty-four.
    if binding.is_editable() {
        let differs = binding.changed(config);
        let reset = ui.add_enabled(differs, egui::Button::new("↺").small());

        if reset
            .on_hover_text("Put this one back to its default")
            .clicked()
        {
            binding.reset(config);
            state.status = format!("Put {} back", binding.name());
            changed = true;
        }
    } else {
        ui.label("");
    }

    match clash(config, bindings, index) {
        Some(other) => {
            ui.label(RichText::new(format!("also {other}")).color(CLASH))
                .on_hover_text(format!(
                    "{} is read in {}, and so is this one, so one press does both",
                    other,
                    binding.scope().label()
                ));
        }
        None => {
            ui.weak(binding.description());
        }
    }

    ui.end_row();
    changed
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The search box is over the key as well as the name and the sentence.
    #[test]
    fn the_search_finds_a_row_by_its_key() {
        let config = Config::default();
        let bindings = bindings::all();
        let quit = bindings.iter().find(|b| b.name() == "Quit").unwrap();

        assert!(matches(quit, &config, "quit"));
        assert!(matches(quit, &config, "close the viewer"));
        assert!(matches(
            quit,
            &config,
            &describe(quit.get(&config).unwrap()).to_lowercase()
        ));
        assert!(!matches(quit, &config, "zzzznothing"));
    }

    /// And by a second key, which is as much the answer to "what is on F13" as
    /// the first.
    #[test]
    fn the_search_finds_a_row_by_a_key_added_to_it() {
        let mut config = Config::default();
        config
            .general
            .sc_exit
            .add(crate::config::Chord::new("F13", &[]));

        let bindings = bindings::all();
        let quit = bindings.iter().find(|b| b.name() == "Quit").unwrap();

        assert!(matches(quit, &config, "f13"));
    }
}
