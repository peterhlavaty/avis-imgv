//! Changing what the keys do, from inside the viewer.
//!
//! Every shortcut the viewer listens for is listed with a sentence saying what
//! it does. Clicking one arms it; the next key pressed becomes the binding,
//! modifiers and all. Escape leaves it as it was, and Delete or Backspace
//! means "no key" — a state the list could already render and nothing could
//! produce.
//!
//! A key already spoken for is not refused — two things on one key is
//! sometimes what a person means — but it is pointed out. What counts as
//! spoken for is now where the two are *read*: "General" is live in every mode,
//! so a general binding on the same key as an image-view one is the collision
//! that actually bites, and the old rule was blind to exactly that.

use eframe::egui::{self, Color32, Key, Modifiers, RichText};

use crate::config::bindings::{self, Binding};
use crate::config::shortcut::{MOD_ALT, MOD_CTRL, MOD_MAC_CMD, MOD_SHIFT};
use crate::config::{Config, Shortcut};

/// What the editor did this frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// A binding changed and the configuration is worth writing out.
    Changed,
}

/// The editor's own state.
#[derive(Debug, Default)]
pub struct State {
    /// The row waiting for a key, by its position in the list.
    listening: Option<usize>,
    /// What the last change did, shown under the list. It was declared and read
    /// and never assigned, so a successful save said nothing at all.
    status: String,
    /// What is being searched for.
    query: String,
    /// Whether only the keys read where the user is are shown.
    this_mode_only: bool,
    /// A global reset that has been asked for and not confirmed.
    confirming_reset: bool,
}

impl State {
    /// Whether a row is armed, which is what the viewer has to mute for.
    ///
    /// Arming a row and pressing Delete used to send the photograph on screen
    /// to the bin *and* fail to capture, because `input::collect` runs before
    /// the window is drawn and nothing told it to stop.
    pub fn is_listening(&self) -> bool {
        self.listening.is_some()
    }
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
        .default_width(640.0)
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
        match captured(ui.ctx()) {
            Some(Captured::Bound(shortcut)) => {
                state.listening = None;
                bindings[index].set(config, shortcut.clone());
                state.status = format!("{} is now {}", bindings[index].name(), describe(&shortcut));
                outcome = Some(Outcome::Changed);
            }
            Some(Captured::Unbound) => {
                state.listening = None;
                bindings[index].set(config, Shortcut::new("", &[]));
                state.status = format!("{} has no key", bindings[index].name());
                outcome = Some(Outcome::Changed);
            }
            Some(Captured::Cancelled) => state.listening = None,
            None => {}
        }
    }

    ui.label(
        "Click a key to change it, then press the one you want. Escape leaves it alone; \
         Delete or Backspace means no key at all.",
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

    egui::ScrollArea::vertical()
        .max_height(520.0)
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
        });

    ui.add_space(8.0);
    outcome = footer(ui, state, config, &bindings).or(outcome);

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
fn is_everywhere(binding: &Binding) -> bool {
    binding.scope() == crate::config::registry::Scope::Everywhere
}

