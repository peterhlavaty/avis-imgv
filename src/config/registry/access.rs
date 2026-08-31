//! How a registry row reaches the value behind it.
//!
//! A pair of accessors rather than one, which is the shape `bindings` has
//! always used: reading happens every frame a page is open and must not need a
//! mutable borrow of the whole configuration. The closures are non-capturing,
//! so they coerce to the `fn` pointers held here and the whole table can be a
//! `static` rather than a `Vec` allocated on every frame.

use crate::config::{Config, Shortcut};

/// One variant of an enumerated field, as the radio group draws it.
pub struct Choice {
    /// What the file holds, which is what a forum answer quotes.
    pub value: &'static str,
    /// What the control says.
    pub label: &'static str,
    /// The line under it. Fewer than five choices get a sentence each, which
    /// is the shape the slideshow's `Motion` control already uses and the best
    /// control in the program.
    pub sentence: &'static str,
}

/// A row that is a button rather than a value.
///
/// They are in the registry so they are searchable like everything else: "config
/// file" has to be a query that lands somewhere.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Run {
    /// Open the configuration file with whatever the system uses.
    OpenConfigFile,
    ShowConfigFolder,
    OpenLogFile,
    ShowLogFolder,
    /// Save the session and start again.
    Restart,
    /// Write out only what differs from the defaults.
    ExportChanges,
    ImportChanges,
    /// Put everything back, having written a backup first.
    ResetEverything,
}

/// A list or a tree with an editor of its own.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum List {
    /// Where photographs are sent, with a digit each.
    Destinations,
    /// The keyword tree.
    Categories,
    /// Which metadata tags the side panel shows.
    MetadataTags,
    /// Commands bound to keys.
    UserActions,
    /// The two context menu lists, drawn as one table with a column saying
    /// where each entry appears.
    ContextMenu,
    /// The six rating keys, whose editor is the six rows on Keys and mouse.
    RatingKeys,
    /// The five colour label keys, likewise.
    LabelKeys,
}

/// How to reach a value, and what kind of value it is.
pub enum Access {
    Bool(fn(&Config) -> bool, fn(&mut Config, bool)),
    /// A whole count. `unit` is what the number is measured in, drawn beside
    /// the control and never in a tooltip.
    Int {
        get: fn(&Config) -> i64,
        set: fn(&mut Config, i64),
        min: i64,
        max: i64,
        unit: &'static str,
        /// Whether a rail is drawn, or only a typed box. A control whose effect
        /// appears at the next launch is worse than a number, because it looks
        /// like it is doing something.
        rail: bool,
    },
    Float {
        get: fn(&Config) -> f32,
        set: fn(&mut Config, f32),
        min: f32,
        max: f32,
        unit: &'static str,
        rail: bool,
    },
    /// A closed set, drawn as radios with a sentence under each.
    Enum {
        get: fn(&Config) -> &'static str,
        set: fn(&mut Config, &str),
        choices: &'static [Choice],
    },
    Text(fn(&Config) -> String, fn(&mut Config, String)),
    /// Text in the placeholder grammar, drawn with the chips and a live
    /// rendering from the photograph in hand.
    Template(fn(&Config) -> String, fn(&mut Config, String)),
    /// A path, drawn with a picker. `None` where the field is optional.
    Path(
        fn(&Config) -> Option<String>,
        fn(&mut Config, Option<String>),
    ),
    /// A hex colour, or nothing.
    Colour(
        fn(&Config) -> Option<String>,
        fn(&mut Config, Option<String>),
    ),
    /// A list with an editor of its own.
    Records(List, fn(&Config) -> usize),
    /// A set of named booleans, drawn as ticks: one decision made of parts.
    Flags {
        get: fn(&Config, &str) -> bool,
        set: fn(&mut Config, &str, bool),
        options: &'static [Choice],
    },
    /// One of the sixty keyboard fields.
    Key(fn(&Config) -> &Shortcut, fn(&mut Config) -> &mut Shortcut),
    /// One of the six rating shortcuts, by its position in the list.
    RatingKey(usize),
    /// One of the five colour label shortcuts.
    LabelKey(usize),
    /// The shortcut on a user action, which is the one shortcut in the file the
    /// editor could not reach.
    ActionKey(usize),
    /// A key the program reads and does not let anybody change.
    ///
    /// Entered rather than left out so the clash checker can see it and a
    /// search for "cheat sheet" finds one.
    Fixed(&'static str),
    /// A value the window shows and never sets: `version`, and the page the
    /// window was left on.
    ReadOnly(fn(&Config) -> String),
    /// A button.
    Run(Run),
}

impl Access {
    /// The value as a whole number, where it is one.
    pub fn int(&self, config: &Config) -> Option<i64> {
        match self {
            Access::Int { get, .. } => Some(get(config)),
            _ => None,
        }
    }

    /// Writes a whole number, clamped to the range the row declares.
    ///
    /// The control's range and the consumer's clamp come from the same row, so
    /// they cannot drift — and hand-editing still wins, because nothing here
    /// touches a value the window did not set.
    pub fn set_int(&self, config: &mut Config, value: i64) {
        if let Access::Int { set, min, max, .. } = self {
            set(config, value.clamp(*min, *max));
        }
    }

