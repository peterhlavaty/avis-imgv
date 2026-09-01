//! What the history watches, and what one difference in it looks like.
//!
//! The program is asked once a frame what it looks like, and the answer is
//! compared with the answer from the frame before. That is the whole recording
//! mechanism: no dispatcher calls anything here, so no dispatcher can forget
//! to, and a sixth route added next year is covered without being told about.
//!
//! # The cost, and why it is a borrow
//!
//! The comparison happens every frame and must not allocate. [`Watched`] is
//! the borrowed form — scalars and references, built from the live state at no
//! cost — and [`Snapshot`] is the owned one. `Watched::matches` compares the
//! two without building anything; `Watched::taken` clones, and is called only
//! on the frames where something actually moved, which is a few times a second
//! at the very most.
//!
//! What is *not* watched is anything derived. `Visible` and the stacks are
//! functions of the narrowing, the stacking and the folder, so putting those
//! back re-derives them, and a `Vec<usize>` per photograph per row of history
//! is saved.

use std::path::{Path, PathBuf};

use crate::app::mode::Mode;
use crate::config::Config;
use crate::view::image_view::viewports::Place;
use crate::view::narrow::Narrowing;
use crate::view::selection::Selection;

/// Which panels are up.
///
/// One struct rather than six bools loose in the snapshot, so that "a panel
/// was opened" is one row of history saying which, rather than six fields the
/// difference has to be hunted through.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Panels {
    pub menu: bool,
    pub side: bool,
    pub metrics: bool,
    pub tags: bool,
    pub filter: bool,
    pub filmstrip: bool,
    pub history: bool,
}

/// One panel: what to call it, and how to read it off the set.
type Named = (&'static str, fn(&Panels) -> bool);

impl Panels {
    /// Each panel, with the name to call it by on a row of the history.
    const EACH: &'static [Named] = &[
        ("the menu", |p| p.menu),
        ("the side panel", |p| p.side),
        ("the frame timings", |p| p.metrics),
        ("the keyword panel", |p| p.tags),
        ("the filter bar", |p| p.filter),
        ("the filmstrip", |p| p.filmstrip),
        ("the history", |p| p.history),
    ];

    /// The first panel that differs, and whether it is now up.
    ///
    /// One at a time because that is how they are toggled; a frame that
    /// changed two is described by the first, which is honest enough for a row
    /// in a list and never wrong about the direction.
    pub fn difference(&self, other: &Panels) -> Option<(&'static str, bool)> {
        Panels::EACH
            .iter()
            .find(|(_, read)| read(self) != read(other))
            .map(|(name, read)| (*name, read(other)))
    }
}

/// Everything the history watches, borrowed.
///
/// Built fresh every frame and thrown away, so it holds references to the two
/// things that are not scalars rather than copies of them.
pub struct Watched<'a> {
    pub folder: &'a Path,
    pub mode: Mode,
    pub panels: Panels,
    pub cursor: usize,
    pub place: Place,
    pub columns: usize,
    pub flattened: bool,
    pub advancing: bool,
    pub selection: &'a Selection,
    pub narrowing: &'a Narrowing,
}

impl Watched<'_> {
    /// Whether the program still looks the way the snapshot says.
    ///
    /// Allocation-free: every arm is a comparison against something already
    /// held. The two collections compare their lengths first, so the ordinary
    /// case of a large selection that has not moved costs one integer.
    pub fn matches(&self, was: &Snapshot) -> bool {
        self.mode == was.mode
            && self.panels == was.panels
            && self.cursor == was.cursor
            && self.place == was.place
            && self.columns == was.columns
            && self.flattened == was.flattened
            && self.advancing == was.advancing
            && self.folder == was.folder
            && self.selection == &was.selection
            && self.narrowing == &was.narrowing
    }

    /// The owned form, built only when something has moved.
    pub fn taken(&self) -> Snapshot {
        Snapshot {
            folder: self.folder.to_path_buf(),
            mode: self.mode,
            panels: self.panels,
            cursor: self.cursor,
            place: self.place,
            columns: self.columns,
            flattened: self.flattened,
            advancing: self.advancing,
            selection: self.selection.clone(),
            narrowing: self.narrowing.clone(),
        }
    }
}

/// Everything the history watches, owned.
#[derive(Debug, Clone, PartialEq)]
pub struct Snapshot {
    pub folder: PathBuf,
    pub mode: Mode,
    pub panels: Panels,
    pub cursor: usize,
    pub place: Place,
    pub columns: usize,
    pub flattened: bool,
    pub advancing: bool,
    pub selection: Selection,
    pub narrowing: Narrowing,
}

