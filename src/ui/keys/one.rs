//! Every key that does one thing, in a window about that one thing.
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

/// What the window is asking of the program, beyond a changed configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Asked {
    /// The list of every key, which this window is one command out of.
    EveryKey,
}

/// Draws the window, if a command is being edited.
///
/// Returns whether the configuration changed and whatever the window asked
/// for. Drawn *after* the list, so `set_modal_layer` leaves this one in front:
/// it is opened from the list as often as from anywhere else, and a window
/// that opens behind the one that opened it reads as nothing happening.
pub fn show(ctx: &egui::Context, state: &mut State, config: &mut Config) -> (bool, Option<Asked>) {
    let Some(editing) = &state.one else {
        return (false, None);
    };

    let bindings = bindings::all();
    let Some(index) = bindings.iter().position(|b| b.path() == editing.path) else {
        // A path the registry no longer has. Nothing to draw and nothing to
        // say: shut the window rather than leave an empty one up.
        state.one = None;
        return (false, None);
    };

    let mut changed = false;
    let mut asked = None;
    let mut open = true;

    let shown = egui::Window::new(RichText::new(title(&bindings[index])).heading())
        .id(egui::Id::new("keys-for-one-command"))
        // Above the windows rather than among them. An `Area` keeps the
        // position it had in its order, and egui raises one only when it is
        // *new* — so the second time this window was opened from the list it
        // was drawn underneath it, which with a list nine hundred pixels tall
        // and opaque reads as the button having stopped working. It is a modal
        // over whatever opened it, and `Order::Foreground` is that in one word
        // rather than a "was it just opened" flag and a `move_to_top`.
        .order(egui::Order::Foreground)
        .open(&mut open)
        .default_width(420.0)
        .resizable(false)
        .collapsible(false)
        .show(ctx, |ui| {
            let (touched, wanted) = contents(ui, state, config, &bindings, index);
            changed = touched;
            asked = wanted;
        });

    crate::utils::in_front(ctx, shown.as_ref());

    if !open || asked.is_some() {
        state.one = None;
    }

    (changed, asked)
}

/// The window's own name, which says which command it is about.
///
/// The same rule as a menu's first row: a window drawn over the thing it
/// belongs to still has to say what it was asked about, because it is opened
/// from eleven surfaces and the answer is different from each.
fn title(binding: &Binding) -> String {
    format!("Keys — {}", binding.name())
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

    // Before anything is drawn, so the key that arms the window on one frame
    // is not also read as the key being bound on the same one.
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

    // The way out to the index, which is where this window ends for the same
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
/// can be understood, and plainly not something this window can move.
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

    /// Every word the window paints.
    ///
    /// Two passes and a screen to draw on: a `Window` is laid out from the
    /// size its contents came to on the frame before, so the first one places
    /// it off screen and paints nothing that survives the clip.
    fn drawn(state: &mut State, config: &mut Config) -> Vec<String> {
        let ctx = egui::Context::default();
        let input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1200.0, 900.0),
            )),
            ..Default::default()
        };

        let _ = ctx.run(input.clone(), |ctx| {
            show(ctx, state, config);
        });

        let output = ctx.run(input, |ctx| {
            show(ctx, state, config);
        });

        output
            .shapes
            .iter()
            .filter_map(|clipped| match &clipped.shape {
                egui::Shape::Text(text) => Some(text.galley.text().to_string()),
                _ => None,
            })
            .collect()
    }

    /// Every key it holds is drawn, with the cross that takes it away and the
    /// button that adds another.
    ///
    /// Written after the window drew nothing on the second opening: an `Area`
    /// keeps the place it had in its order and egui raises one only when it is
    /// *new*, so it went behind the list that opened it. Nothing here can see a
    /// z-order, but a window that has stopped being drawn at all fails this,
    /// and that is the shape the fault took from the outside.
    #[test]
    fn the_window_draws_every_key_and_the_way_to_add_one() {
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

        for wanted in ["Keys — Quit", "F13", "Ctrl + F14", "✖", "Add a key…"] {
            assert!(
                drawn.iter().any(|text| text.contains(wanted)),
                "{wanted} is not drawn: {drawn:?}"
            );
        }

        // And the window is still up: nothing about drawing it shuts it.
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

    /// The window is about one command and says which, the way every menu in
    /// the program does.
    #[test]
    fn the_window_is_named_for_the_command_it_is_about() {
        let bindings = bindings::all();
        let quit = bindings
            .iter()
            .find(|b| b.name() == "Quit")
            .expect("it is there");

        assert_eq!(title(quit), "Keys — Quit");
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