    pub fn float(&self, config: &Config) -> Option<f32> {
        match self {
            Access::Float { get, .. } => Some(get(config)),
            _ => None,
        }
    }

    pub fn set_float(&self, config: &mut Config, value: f32) {
        if let Access::Float { set, min, max, .. } = self {
            set(config, value.clamp(*min, *max));
        }
    }

    pub fn boolean(&self, config: &Config) -> Option<bool> {
        match self {
            Access::Bool(get, _) => Some(get(config)),
            _ => None,
        }
    }

    pub fn set_bool(&self, config: &mut Config, value: bool) {
        if let Access::Bool(_, set) = self {
            set(config, value);
        }
    }

    /// The value as text, for a string, a template, a path or a colour.
    pub fn text(&self, config: &Config) -> Option<String> {
        match self {
            Access::Text(get, _) | Access::Template(get, _) => Some(get(config)),
            Access::Path(get, _) | Access::Colour(get, _) => Some(get(config).unwrap_or_default()),
            Access::ReadOnly(get) => Some(get(config)),
            _ => None,
        }
    }

    pub fn set_text(&self, config: &mut Config, value: String) {
        match self {
            Access::Text(_, set) | Access::Template(_, set) => set(config, value),
            Access::Path(_, set) | Access::Colour(_, set) => {
                let trimmed = value.trim().to_string();
                set(config, (!trimmed.is_empty()).then_some(trimmed));
            }
            _ => {}
        }
    }

    /// Whether one named boolean of a set is on.
    pub fn flag(&self, config: &Config, name: &str) -> Option<bool> {
        match self {
            Access::Flags { get, .. } => Some(get(config, name)),
            _ => None,
        }
    }

    pub fn set_flag(&self, config: &mut Config, name: &str, on: bool) {
        if let Access::Flags { set, .. } = self {
            set(config, name, on);
        }
    }

    /// Which variant an enumerated field holds, as the file spells it.
    pub fn choice(&self, config: &Config) -> Option<&'static str> {
        match self {
            Access::Enum { get, .. } => Some(get(config)),
            _ => None,
        }
    }

    pub fn set_choice(&self, config: &mut Config, value: &str) {
        if let Access::Enum { set, choices, .. } = self {
            // Only a variant the row declares: an unknown string would
            // deserialise to the default and look like nothing happened.
            if choices.iter().any(|choice| choice.value == value) {
                set(config, value);
            }
        }
    }

    /// Whether this field differs between two configurations.
    pub fn differs(&self, a: &Config, b: &Config) -> bool {
        match self {
            Access::Bool(get, _) => get(a) != get(b),
            Access::Int { get, .. } => get(a) != get(b),
            // Compared as bits rather than by `==`, because a field somebody
            // hand-edited to a NaN would otherwise always read as changed and
            // always read as unchanged at the same time.
            Access::Float { get, .. } => get(a).to_bits() != get(b).to_bits(),
            Access::Enum { get, .. } => get(a) != get(b),
            Access::Text(get, _) | Access::Template(get, _) => get(a) != get(b),
            Access::Path(get, _) | Access::Colour(get, _) => get(a) != get(b),
            Access::Records(_, count) => count(a) != count(b),
            Access::Flags { get, options, .. } => options
                .iter()
                .any(|option| get(a, option.value) != get(b, option.value)),
            Access::Key(get, _) => !same_key(get(a), get(b)),
            Access::RatingKey(i) => match (a.tags.sc_rating.get(*i), b.tags.sc_rating.get(*i)) {
                (Some(a), Some(b)) => !same_key(a, b),
                (a, b) => a.is_some() != b.is_some(),
            },
            Access::LabelKey(i) => match (a.tags.sc_label.get(*i), b.tags.sc_label.get(*i)) {
                (Some(a), Some(b)) => !same_key(a, b),
                (a, b) => a.is_some() != b.is_some(),
            },
            Access::ActionKey(i) => {
                match (
                    a.image_view.user_actions.get(*i),
                    b.image_view.user_actions.get(*i),
                ) {
                    (Some(a), Some(b)) => !same_key(&a.shortcut, &b.shortcut),
                    (a, b) => a.is_some() != b.is_some(),
                }
            }
            // Nothing anybody can change is nothing that can differ.
            Access::Fixed(_) | Access::Run(_) => false,
            Access::ReadOnly(get) => get(a) != get(b),
        }
    }

