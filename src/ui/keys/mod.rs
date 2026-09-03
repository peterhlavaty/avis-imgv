//! Changing what the keys do, from inside the viewer.
//!
//! Two cards, and they are an index and a page. [`list`] draws every shortcut
//! the viewer listens for with a sentence saying what it does, which is what
//! somebody reads who wants to know what the keyboard looks like; [`one`]
//! holds a single command and every key bound to it, which is where a key is
//! added or taken away. The list used to be both, and being both is what
//! limited a command to one key: a row there is one button, so binding a
//! second key meant losing the first.
//!
//! A key already spoken for is not refused — two things on one key is
//! sometimes what a person means — but it is pointed out, against the key
//! rather than against the command, because a command with two keys can be
//! clear on one of them and taken on the other. What counts as spoken for is
//! where the two are *read*: "General" is live in every mode, so a general
//! binding on the same key as an image-view one is the collision that actually
//! bites, and the old rule was blind to exactly that.

mod capture;
mod describe;
mod list;
pub mod one;
mod shown;

use eframe::egui::{self, Color32};

use crate::config::bindings::{self, Binding};
use crate::config::{Chord, Config};

pub use describe::{chord, describe, describe_into};
pub use one::Editing;
pub use shown::{button, checkbox, of, publish, radio, scopes_for};

/// What the editor did this frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// A binding changed and the configuration is worth writing out.
    Changed,
}

/// The editor's own state, which both cards share.
///
/// One state rather than two because the card for one command is opened from
/// the list as often as from anywhere else, and a person who has just changed a
/// key in it should find the list saying so.
#[derive(Debug, Default)]
pub struct State {
    /// The command whose keys are being edited on a card of its own.
    pub(super) one: Option<Editing>,
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
    /// Arms the card holding every key bound to the command at `path`.
    ///
    /// Everything that offers to change a key comes through here: the row in
    /// the list, the key on the settings page, that row's own menu, the cheat
    /// sheet, and the ten `surface::bind_a_key` rows.
    pub fn arm(&mut self, path: &'static str) {
        self.one = Some(Editing::new(path));
    }

    /// Whether a key is being waited for, which is what the viewer has to mute
    /// for.
    ///
    /// Arming a row and pressing Delete used to send the photograph on screen
    /// to the bin *and* fail to capture, because `input::collect` runs before
    /// the card is drawn and nothing told it to stop.
    pub fn is_listening(&self) -> bool {
        self.one.as_ref().is_some_and(Editing::is_listening)
    }

    /// Whether one command's keys are being edited, which is a card in front.
    pub fn editing_one(&self) -> bool {
        self.one.is_some()
    }

    /// Puts that card down, when the deck says it is no longer on screen.
    pub fn close_one(&mut self) {
        self.one = None;
    }

    /// Waits for the next key pressed, or stops waiting.
    pub(super) fn listen(&mut self, to_a_key: bool) {
        if let Some(editing) = &mut self.one {
            editing.listening = to_a_key;
        }
    }

    /// What the card says under its list about the last thing done there.
    pub(super) fn said(&mut self, status: String) {
        if let Some(editing) = &mut self.one {
            editing.status = status;
        }
    }

    /// That sentence, for the card that draws it.
    pub(super) fn status_of_one(&self) -> &str {
        self.one
            .as_ref()
            .map(|editing| editing.status.as_str())
            .unwrap_or_default()
    }
}

/// Draws the list of every command.
///
/// Two answers: whether anything changed, and whether a row asked for the card
/// that holds one command's keys. A row arms the editor itself — [`State::arm`]
/// is the one door into that and the row is inside a grid inside a scrolling
/// area, four call frames from anything holding the deck — so the ask is
/// noticed here rather than threaded back through `row`.
pub fn list(ui: &mut egui::Ui, state: &mut State, config: &mut Config) -> (Option<Outcome>, Armed) {
    let was_editing = state.editing_one();
    let outcome = list::contents(ui, state, config);

    (outcome, Armed(state.editing_one() && !was_editing))
}

/// Whether a row asked for one command's keys on this frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Armed(pub bool);

