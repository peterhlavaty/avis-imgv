//! Changing what the keys do, from inside the viewer.
//!
//! Every shortcut the viewer listens for is listed with a sentence saying what
//! it does. Clicking one arms it; the next key pressed becomes the binding,
//! modifiers and all. Escape leaves it as it was.
//!
//! A key already spoken for is not refused — two things on one key is
//! sometimes what a person means, and the sections make it obvious when it is
//! not — but it is pointed out.

use eframe::egui::{self, Color32, Key, Modifiers, RichText};

use crate::config::bindings::{self, Binding};
use crate::config::shortcut::{MOD_ALT, MOD_CTRL, MOD_SHIFT};
use crate::config::{Config, Shortcut};

/// What the editor did this frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// A binding changed and the configuration is worth writing out.
    Changed,
}

/// The editor's own state, which is only ever "which row is listening".
#[derive(Debug, Default)]
pub struct State {
    /// The row waiting for a key, by its position in the list.
    listening: Option<usize>,
    /// What the last save said, shown under the list.
    status: String,
}

/// Draws the editor as a window, returning whether anything changed.
pub fn show(
    ctx: &egui::Context,
    open: &mut bool,
    state: &mut State,
    config: &mut Config,
) -> Option<Outcome> {
    let mut outcome = None;

    // Nothing is listening once the window is shut, or the next key pressed
    // anywhere would land in a row nobody can see.
    if !*open {
        state.listening = None;
        return None;
    }

    egui::Window::new("Keyboard")
        .open(open)
        .default_width(560.0)
        .resizable(true)
        .show(ctx, |ui| {
            outcome = contents(ui, state, config);
        });

    outcome
}

fn contents(ui: &mut egui::Ui, state: &mut State, config: &mut Config) -> Option<Outcome> {
    let bindings = bindings::all();
    let mut outcome = None;

    if let Some(index) = state.listening {
        if let Some(pressed) = captured(ui.ctx()) {
            state.listening = None;

            if let Some(shortcut) = pressed {
                bindings[index].set(config, shortcut);
                outcome = Some(Outcome::Changed);
            }
        }
    }

    ui.label("Click a key to change it, then press the one you want. Escape leaves it alone.");
    ui.add_space(6.0);

    egui::ScrollArea::vertical()
        .max_height(520.0)
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for section in bindings::SECTIONS {
                let rows: Vec<usize> = bindings
                    .iter()
                    .enumerate()
                    .filter(|(_, binding)| binding.section == *section)
                    .map(|(index, _)| index)
                    .collect();

                if rows.is_empty() {
                    continue;
                }

                ui.add_space(8.0);
                ui.label(RichText::new(*section).heading());

                egui::Grid::new(("keys", section))
                    .num_columns(3)
                    .striped(true)
                    .spacing([14.0, 4.0])
                    .show(ui, |ui| {
                        for index in rows {
                            row(ui, state, config, &bindings, index);
                        }
                    });
            }
        });

    ui.add_space(8.0);
    ui.horizontal(|ui| {
        if ui.button("Put everything back to the defaults").clicked() {
            let defaults = Config::default();

            for binding in &bindings {
                if let Some(shortcut) = binding.get(&defaults).cloned() {
                    binding.set(config, shortcut);
                }
            }

            outcome = Some(Outcome::Changed);
        }

        if !state.status.is_empty() {
            ui.weak(&state.status);
        }
    });

    outcome
}

fn row(ui: &mut egui::Ui, state: &mut State, config: &Config, bindings: &[Binding], index: usize) {
    let binding = &bindings[index];
    let listening = state.listening == Some(index);

    ui.label(binding.name).on_hover_text(binding.description);

    let label = if listening {
        "press a key…".to_string()
    } else {
        binding
            .get(config)
            .map(describe)
            .unwrap_or_else(|| "unbound".to_string())
    };

    let button = egui::Button::new(RichText::new(label).monospace());
    if ui.add(button).clicked() {
        state.listening = if listening { None } else { Some(index) };
    }

    match clash(config, bindings, index) {
        Some(other) => {
            ui.label(RichText::new(format!("also {other}")).color(CLASH))
                .on_hover_text(binding.description);
        }
        None => {
            ui.weak(binding.description);
        }
    }

    ui.end_row();
}

/// The colour of the note saying two things share a key.
const CLASH: Color32 = Color32::from_rgb(215, 175, 110);

/// The name of another binding on the same key, if there is one.
///
/// Only within a section: the gallery and the image view are never on screen
/// at once, so sharing a key between them is not a clash.
pub fn clash(config: &Config, bindings: &[Binding], index: usize) -> Option<&'static str> {
    let binding = &bindings[index];
    let shortcut = binding.get(config)?;

    bindings
        .iter()
        .enumerate()
        .find(|(other, candidate)| {
            *other != index
                && candidate.section == binding.section
                && candidate
                    .get(config)
                    .is_some_and(|found| found.kbd_shortcut == shortcut.kbd_shortcut)
        })
        .map(|(_, candidate)| candidate.name)
}