impl Snapshot {
    /// What moved between two snapshots, one entry per thing.
    ///
    /// Only what differs: a row of history that carried the whole state would
    /// be a row nobody could read and a great deal of memory spent saying
    /// "and everything else is as it was".
    pub fn diff(&self, now: &Snapshot) -> Vec<Change> {
        let mut changes = Vec::new();

        if self.folder != now.folder {
            changes.push(Change::Folder(self.folder.clone(), now.folder.clone()));
        }
        if self.mode != now.mode {
            changes.push(Change::Mode(self.mode, now.mode));
        }
        if self.panels != now.panels {
            changes.push(Change::Panels(self.panels, now.panels));
        }
        if self.cursor != now.cursor {
            changes.push(Change::Cursor(self.cursor, now.cursor));
        }
        if self.place != now.place {
            changes.push(Change::Place(self.place, now.place));
        }
        if self.columns != now.columns {
            changes.push(Change::Columns(self.columns, now.columns));
        }
        if self.flattened != now.flattened {
            changes.push(Change::Flattened(self.flattened, now.flattened));
        }
        if self.advancing != now.advancing {
            changes.push(Change::Advancing(self.advancing, now.advancing));
        }
        if self.selection != now.selection {
            changes.push(Change::Selection(
                Box::new(self.selection.clone()),
                Box::new(now.selection.clone()),
            ));
        }
        if self.narrowing != now.narrowing {
            changes.push(Change::Narrowing(
                Box::new(self.narrowing.clone()),
                Box::new(now.narrowing.clone()),
            ));
        }

        changes
    }
}

/// One thing that moved, and what it moved from and to.
///
/// Both halves, always, which is what makes every one of these runnable in
/// either direction without a second kind of recording.
#[derive(Clone)]
pub enum Change {
    Folder(PathBuf, PathBuf),
    Mode(Mode, Mode),
    Panels(Panels, Panels),
    Cursor(usize, usize),
    Place(Place, Place),
    Columns(usize, usize),
    Flattened(bool, bool),
    Advancing(bool, bool),
    Selection(Box<Selection>, Box<Selection>),
    Narrowing(Box<Narrowing>, Box<Narrowing>),
    /// The configuration, whole.
    ///
    /// Whole rather than one field, because the registry reaches a field
    /// through a typed accessor per kind and writing a value back through
    /// fourteen of those is fourteen chances to be subtly wrong about one. A
    /// `Config` is a few kilobytes beside a program that holds photographs in
    /// gigabytes, a settings change is a thing somebody does a handful of
    /// times in a session, and this cannot drift from the shape of the file:
    /// a field added tomorrow is covered without being mentioned here.
    Settings(Box<Config>, Box<Config>),
}

impl Change {
    /// Which slot this is about, so that two changes to the same one can be
    /// recognised as the same nudge continuing.
    pub fn slot(&self) -> Slot {
        match self {
            Change::Folder(..) => Slot::Folder,
            Change::Mode(..) => Slot::Mode,
            Change::Panels(..) => Slot::Panels,
            Change::Cursor(..) => Slot::Cursor,
            Change::Place(..) => Slot::Place,
            Change::Columns(..) => Slot::Columns,
            Change::Flattened(..) => Slot::Flattened,
            Change::Advancing(..) => Slot::Advancing,
            Change::Selection(..) => Slot::Selection,
            Change::Narrowing(..) => Slot::Narrowing,
            Change::Settings(..) => Slot::Settings,
        }
    }

    /// Whether this is a field of the configuration rather than a view.
    pub fn is_a_setting(&self) -> bool {
        matches!(self.slot(), Slot::Settings)
    }

    /// Whether this is the kind of thing a gesture produces a stream of.
    ///
    /// Zooming, panning and walking the folder arrive once a frame while the
    /// hand is moving. One row of history each would bury everything else in
    /// the list and make one press of undo worth a sixtieth of a second.
    pub fn is_continuous(&self) -> bool {
        matches!(self.slot(), Slot::Cursor | Slot::Place)
    }