/// Whether a row survives the search box.
///
/// Over the key as well as the name and the sentence: "what is on F3" is asked
/// as often as "what is the key for stacking".
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
    let listening = state.listening == Some(index);
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
            let label = if listening {
                "press a key…".to_string()
            } else {
                binding
                    .get(config)
                    .map(describe)
                    .unwrap_or_else(|| "no key".to_string())
            };

            let button = egui::Button::new(RichText::new(label).monospace());
            if ui.add(button).clicked() {
                state.listening = if listening { None } else { Some(index) };
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

/// The colour of the note saying two things share a key.
const CLASH: Color32 = Color32::from_rgb(215, 175, 110);

/// The name of another binding on the same key, if there is one.
///
/// Decided by where the two are *read* rather than by which heading they were
/// filed under. The old rule compared only within a section, on the sound
/// ground that the gallery and the image view are never on screen at once — but
/// "General" is live in every mode, so the collision it was blind to is exactly
/// the one that bites: Quit on the gallery's scroll key means the folder
/// scrolls and the program exits.
pub fn clash(config: &Config, bindings: &[Binding], index: usize) -> Option<&'static str> {
    let binding = &bindings[index];
    let pressed = binding.get(config)?;

    // A row with no key cannot collide with the other rows that have none.
    if pressed.key.trim().is_empty() {
        return None;
    }

    bindings
        .iter()
        .enumerate()
        .find(|(other, candidate)| {
            *other != index
                && candidate.scope().overlaps(binding.scope())
                && candidate
                    .get(config)
                    .is_some_and(|found| found.kbd_shortcut == pressed.kbd_shortcut)
        })
        .map(|(_, candidate)| candidate.name())
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

        found.push(format!(
            "{} and {} are both on {key}, and both are read in {}",
            binding.name(),
            other,
            binding.scope().label()
        ));
    }

    found
}

/// How a shortcut reads on its button: `Ctrl + Plus`.
pub fn describe(shortcut: &Shortcut) -> String {
    if shortcut.key.trim().is_empty() {
        return "no key".to_string();
    }

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

/// What an armed row saw.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Captured {
    Bound(Shortcut),
    /// Delete or Backspace: no key at all.
    Unbound,
    /// Escape: leave it as it was.
    Cancelled,
}

/// The key pressed this frame, as a shortcut.
fn captured(ctx: &egui::Context) -> Option<Captured> {
    let pressed = ctx.input(|input| {
        input.events.iter().find_map(|event| match event {
            egui::Event::Key {
                key,
                pressed: true,
                modifiers,
                ..
            } => Some((*key, *modifiers)),
            _ => None,
        })
    });

    let (key, modifiers) = pressed?;

    if key == Key::Escape {
        return Some(Captured::Cancelled);
    }

    // A state `describe` could already render and nothing could produce.
    if matches!(key, Key::Delete | Key::Backspace) && modifiers.is_none() {
        return Some(Captured::Unbound);
    }

    Some(Captured::Bound(Shortcut::new(
        &canonical(key),
        &modifier_names(modifiers),
    )))
}

/// The modifiers held, in the words the file uses.
///
/// `cmd` is emitted on macOS rather than folding Command into Ctrl, because on
/// that platform they are two different keys and a file that says `ctrl` asks
/// for the one nobody presses.
fn modifier_names(modifiers: Modifiers) -> Vec<&'static str> {
    let mut names: Vec<&'static str> = Vec::new();

    if modifiers.ctrl {
        names.push(MOD_CTRL);
    }
    if modifiers.mac_cmd || (cfg!(target_os = "macos") && modifiers.command && !modifiers.ctrl) {
        names.push(MOD_MAC_CMD);
    }
    if modifiers.alt {
        names.push(MOD_ALT);
    }
    if modifiers.shift {
        names.push(MOD_SHIFT);
    }

    names
}