/// Every pair of bindings that share a key, as a sentence each.
///
/// Called at startup, because a configuration written by an older build keeps
/// whatever it said for ever — `serde` only fills in the keys that are
/// missing, never the ones that have since moved. One such file on the
/// author's machine had zoom-in and show-more-images both on plain `Plus`,
/// which made the side-by-side view unreachable and said nothing about it.
pub fn clashes(config: &Config) -> Vec<String> {
    let bindings = bindings::all();
    let mut found: Vec<String> = Vec::new();

    for (index, binding) in bindings.iter().enumerate() {
        let Some(other) = clash(config, &bindings, index) else {
            continue;
        };

        // Each pair once: the second one round is the same clash.
        if found.iter().any(|said| said.contains(other)) {
            continue;
        }

        let key = binding
            .get(config)
            .map(describe)
            .unwrap_or_else(|| "that key".to_string());

        found.push(format!("{} and {} are both on {key}", binding.name, other));
    }

    found
}

/// How a shortcut reads on its button: `Ctrl + Plus`.
pub fn describe(shortcut: &Shortcut) -> String {
    let mut parts: Vec<String> = shortcut
        .modifiers
        .iter()
        .map(|modifier| capitalised(modifier))
        .collect();

    parts.push(shortcut.key.clone());
    parts.join(" + ")
}

fn capitalised(text: &str) -> String {
    let mut chars = text.chars();

    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

/// The key pressed this frame, as a shortcut.
///
/// `Some(None)` means the user pressed escape and wants out; `None` means
/// nothing has been pressed yet.
fn captured(ctx: &egui::Context) -> Option<Option<Shortcut>> {
    let (key, modifiers) = ctx.input(|input| {
        let pressed = input.events.iter().find_map(|event| match event {
            egui::Event::Key {
                key,
                pressed: true,
                modifiers,
                ..
            } => Some((*key, *modifiers)),
            _ => None,
        });

        (pressed.map(|(key, _)| key), pressed.map(|(_, m)| m))
    });

    let key = key?;
    if key == Key::Escape {
        return Some(None);
    }

    let modifiers = modifiers.unwrap_or(Modifiers::NONE);
    let mut names: Vec<&str> = Vec::new();

    if modifiers.ctrl || modifiers.command {
        names.push(MOD_CTRL);
    }
    if modifiers.alt {
        names.push(MOD_ALT);
    }
    if modifiers.shift {
        names.push(MOD_SHIFT);
    }

    Some(Some(Shortcut::new(key.name(), &names)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_shortcut_reads_as_its_parts() {
        assert_eq!(describe(&Shortcut::new("Plus", &[MOD_CTRL])), "Ctrl + Plus");
        assert_eq!(describe(&Shortcut::new("f", &[])), "f");
        assert_eq!(
            describe(&Shortcut::new("q", &[MOD_CTRL, MOD_SHIFT])),
            "Ctrl + Shift + q"
        );
    }

    #[test]
    fn two_things_on_one_key_in_one_section_are_noticed() {
        let mut config = Config::default();
        let bindings = bindings::all();

        let fit = bindings.iter().position(|b| b.name == "Fit").unwrap();
        let frame = bindings
            .iter()
            .position(|b| b.name == "White frame")
            .unwrap();

        assert_eq!(clash(&config, &bindings, fit), None);

        bindings[frame].set(&mut config, Shortcut::new("f", &[]));
        assert_eq!(clash(&config, &bindings, fit), Some("White frame"));
    }

    #[test]
    fn the_defaults_do_not_clash() {
        assert_eq!(clashes(&Config::default()), Vec::<String>::new());
    }

    /// A configuration written by an older build keeps whatever it said for
    /// ever: serde only fills in the keys that are missing, never the ones
    /// that have since moved.
    #[test]
    fn a_configuration_that_clashes_says_which_two() {
        let mut config = Config::default();
        // What an older build left behind on the author's machine, which made
        // the side-by-side view unreachable and said nothing about it.
        config.image_view.sc_more_images_shown = Shortcut::new("Plus", &[]);

        let said = clashes(&config);

        assert_eq!(said.len(), 1, "{said:?}");
        assert!(said[0].contains("Plus"), "{said:?}");
    }

    #[test]
    fn the_same_key_in_two_sections_is_not_a_clash() {
        let mut config = Config::default();
        let bindings = bindings::all();

        let scroll = bindings
            .iter()
            .position(|b| b.name == "Scroll down")
            .unwrap();
        let quit = bindings.iter().position(|b| b.name == "Quit").unwrap();

        let scrolling = config.grid_view.sc_scroll.clone();
        bindings[quit].set(&mut config, scrolling);

        assert_eq!(clash(&config, &bindings, scroll), None);
    }

    #[test]
    fn the_defaults_have_nothing_doubled_up() {
        let config = Config::default();
        let bindings = bindings::all();

        let doubled: Vec<&str> = (0..bindings.len())
            .filter(|index| clash(&config, &bindings, *index).is_some())
            .map(|index| bindings[index].name)
            .collect();

        assert!(doubled.is_empty(), "{doubled:?} share a key out of the box");
    }

    #[test]
    fn a_capitalised_modifier_survives_an_empty_name() {
        assert_eq!(capitalised(""), "");
        assert_eq!(capitalised("ctrl"), "Ctrl");
    }
}