    /// Folds a later change to the same slot into this one.
    ///
    /// The `before` is kept, because where the gesture *started* is where
    /// undoing it has to arrive; the `after` becomes wherever it has got to.
    pub fn absorb(&mut self, newer: &Change) {
        match (self, newer) {
            (Change::Folder(_, to), Change::Folder(_, now)) => *to = now.clone(),
            (Change::Mode(_, to), Change::Mode(_, now)) => *to = *now,
            (Change::Panels(_, to), Change::Panels(_, now)) => *to = *now,
            (Change::Cursor(_, to), Change::Cursor(_, now)) => *to = *now,
            (Change::Place(_, to), Change::Place(_, now)) => *to = *now,
            (Change::Columns(_, to), Change::Columns(_, now)) => *to = *now,
            (Change::Flattened(_, to), Change::Flattened(_, now)) => *to = *now,
            (Change::Advancing(_, to), Change::Advancing(_, now)) => *to = *now,
            (Change::Selection(_, to), Change::Selection(_, now)) => *to = now.clone(),
            (Change::Narrowing(_, to), Change::Narrowing(_, now)) => *to = now.clone(),
            (Change::Settings(_, to), Change::Settings(_, now)) => *to = now.clone(),
            // Different slots never meet here: the caller matches on `slot`
            // first, and folding one kind into another would lose both halves.
            _ => {}
        }
    }

    /// What this was, in a few words, for the row that stands for it.
    pub fn label(&self) -> String {
        match self {
            Change::Folder(_, to) => format!(
                "Opened {}",
                to.file_name().unwrap_or(to.as_os_str()).to_string_lossy()
            ),
            Change::Mode(_, to) => format!("Went to {}", to.label().to_lowercase()),
            Change::Panels(from, to) => match from.difference(to) {
                Some((name, true)) => format!("Opened {name}"),
                Some((name, false)) => format!("Closed {name}"),
                None => "Moved a panel".to_string(),
            },
            Change::Cursor(from, to) => match to > from {
                true => "Went forward".to_string(),
                false => "Went back".to_string(),
            },
            Change::Place(from, to) => match to.zoom.partial_cmp(&from.zoom) {
                Some(std::cmp::Ordering::Greater) => "Zoomed in".to_string(),
                Some(std::cmp::Ordering::Less) => "Zoomed out".to_string(),
                _ => "Panned".to_string(),
            },
            Change::Columns(_, to) => format!("{to} columns"),
            Change::Flattened(_, true) => "Read the sub-folders too".to_string(),
            Change::Flattened(_, false) => "Read this folder only".to_string(),
            Change::Advancing(_, true) => "Moved on after every mark".to_string(),
            Change::Advancing(_, false) => "Stayed put after a mark".to_string(),
            Change::Selection(_, to) => match to.len() {
                0 => "Picked nothing out".to_string(),
                1 => "Picked one out".to_string(),
                many => format!("Picked {many} out"),
            },
            Change::Narrowing(..) => "Narrowed the folder".to_string(),
            Change::Settings(from, to) => match crate::config::registry::rows()
                .iter()
                .find(|row| row.access.differs(from, to))
            {
                Some(row) => format!("Changed {}", row.label.to_lowercase()),
                None => "Changed a setting".to_string(),
            },
        }
    }
}

/// Printed as what it is and where it went, never as what it carries.
///
/// The derived form would put two whole configurations into one line of a log,
/// which is both unreadable and the reason `Config` would have to derive
/// `Debug` through every section it holds.
impl std::fmt::Debug for Change {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}({})", self.slot(), self.label())
    }
}

/// Which piece of state a change is about.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Slot {
    Folder,
    Mode,
    Panels,
    Cursor,
    Place,
    Columns,
    Flattened,
    Advancing,
    Selection,
    Narrowing,
    Settings,
}

#[cfg(test)]
mod tests {
    use super::*;
    use eframe::epaint::Vec2;

    fn snapshot() -> Snapshot {
        Snapshot {
            folder: PathBuf::from("/photos"),
            mode: Mode::Image,
            panels: Panels::default(),
            cursor: 0,
            place: Place::UNTOUCHED,
            columns: 4,
            flattened: false,
            advancing: false,
            selection: Selection::default(),
            narrowing: Narrowing::default(),
        }
    }

