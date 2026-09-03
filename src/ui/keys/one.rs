//! Every key that does one thing, on a card about that one thing.
//!
//! The list of ninety bindings is an index: it is where somebody goes who
//! wants to see what the keyboard looks like. It is the wrong place to *change*
//! a key from, and it was the only one — a row there is a single button, so a
//! command had exactly one key and rebinding it meant losing the key it had.
//!
//! This is the other half. It is opened from the row in the list, from the
//! key on the settings page, from the cheat sheet, and from **Keys for…** on
//! any of the ten surfaces that carry that row — so a person who can see a
//! thing changes its keys from where they are standing, which is the same rule
//! the rest of the settings follow. It holds one command, every key bound to
//! it, a cross against each and a button that takes the next key pressed.

use eframe::egui::{self, RichText};

use super::{capture, describe, State, CLASH};
use crate::config::bindings::{self, Binding};
use crate::config::{Chord, Config};

/// The command whose keys are being edited.
///
/// A record of three fields; what may be done to it is on `State`, which is
/// what everything outside this module holds.
#[derive(Debug, Clone)]
pub struct Editing {
    /// Its path in the configuration file, which is its identity. A path
    /// rather than a position, because the list of bindings is a filtered view
    /// over the registry and its positions are not stable.
    path: &'static str,
    /// Whether the next key pressed is being taken as one of its keys.
    pub(super) listening: bool,
    /// What the last thing done here said, under the list.
    pub(super) status: String,
}

impl Editing {
    pub fn new(path: &'static str) -> Editing {
        Editing {
            path,
            listening: false,
            status: String::new(),
        }
    }

    pub fn is_listening(&self) -> bool {
        self.listening
    }
}

/// What the card is asking of the program, beyond a changed configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Asked {
    /// The list of every key, which this card is one command out of.
    EveryKey,
}

/// Draws the card, if a command is being edited.
///
/// Returns whether the configuration changed and whatever the card asked for.
/// A path the registry no longer has draws nothing and says nothing: the card
/// is put down rather than left up and empty.
pub fn card(ui: &mut egui::Ui, state: &mut State, config: &mut Config) -> (bool, Option<Asked>) {
    let Some(editing) = &state.one else {
        return (false, None);
    };

    let bindings = bindings::all();
    let Some(index) = bindings.iter().position(|b| b.path() == editing.path) else {
        state.one = None;
        return (false, None);
    };

    contents(ui, state, config, &bindings, index)
}

/// Which command the card is about, for the deck's own bar.
///
/// The same rule as a menu's first row: a card is opened from eleven surfaces
/// and the answer is different from each, so it says what it was asked about
/// rather than leaving it to be worked out.
pub fn about(state: &State) -> Option<String> {
    let editing = state.one.as_ref()?;
    let bindings = bindings::all();

    bindings
        .iter()
        .find(|binding| binding.path() == editing.path)
        .map(|binding| binding.name().to_string())
}

fn contents(
    ui: &mut egui::Ui,
    state: &mut State,
    config: &mut Config,
    bindings: &[Binding],
    index: usize,
) -> (bool, Option<Asked>) {
    let binding = &bindings[index];
    let mut changed = false;
    let mut asked = None;

    // Before anything is drawn, so the key that arms the card on one frame is
    // not also read as the key being bound on the same one.
    if state.is_listening() {
        changed |= take_a_key(ui.ctx(), state, config, binding);
    }

    ui.weak(binding.description());
    ui.add_space(8.0);

    match binding.fixed() {
        Some(key) => fixed(ui, key),
        None if !binding.exists(config) => {
            ui.weak("This command is not in your configuration, so it has no keys.");
        }
        None => changed |= editable(ui, state, config, bindings, index),
    }

    ui.add_space(10.0);
    ui.separator();

    // The way out to the index, which is where this card ends for the same
    // reason every menu ends on the settings page that owns it.
    if ui
        .button("All keys…")
        .on_hover_text("The whole keyboard, with a sentence against each key")
        .clicked()
    {
        asked = Some(Asked::EveryKey);
    }

    if !state.status_of_one().is_empty() {
        ui.add_space(4.0);
        ui.weak(state.status_of_one().to_string());
    }

    (changed, asked)
}

/// A key the program reads for itself: drawn so the clash checker's findings
/// can be understood, and plainly not something this card can move.
fn fixed(ui: &mut egui::Ui, key: &'static str) {
    ui.horizontal(|ui| {
        ui.add_enabled(false, egui::Button::new(RichText::new(key).monospace()));
        ui.weak("The viewer reads this key itself; it cannot be changed.");
    });
}