/// The colour of the note saying two things share a key.
const CLASH: Color32 = Color32::from_rgb(215, 175, 110);

/// The name of another binding on one of this one's keys, if there is one.
pub fn clash(config: &Config, bindings: &[Binding], index: usize) -> Option<&'static str> {
    let pressed = bindings[index].get(config)?;

    pressed
        .chords()
        .iter()
        .find_map(|chord| clash_on(config, bindings, index, chord))
}

/// The name of another binding on this particular key, if there is one.
///
/// Per key rather than per command now that a command can carry several. A
/// binding whose first key is its own and whose second is taken is a real
/// collision on every press of the second, and saying so against the command
/// would leave a person looking at two keys with no idea which of them was
/// meant.
///
/// Decided by where the two are *read* rather than by which heading they were
/// filed under. The old rule compared only within a section, on the sound
/// ground that the gallery and the image view are never on screen at once — but
/// "General" is live in every mode, so the collision it was blind to is exactly
/// the one that bites: Quit on the gallery's scroll key means the folder
/// scrolls and the program exits.
pub fn clash_on(
    config: &Config,
    bindings: &[Binding],
    index: usize,
    pressed: &Chord,
) -> Option<&'static str> {
    clashing(config, bindings, index, pressed).map(|other| bindings[other].name())
}

/// Every pair of bindings that share a key, as a sentence each.
///
/// Called at startup, because a configuration written by an older build keeps
/// whatever it said for ever — `serde` only fills in the keys that are
/// missing, never the ones that have since moved. One such file on the
/// author's machine had zoom-in and show-more-images both on plain `Plus`,
/// which made the side-by-side view unreachable and said nothing about it.
///
/// A pair is said once *per key they share*: two commands on both of two keys
/// is two things gone wrong, and a person who fixes one of them needs to be
/// told about the other.
pub fn clashes(config: &Config) -> Vec<String> {
    let bindings = bindings::all();
    let mut said: Vec<(usize, usize, String)> = Vec::new();

    for (index, binding) in bindings.iter().enumerate() {
        let Some(pressed) = binding.get(config) else {
            continue;
        };

        for key in pressed.chords() {
            let Some(other) = clashing(config, &bindings, index, key) else {
                continue;
            };

            let pair = (index.min(other), index.max(other), chord(key));
            if !said.contains(&pair) {
                said.push(pair);
            }
        }
    }

    said.into_iter()
        .map(|(one, two, key)| {
            format!(
                "{} and {} are both on {key}, and both are read in {}",
                bindings[one].name(),
                bindings[two].name(),
                bindings[one].scope().label()
            )
        })
        .collect()
}