/// The spelling of a key that reads back as the same key.
///
/// `Key::name` gives "PageUp"; `capitalize_first_char` turns whatever is
/// written into "Pageup", which `Key::from_name` rejects — so those two of the
/// eighty names became the unreachable sentinel on the way back in. Written
/// canonically here, and both spellings accepted on read.
fn canonical(key: Key) -> String {
    key.name().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::registry::Scope;

    #[test]
    fn a_shortcut_reads_as_its_parts() {
        assert_eq!(describe(&Shortcut::new("Plus", &[MOD_CTRL])), "Ctrl + Plus");
        assert_eq!(describe(&Shortcut::new("f", &[])), "f");
        assert_eq!(
            describe(&Shortcut::new("q", &[MOD_CTRL, MOD_SHIFT])),
            "Ctrl + Shift + q"
        );
    }

    /// A row with no key says so rather than drawing an empty button.
    #[test]
    fn a_binding_with_no_key_says_so() {
        assert_eq!(describe(&Shortcut::new("", &[])), "no key");
    }

    #[test]
    fn two_things_on_one_key_in_one_scope_are_noticed() {
        let mut config = Config::default();
        let bindings = bindings::all();

        let fit = bindings.iter().position(|b| b.name() == "Fit").unwrap();
        let zoom = bindings
            .iter()
            .position(|b| b.name() == "Zoom step")
            .unwrap();

        let wanted = config.image_view.sc_fit.clone();
        bindings[zoom].set(&mut config, wanted);

        assert_eq!(clash(&config, &bindings, fit), Some("Zoom step"));
    }

    /// The collision the old rule was blind to, and the one that bites: a key
    /// read in every mode against one read in a view.
    #[test]
    fn a_key_read_everywhere_collides_with_one_read_in_a_view() {
        let mut config = Config::default();
        config.general.sc_exit = config.grid_view.sc_scroll.clone();

        let said = clashes(&config);
        assert!(
            said.iter().any(|line| line.contains("Quit")),
            "the collision was not reported: {said:?}"
        );
    }

    /// And the one it got right: the two views are never both on screen.
    #[test]
    fn the_two_views_may_still_share_a_key() {
        let mut config = Config::default();
        config.grid_view.sc_scroll = config.image_view.sc_fit.clone();

        let bindings = bindings::all();
        let fit = bindings.iter().position(|b| b.name() == "Fit").unwrap();

        assert_eq!(clash(&config, &bindings, fit), None);
    }

    /// A fresh configuration has no collisions at all, which is the assertion
    /// the startup warning rests on.
    #[test]
    fn the_defaults_do_not_collide() {
        let said = clashes(&Config::default());

        assert!(said.is_empty(), "the defaults collide: {said:?}");
    }

    /// Two rows with no key are not a collision.
    #[test]
    fn unbound_rows_do_not_collide_with_each_other() {
        let mut config = Config::default();
        config.general.sc_exit = Shortcut::new("", &[]);
        config.general.sc_menu = Shortcut::new("", &[]);

        let bindings = bindings::all();
        let quit = bindings.iter().position(|b| b.name() == "Quit").unwrap();

        assert_eq!(clash(&config, &bindings, quit), None);
    }

    /// Every key name the editor writes has to read back as the same key, or a
    /// rebind is a command made unreachable.
    #[test]
    fn every_captured_key_name_reads_back() {
        for key in Key::ALL {
            let written = canonical(*key);
            assert!(
                crate::config::shortcut::names_a_key(&written),
                "{written} does not read back as a key"
            );
        }
    }

    #[test]
    fn escape_leaves_a_row_alone_and_delete_clears_it() {
        assert_eq!(pressed(Key::Escape, Modifiers::NONE), Captured::Cancelled);
        assert_eq!(pressed(Key::Delete, Modifiers::NONE), Captured::Unbound);
        assert_eq!(pressed(Key::Backspace, Modifiers::NONE), Captured::Unbound);

        // With a modifier it is an ordinary binding: Ctrl+Delete is a key
        // somebody may want.
        assert!(matches!(
            pressed(Key::Delete, Modifiers::CTRL),
            Captured::Bound(_)
        ));
    }

    fn pressed(key: Key, modifiers: Modifiers) -> Captured {
        let ctx = egui::Context::default();
        ctx.begin_pass(egui::RawInput {
            events: vec![egui::Event::Key {
                key,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers,
            }],
            ..Default::default()
        });

        captured(&ctx).expect("a key was pressed")
    }

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

    #[test]
    fn only_the_keys_read_everywhere_is_a_real_filter() {
        let bindings = bindings::all();

        assert!(bindings.iter().any(is_everywhere));
        assert!(bindings.iter().any(|b| !is_everywhere(b)));
        assert!(bindings
            .iter()
            .filter(|b| b.name() == "Fit")
            .all(|b| b.scope() == Scope::ImageView));
    }
}