/// The list of keys, and the two buttons under it.
fn editable(
    ui: &mut egui::Ui,
    state: &mut State,
    config: &mut Config,
    bindings: &[Binding],
    index: usize,
) -> bool {
    let binding = &bindings[index];
    let mut changed = false;

    let Some(shortcut) = binding.get(config).cloned() else {
        return false;
    };

    if shortcut.is_empty() {
        ui.weak("No key. This command is reached from a menu, or from nowhere at all.");
    }

    // Collected first, because a row may take a key away and the list cannot
    // be walked while it is being written to.
    let mut take_away = None;

    for (at, chord) in shortcut.chords().iter().enumerate() {
        ui.horizontal(|ui| {
            ui.add_enabled(
                false,
                egui::Button::new(RichText::new(describe::chord(chord)).monospace()),
            );

            // A cross rather than the Delete key that used to mean this. One
            // key or five, each is taken away where it is written, and the
            // last one leaving is how a command is left with none.
            if ui
                .small_button("✖")
                .on_hover_text("Take this key away")
                .clicked()
            {
                take_away = Some(at);
            }

            if let Some(other) = super::clash_on(config, bindings, index, chord) {
                ui.label(RichText::new(format!("also {other}")).color(CLASH))
                    .on_hover_text(format!(
                        "{other} is read in {}, and so is this one, so one press does both",
                        binding.scope().label()
                    ));
            }
        });
    }

    if let Some(at) = take_away {
        let mut shortcut = shortcut.clone();
        let gone = describe::chord(&shortcut.chords()[at]);
        shortcut.remove(at);

        let left = shortcut.is_empty();
        binding.set(config, shortcut);
        state.said(if left {
            format!("{gone} taken away; {} has no key now", binding.name())
        } else {
            format!("{gone} taken away")
        });

        return true;
    }

    ui.add_space(8.0);

    ui.horizontal(|ui| {
        let listening = state.is_listening();

        let label = if listening {
            "press a key…"
        } else {
            "Add a key…"
        };

        if ui
            .button(label)
            .on_hover_text("The next key pressed is added to the list. Escape leaves it alone.")
            .clicked()
        {
            state.listen(!listening);
        }

        let differs = binding.changed(config);
        if ui
            .add_enabled(differs, egui::Button::new("↺"))
            .on_hover_text("Put this command back to the keys a fresh configuration gives it")
            .clicked()
        {
            binding.reset(config);
            state.said(format!("Put {} back", binding.name()));
            changed = true;
        }
    });

    changed
}

/// Takes this frame's press as another key for the command.
fn take_a_key(
    ctx: &egui::Context,
    state: &mut State,
    config: &mut Config,
    binding: &Binding,
) -> bool {
    match capture::captured(ctx) {
        Some(capture::Captured::Pressed(chord)) => {
            state.listen(false);
            add(state, config, binding, chord)
        }
        Some(capture::Captured::Cancelled) => {
            state.listen(false);
            false
        }
        None => false,
    }
}