/// Which other binding is on this key, by its position in the list.
fn clashing(config: &Config, bindings: &[Binding], index: usize, pressed: &Chord) -> Option<usize> {
    let binding = &bindings[index];

    bindings
        .iter()
        .enumerate()
        .find(|(other, candidate)| {
            *other != index
                && candidate.scope().overlaps(binding.scope())
                && candidate
                    .get(config)
                    .is_some_and(|found| found.holds(pressed))
        })
        .map(|(other, _)| other)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::registry::Scope;
    use crate::config::Shortcut;

    /// Nothing the viewer ships with collides with anything else it ships
    /// with.
    ///
    /// The startup warning is for configurations people have edited; this is
    /// for the ones nobody has. It was written after a new key was given a
    /// default already taken by *Stacks*, which the checker looked straight
    /// past because that row claimed to be read only in the contact sheet
    /// while its key was being eaten in every mode.
    #[test]
    fn the_shipped_defaults_do_not_collide_with_each_other() {
        let said = clashes(&Config::default());
        assert!(said.is_empty(), "{said:#?}");
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

    /// Two rows with no key are not a collision.
    #[test]
    fn unbound_rows_do_not_collide_with_each_other() {
        let mut config = Config::default();
        config.general.sc_exit = Shortcut::unbound();
        config.general.sc_menu = Shortcut::unbound();

        let bindings = bindings::all();
        let quit = bindings.iter().position(|b| b.name() == "Quit").unwrap();

        assert_eq!(clash(&config, &bindings, quit), None);
    }

    /// A command clear on its first key and taken on its second collides, and
    /// the collision is reported against the second — which is the one that
    /// has to move.
    #[test]
    fn a_second_key_that_is_taken_is_a_collision_of_its_own() {
        let mut config = Config::default();
        let taken = config.general.sc_menu.first().expect("it has one").clone();

        config.general.sc_exit.add(taken.clone());

        let bindings = bindings::all();
        let quit = bindings.iter().position(|b| b.name() == "Quit").unwrap();

        assert_eq!(clash_on(&config, &bindings, quit, &taken), Some("Menu"));

        // And the first key, which nothing else has, is clear.
        let first = config.general.sc_exit.first().expect("it has one").clone();
        assert_eq!(clash_on(&config, &bindings, quit, &first), None);

        let said = clashes(&config);
        assert!(
            said.iter().any(|line| line.contains(&chord(&taken))),
            "the key was not named: {said:?}"
        );
    }

    /// Two commands sharing two keys is two things gone wrong, and fixing one
    /// leaves the other.
    #[test]
    fn a_pair_on_two_keys_is_said_twice() {
        let mut config = Config::default();

        config.general.sc_exit = Shortcut::new("F13", &[]);
        config.general.sc_exit.add(Chord::new("F14", &[]));
        config.general.sc_menu = config.general.sc_exit.clone();

        let said = clashes(&config);
        let ours: Vec<&String> = said.iter().filter(|line| line.contains("Quit")).collect();

        assert_eq!(ours.len(), 2, "{said:#?}");
    }

    #[test]
    fn only_the_keys_read_everywhere_is_a_real_filter() {
        let bindings = bindings::all();

        assert!(bindings.iter().any(list::is_everywhere));
        assert!(bindings.iter().any(|b| !list::is_everywhere(b)));
        assert!(bindings
            .iter()
            .filter(|b| b.name() == "Fit")
            .all(|b| b.scope() == Scope::ImageView));
    }

    /// The row that opens one command's keys asks for the card, so the deck
    /// puts it up.
    ///
    /// A row arms the editor itself, four call frames deep, and nothing said
    /// so: clicking a key in the list armed the editor, the deck did not hear
    /// about it, and the reconciliation at the foot of the frame put the
    /// editor straight back down again. From the outside the key was a button
    /// that did nothing at all.
    #[test]
    fn a_key_in_the_list_asks_for_the_card_that_holds_it() {
        let mut config = Config::default();
        let mut state = State::default();

        let ctx = egui::Context::default();
        let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1400.0, 1000.0));
        let frame = |input: egui::RawInput, state: &mut State, config: &mut Config| {
            let mut armed = Armed(false);
            let output = ctx.run(input, |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    let (_, asked) = list(ui, state, config);
                    armed = asked;
                });
            });

            (output, armed)
        };

        let quiet = egui::RawInput {
            screen_rect: Some(screen),
            ..Default::default()
        };

        // Drawn once to find where the key landed, and nothing is asked for on
        // a frame nobody clicked anything.
        let (output, armed) = frame(quiet.clone(), &mut state, &mut config);
        assert_eq!(armed, Armed(false));

        let bindings = bindings::all();
        let quit = bindings.iter().find(|b| b.name() == "Quit").unwrap();
        let key = describe(quit.get(&config).unwrap());
        let at = crate::ui::drawn::text_at(&output, &key).expect("the key is drawn");

        let clicked = egui::RawInput {
            screen_rect: Some(screen),
            events: vec![
                egui::Event::PointerMoved(at),
                egui::Event::PointerButton {
                    pos: at,
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers: egui::Modifiers::NONE,
                },
                egui::Event::PointerButton {
                    pos: at,
                    button: egui::PointerButton::Primary,
                    pressed: false,
                    modifiers: egui::Modifiers::NONE,
                },
            ],
            ..Default::default()
        };

        let (_, armed) = frame(clicked, &mut state, &mut config);

        assert_eq!(armed, Armed(true));
        assert!(state.editing_one());
    }
}