    fn watching<'a>(of: &'a Snapshot) -> Watched<'a> {
        Watched {
            folder: &of.folder,
            mode: of.mode,
            panels: of.panels,
            cursor: of.cursor,
            place: of.place,
            columns: of.columns,
            flattened: of.flattened,
            advancing: of.advancing,
            selection: &of.selection,
            narrowing: &of.narrowing,
        }
    }

    #[test]
    fn a_snapshot_matches_itself_and_nothing_is_recorded() {
        let was = snapshot();

        assert!(watching(&was).matches(&was));
        assert!(was.diff(&was).is_empty());
    }

    #[test]
    fn a_moved_cursor_is_one_change_and_nothing_else() {
        let was = snapshot();
        let mut now = snapshot();
        now.cursor = 7;

        assert!(!watching(&now).matches(&was));

        let changes = was.diff(&now);
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].slot(), Slot::Cursor);
        assert_eq!(changes[0].label(), "Went forward");
    }

    /// Two things moved on one frame are two changes in one row, so undoing
    /// puts both back together.
    #[test]
    fn two_things_moving_at_once_are_two_changes() {
        let was = snapshot();
        let mut now = snapshot();
        now.mode = Mode::Grid;
        now.columns = 6;

        let changes = was.diff(&now);

        assert_eq!(changes.len(), 2);
        assert_eq!(changes[0].slot(), Slot::Mode);
        assert_eq!(changes[1].slot(), Slot::Columns);
    }

    #[test]
    fn a_panel_says_which_one_and_which_way() {
        let now = Panels {
            tags: true,
            ..Panels::default()
        };

        assert_eq!(
            Panels::default().difference(&now),
            Some(("the keyword panel", true))
        );
        assert_eq!(
            now.difference(&Panels::default()),
            Some(("the keyword panel", false))
        );
        assert_eq!(Panels::default().difference(&Panels::default()), None);
    }

    #[test]
    fn a_panel_change_is_labelled_by_what_happened_to_it() {
        let was = snapshot();
        let mut now = snapshot();
        now.panels.filmstrip = true;

        assert_eq!(was.diff(&now)[0].label(), "Opened the filmstrip");
        assert_eq!(now.diff(&was)[0].label(), "Closed the filmstrip");
    }

    /// Only zooming and walking arrive once a frame; nothing else may be
    /// folded away, or a mode change would vanish into the one before it.
    #[test]
    fn only_the_cursor_and_the_place_are_continuous() {
        let was = snapshot();

        let mut moved = snapshot();
        moved.cursor = 1;
        assert!(was.diff(&moved)[0].is_continuous());

        let mut zoomed = snapshot();
        zoomed.place = Place {
            zoom: 2.0,
            pan: Vec2::ZERO,
        };
        assert!(was.diff(&zoomed)[0].is_continuous());

        let mut moded = snapshot();
        moded.mode = Mode::Grid;
        assert!(!was.diff(&moded)[0].is_continuous());

        let mut panelled = snapshot();
        panelled.panels.menu = true;
        assert!(!was.diff(&panelled)[0].is_continuous());
    }

    /// Folding keeps where the gesture started, which is where undoing it has
    /// to arrive.
    #[test]
    fn absorbing_keeps_the_beginning_and_takes_the_new_end() {
        let mut change = Change::Cursor(0, 1);
        change.absorb(&Change::Cursor(1, 2));
        change.absorb(&Change::Cursor(2, 9));

        match change {
            Change::Cursor(from, to) => {
                assert_eq!((from, to), (0, 9), "undo goes back to where it began");
            }
            other => panic!("{other:?}"),
        }
    }

    /// A zoom folded into a pan would lose one of them; the caller keys on the
    /// slot, and this is the guard that says so.
    #[test]
    fn absorbing_a_different_slot_changes_nothing() {
        let mut change = Change::Cursor(0, 1);
        change.absorb(&Change::Mode(Mode::Image, Mode::Grid));

        assert!(matches!(change, Change::Cursor(0, 1)));
    }

    #[test]
    fn zooming_in_and_out_are_told_apart_from_panning() {
        let out = Place {
            zoom: 1.0,
            pan: Vec2::ZERO,
        };
        let close = Place {
            zoom: 4.0,
            pan: Vec2::ZERO,
        };
        let moved = Place {
            zoom: 1.0,
            pan: Vec2::new(10.0, 0.0),
        };

        assert_eq!(Change::Place(out, close).label(), "Zoomed in");
        assert_eq!(Change::Place(close, out).label(), "Zoomed out");
        assert_eq!(Change::Place(out, moved).label(), "Panned");
    }

    /// A settings row names the field that moved, off the registry, so the
    /// row in the panel says which setting rather than "a setting".
    #[test]
    fn a_settings_change_names_the_field_that_moved() {
        let was = Config::default();
        let mut now = Config::default();
        now.general.restore_session = !was.general.restore_session;

        let change = Change::Settings(Box::new(was), Box::new(now));

        assert_eq!(change.label(), "Changed open where the last run left off");
    }
}