/// Writes a chord into the binding, saying why when nothing happens.
///
/// A key another command already has is *not* refused — two things on one key
/// is sometimes what a person means, and the row beside it says so — but a key
/// this command already has is, because adding it twice would leave a row that
/// does nothing and a cross that appears to remove nothing.
fn add(state: &mut State, config: &mut Config, binding: &Binding, chord: Chord) -> bool {
    let Some(mut shortcut) = binding.get(config).cloned() else {
        return false;
    };

    let said = describe::chord(&chord);

    if !shortcut.add(chord) {
        state.said(format!("{} is already {said}", binding.name()));
        return false;
    }

    state.said(format!(
        "{} is now {}",
        binding.name(),
        describe::describe(&shortcut)
    ));
    binding.set(config, shortcut);
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Shortcut;

    /// Every word the card paints.
    fn drawn(state: &mut State, config: &mut Config) -> Vec<String> {
        let ctx = egui::Context::default();
        let input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1200.0, 900.0),
            )),
            ..Default::default()
        };

        let output = ctx.run(input, |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                card(ui, state, config);
            });
        });

        crate::ui::drawn::text(&output)
    }

    /// Every key it holds is drawn, with the cross that takes it away and the
    /// button that adds another.
    #[test]
    fn the_card_draws_every_key_and_the_way_to_add_one() {
        let mut config = Config::default();
        let mut state = State::default();
        let bindings = bindings::all();
        let quit = bindings
            .iter()
            .find(|b| b.name() == "Quit")
            .expect("it is there");

        let mut shortcut = Shortcut::new("F13", &[]);
        shortcut.add(Chord::new("F14", &[crate::config::shortcut::MOD_CTRL]));
        quit.set(&mut config, shortcut);
        state.arm(quit.path());

        let drawn = drawn(&mut state, &mut config);

        for wanted in ["F13", "Ctrl + F14", "✖", "Add a key…"] {
            assert!(
                drawn.iter().any(|text| text.contains(wanted)),
                "{wanted} is not drawn: {drawn:?}"
            );
        }

        // And the card is still up: nothing about drawing it puts it down.
        assert!(state.editing_one());
    }

    /// A command with no keys says so rather than drawing an empty list, and
    /// still offers the button that gives it one.
    #[test]
    fn a_command_with_no_keys_says_so_and_can_still_be_given_one() {
        let mut config = Config::default();
        let mut state = State::default();
        let bindings = bindings::all();
        let quit = bindings
            .iter()
            .find(|b| b.name() == "Quit")
            .expect("it is there");

        quit.set(&mut config, Shortcut::unbound());
        state.arm(quit.path());

        let drawn = drawn(&mut state, &mut config);

        assert!(drawn.iter().any(|text| text.contains("No key")));
        assert!(drawn.iter().any(|text| text.contains("Add a key…")));
    }

    /// The card is about one command and says which, the way every menu in
    /// the program does.
    #[test]
    fn the_card_is_named_for_the_command_it_is_about() {
        let bindings = bindings::all();
        let quit = bindings
            .iter()
            .find(|b| b.name() == "Quit")
            .expect("it is there");

        let mut state = State::default();
        state.arm(quit.path());

        assert_eq!(about(&state), Some("Quit".to_string()));
        assert_eq!(about(&State::default()), None);
    }

    /// A second key is added and the first is left where it was, which is the
    /// whole feature.
    #[test]
    fn a_second_key_is_added_beside_the_first() {
        let mut config = Config::default();
        let mut state = State::default();
        let bindings = bindings::all();
        let quit = bindings
            .iter()
            .find(|b| b.name() == "Quit")
            .expect("it is there");

        state.arm(quit.path());
        let first = quit.get(&config).expect("it has one").clone();

        assert!(add(&mut state, &mut config, quit, Chord::new("F12", &[])));

        let now = quit.get(&config).expect("it still has one");
        assert_eq!(now.len(), first.len() + 1);
        assert_eq!(now.first(), first.first());
        assert!(now.holds(&Chord::new("F12", &[])));
    }

    /// And the same key twice is refused, with a sentence saying why rather
    /// than a row that does nothing.
    #[test]
    fn the_same_key_is_not_added_twice() {
        let mut config = Config::default();
        let mut state = State::default();
        let bindings = bindings::all();
        let quit = bindings
            .iter()
            .find(|b| b.name() == "Quit")
            .expect("it is there");

        state.arm(quit.path());
        quit.set(&mut config, Shortcut::new("F12", &[]));

        assert!(!add(&mut state, &mut config, quit, Chord::new("F12", &[])));
        assert_eq!(quit.get(&config).expect("one").len(), 1);
        assert!(state
            .one
            .as_ref()
            .expect("armed")
            .status
            .contains("already"));
    }

    /// A key another command already has is allowed: two things on one key is
    /// sometimes what a person means, and the row beside it says so.
    #[test]
    fn a_key_another_command_has_is_not_refused() {
        let mut config = Config::default();
        let mut state = State::default();
        let bindings = bindings::all();
        let quit = bindings
            .iter()
            .find(|b| b.name() == "Quit")
            .expect("it is there");
        let taken = config.general.sc_menu.first().expect("it has one").clone();

        state.arm(quit.path());
        assert!(add(&mut state, &mut config, quit, taken.clone()));
        assert!(quit.get(&config).expect("one").holds(&taken));
    }

    /// The window mutes the viewer's keys only while it is waiting for one.
    #[test]
    fn it_listens_only_when_it_was_asked_to() {
        let mut state = State::default();
        assert!(!state.editing_one());
        assert!(!state.is_listening());

        state.arm("general.sc_exit");
        assert!(state.editing_one());
        assert!(!state.is_listening());

        state.listen(true);
        assert!(state.is_listening());
    }
}