    /// The shortcut this row is bound to, where it is a key at all.
    pub fn shortcut<'a>(&self, config: &'a Config) -> Option<&'a Shortcut> {
        match self {
            Access::Key(get, _) => Some(get(config)),
            Access::RatingKey(i) => config.tags.sc_rating.get(*i),
            Access::LabelKey(i) => config.tags.sc_label.get(*i),
            Access::ActionKey(i) => config
                .image_view
                .user_actions
                .get(*i)
                .map(|action| &action.shortcut),
            _ => None,
        }
    }

    /// Replaces the shortcut, where there is one to replace.
    pub fn set_shortcut(&self, config: &mut Config, shortcut: Shortcut) {
        match self {
            Access::Key(_, set) => *set(config) = shortcut,
            Access::RatingKey(i) => {
                if let Some(field) = config.tags.sc_rating.get_mut(*i) {
                    *field = shortcut;
                }
            }
            Access::LabelKey(i) => {
                if let Some(field) = config.tags.sc_label.get_mut(*i) {
                    *field = shortcut;
                }
            }
            Access::ActionKey(i) => {
                if let Some(action) = config.image_view.user_actions.get_mut(*i) {
                    action.shortcut = shortcut;
                }
            }
            _ => {}
        }
    }

    /// Whether this row can be bound to a key at all, changeable or not.
    pub fn is_a_key(&self) -> bool {
        matches!(
            self,
            Access::Key(..)
                | Access::RatingKey(_)
                | Access::LabelKey(_)
                | Access::ActionKey(_)
                | Access::Fixed(_)
        )
    }

    /// Whether the interface may write to this row.
    pub fn is_writable(&self) -> bool {
        !matches!(
            self,
            Access::Fixed(_) | Access::Run(_) | Access::ReadOnly(_)
        )
    }

    /// Puts this field back to what a fresh configuration holds.
    pub fn reset(&self, config: &mut Config) {
        let fresh = Config::default();

        match self {
            Access::Bool(get, set) => set(config, get(&fresh)),
            Access::Int { get, set, .. } => set(config, get(&fresh)),
            Access::Float { get, set, .. } => set(config, get(&fresh)),
            Access::Enum { get, set, .. } => set(config, get(&fresh)),
            Access::Text(get, set) | Access::Template(get, set) => set(config, get(&fresh)),
            Access::Path(get, set) | Access::Colour(get, set) => set(config, get(&fresh)),
            Access::Records(list, _) => reset_records(*list, config, &fresh),
            Access::Flags { get, set, options } => {
                for option in *options {
                    set(config, option.value, get(&fresh, option.value));
                }
            }
            Access::Key(get, _) => {
                let wanted = get(&fresh).clone();
                self.set_shortcut(config, wanted);
            }
            Access::RatingKey(i) => {
                if let Some(wanted) = fresh.tags.sc_rating.get(*i).cloned() {
                    self.set_shortcut(config, wanted);
                }
            }
            Access::LabelKey(i) => {
                if let Some(wanted) = fresh.tags.sc_label.get(*i).cloned() {
                    self.set_shortcut(config, wanted);
                }
            }
            // A user action is the user's own; there is no default to go back
            // to, and putting an empty shortcut there would silence it.
            Access::ActionKey(_) | Access::Fixed(_) | Access::Run(_) | Access::ReadOnly(_) => {}
        }
    }
}

fn reset_records(list: List, config: &mut Config, fresh: &Config) {
    match list {
        List::Destinations => config.cull.destinations = fresh.cull.destinations.clone(),
        List::Categories => config.tags.categories = fresh.tags.categories.clone(),
        List::MetadataTags => config.general.metadata_tags = fresh.general.metadata_tags.clone(),
        List::UserActions => config.image_view.user_actions = fresh.image_view.user_actions.clone(),
        List::ContextMenu => {
            config.image_view.context_menu = fresh.image_view.context_menu.clone();
            config.grid_view.context_menu = fresh.grid_view.context_menu.clone();
        }
        List::RatingKeys => config.tags.sc_rating = fresh.tags.sc_rating.clone(),
        List::LabelKeys => config.tags.sc_label = fresh.tags.sc_label.clone(),
    }
}

/// Two shortcuts meaning the same key press.
fn same_key(a: &Shortcut, b: &Shortcut) -> bool {
    a.key.eq_ignore_ascii_case(&b.key) && a.modifiers == b.modifiers
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::registry;

    #[test]
    fn a_key_reads_back_what_was_written_to_it() {
        let mut config = Config::default();
        let row = registry::row("general.sc_exit").expect("the registry has it");

        row.access
            .set_shortcut(&mut config, Shortcut::new("F9", &["ctrl"]));

        assert_eq!(
            row.access.shortcut(&config).map(|s| s.key.as_str()),
            Some("F9")
        );
        assert!(row.changed(&config));
    }

    #[test]
    fn a_reset_puts_a_field_back() {
        let mut config = Config::default();
        let row = registry::row("cache.ram_budget_mb").expect("the registry has it");

        row.access.set_int(&mut config, 8192);
        assert!(row.changed(&config));

        row.access.reset(&mut config);
        assert!(!row.changed(&config));
    }

    /// A key the program reads and nobody can change is in the table anyway,
    /// so the clash checker can see it.
    #[test]
    fn a_fixed_key_is_not_writable() {
        let fixed = Access::Fixed("Escape");

        assert!(fixed.is_a_key());
        assert!(!fixed.is_writable());
    }
}
