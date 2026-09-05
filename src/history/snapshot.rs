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

use crate::choices::Choices;
use crate::collection::narrow::Narrowing;
use crate::collection::place::Place;
use crate::collection::selection::Selection;
use crate::config::Config;
use crate::mode::Mode;

/// Which panels are up.
///
/// One struct rather than six bools loose in the snapshot, so that "a panel
/// was opened" is one row of history saying which, rather than six fields the
/// difference has to be hunted through.
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, Copy, PartialEq, Eq, Default)]
#[serde(default)]
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
    ///
    /// Reachable from the crate so that `app::chrome` can hold the list of
    /// panels and the fields here against each other: a panel with a flag on
    /// `App` and no field in this struct is one a fullscreen mode would leave
    /// on the screen and the history would never put back.
    pub(crate) const EACH: &'static [Named] = &[
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
    /// How large the photograph is drawn, as the bottom bar says it.
    ///
    /// Carried and not compared, like the name: it is a function of the zoom
    /// and of the window, and a window resized under a viewport nobody touched
    /// would otherwise be a row of history.
    pub zoom_percent: f32,
    /// The photograph on screen.
    ///
    /// Deliberately not compared: it is a function of the cursor, and comparing
    /// it as well would make a row out of a folder being re-read under a
    /// cursor that never moved. It is carried so that a row can say *which*
    /// photograph was moved to rather than only that one was.
    pub showing: &'a Path,
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
            showing: self.showing.to_path_buf(),
            zoom_percent: self.zoom_percent,
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
#[derive(Debug, Clone)]
pub struct Snapshot {
    pub folder: PathBuf,
    /// The photograph that was on screen. Not compared; see [`Watched`].
    pub showing: PathBuf,
    /// How large it was drawn. Not compared; see [`Watched`].
    pub zoom_percent: f32,
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
            changes.push(Change::Cursor {
                from: self.cursor,
                to: now.cursor,
                name: name_of(&now.showing),
            });
        }
        if self.place != now.place {
            changes.push(Change::Place {
                from: self.place,
                to: now.place,
                percent: now.zoom_percent,
            });
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
#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub enum Change {
    Folder(PathBuf, PathBuf),
    Mode(Mode, Mode),
    Panels(Panels, Panels),
    /// The photograph moved to, and what it is called.
    ///
    /// The name is carried rather than looked up when the row is drawn,
    /// because by then the folder may hold something else entirely — and
    /// because a row is a record of what happened, not a question asked later.
    Cursor {
        from: usize,
        to: usize,
        name: String,
    },
    /// Where the photograph was zoomed and panned to, and how large that is.
    ///
    /// The percentage is what the bottom bar says, which is the number anybody
    /// means by "how far in am I". It cannot be worked out from `Place` alone:
    /// that is a multiple of the fitted size, and what the fitted size is
    /// depends on the window.
    Place {
        from: Place,
        to: Place,
        percent: f32,
    },
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
            Change::Cursor { .. } => Slot::Cursor,
            Change::Place { .. } => Slot::Place,
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

    /// Whether this is a change to which photographs are picked out.
    pub fn is_a_selection(&self) -> bool {
        matches!(self.slot(), Slot::Selection)
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
            (
                Change::Cursor { to, name, .. },
                Change::Cursor {
                    to: now,
                    name: called,
                    ..
                },
            ) => {
                *to = *now;
                // The name goes with the end of the gesture, which is where it
                // came to rest and the only photograph worth naming.
                name.clone_from(called);
            }
            (
                Change::Place { to, percent, .. },
                Change::Place {
                    to: now,
                    percent: reached,
                    ..
                },
            ) => {
                *to = *now;
                *percent = *reached;
            }
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
            Change::Cursor { from, to, name } => {
                let way = match to > from {
                    true => "Went forward",
                    false => "Went back",
                };

                match name.is_empty() {
                    true => way.to_string(),
                    false => format!("{way} to {name}"),
                }
            }
            Change::Place { from, to, percent } => {
                let way = match to.zoom.partial_cmp(&from.zoom) {
                    Some(std::cmp::Ordering::Greater) => "Zoomed in",
                    Some(std::cmp::Ordering::Less) => "Zoomed out",
                    // Same zoom, so the pan is what moved.
                    _ => return "Panned".to_string(),
                };

                match *percent > 0.0 {
                    true => format!("{way} to {percent:.0}%"),
                    // Before the first frame has been drawn there is no
                    // geometry to ask, and a percentage of nothing would be a
                    // lie rather than a gap.
                    false => way.to_string(),
                }
            }
            Change::Columns(_, to) => match to {
                1 => "Showed one column".to_string(),
                many => format!("Showed {many} columns"),
            },
            Change::Flattened(_, true) => "Read the sub-folders too".to_string(),
            Change::Flattened(_, false) => "Read this folder only".to_string(),
            Change::Advancing(_, true) => "Moved on after every mark".to_string(),
            Change::Advancing(_, false) => "Stayed put after a mark".to_string(),
            Change::Selection(_, to) => match to.len() {
                0 => "Picked nothing out".to_string(),
                1 => "Picked one out".to_string(),
                many => format!("Picked {many} out"),
            },
            Change::Narrowing(from, to) => narrowing(from, to),
            Change::Settings(from, to) => match crate::config::registry::rows()
                .iter()
                .find(|row| row.access.differs(from, to))
            {
                Some(row) => setting(row, to),
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

/// What the folder was narrowed or ordered by.
///
/// "Narrowed the folder" was true of eight different rules and useful for
/// none of them. Compared field by field, in the order somebody would think of
/// them, and the first difference is the row: one gesture changes one rule.
fn narrowing(from: &Narrowing, to: &Narrowing) -> String {
    if from.suspended != to.suspended {
        return match to.suspended {
            true => "Showed everything, keeping the rules".to_string(),
            false => "Put the rules back".to_string(),
        };
    }

    if from.sort != to.sort || from.descending != to.descending {
        let backwards = match to.descending {
            true => ", backwards",
            false => "",
        };

        return format!("Sorted by {}{backwards}", to.sort.label().to_lowercase());
    }

    let (was, now) = (&from.rules, &to.rules);

    if was.min_stars != now.min_stars || was.max_stars != now.max_stars {
        return match (now.min_stars, now.max_stars) {
            (0, most) if most >= crate::metadata::xmp::MAX_RATING as u8 => {
                "Showed any rating".to_string()
            }
            (least, most) if least == most => format!("Showed only {least} stars"),
            (least, most) if most >= crate::metadata::xmp::MAX_RATING as u8 => {
                format!("Showed {least} stars and up")
            }
            (least, most) => format!("Showed {least} to {most} stars"),
        };
    }

    if was.flag != now.flag {
        return format!("Showed {}", now.flag.label().to_lowercase());
    }

    if was.label != now.label {
        return format!("Showed {}", now.label.label().to_lowercase());
    }

    if was.name_contains != now.name_contains {
        return match now.name_contains.is_empty() {
            true => "Stopped searching the names".to_string(),
            false => format!("Searched the names for {}", now.name_contains),
        };
    }

    if was.keyword != now.keyword {
        return match now.keyword.is_empty() {
            true => "Stopped narrowing by keyword".to_string(),
            false => format!("Showed only the keyword {}", now.keyword),
        };
    }

    if was.extensions != now.extensions {
        return match now.extensions.is_empty() {
            true => "Showed every kind of file".to_string(),
            false => format!("Showed only {}", now.extensions),
        };
    }

    "Narrowed the folder".to_string()
}

/// What a settings row was changed *to*, not merely that it changed.
///
/// Read back through the same accessor the window writes with, so a row says
/// the value the file now holds. Anything with no plain reading — a set of
/// flags, a list of destinations — says only that it moved, which is honest.
fn setting(row: &crate::config::registry::Row, to: &Config) -> String {
    let label = row.label.to_lowercase();

    if let Some(on) = row.access.boolean(to) {
        return match on {
            true => format!("Turned on {label}"),
            false => format!("Turned off {label}"),
        };
    }

    if let Some(value) = row.access.int(to) {
        return format!("Set {label} to {value}");
    }

    if let Some(value) = row.access.float(to) {
        return format!("Set {label} to {value:.2}");
    }

    if let Some(choice) = row.access.choice(to) {
        return format!("Set {label} to {}", choice.replace('_', " "));
    }

    if let Some(text) = row.access.text(to) {
        return match text.is_empty() {
            true => format!("Cleared {label}"),
            false => format!("Set {label} to {text}"),
        };
    }

    format!("Changed {label}")
}

/// What a photograph is called, which is all a row needs of a path.
fn name_of(path: &Path) -> String {
    path.file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into()
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
    use crate::collection::place::Pan;

    fn snapshot() -> Snapshot {
        Snapshot {
            folder: PathBuf::from("/photos"),
            showing: PathBuf::from("/photos/DSC0001.jpg"),
            zoom_percent: 100.0,
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

    /// A change of zoom or pan, with the percentage it came to rest at.
    fn zoomed(from: Place, to: Place, percent: f32) -> Change {
        Change::Place { from, to, percent }
    }

    /// What a fullscreen mode sets the panels to when it takes the screen.
    ///
    /// `App::change_screen` puts them away by writing the default, so a panel
    /// added with a default of `true` would be a panel a slideshow left on
    /// screen. Asked through `EACH`, which is the exhaustive list.
    #[test]
    fn no_panel_is_up_by_default() {
        let away = Panels::default();

        for (name, read) in Panels::EACH {
            assert!(!read(&away), "{name} is up in the default");
        }
    }

    /// A move of the cursor, named by where it ended up.
    fn walked(from: usize, to: usize, name: &str) -> Change {
        Change::Cursor {
            from,
            to,
            name: name.to_string(),
        }
    }

    fn watching<'a>(of: &'a Snapshot) -> Watched<'a> {
        Watched {
            folder: &of.folder,
            showing: &of.showing,
            zoom_percent: of.zoom_percent,
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
        assert_eq!(changes[0].label(), "Went forward to DSC0001.jpg");
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

    /// Carried, not compared. A folder re-read under a cursor that never moved
    /// would otherwise be a row of history saying somebody had gone somewhere.
    #[test]
    fn the_photograph_on_screen_is_not_compared() {
        let was = snapshot();
        let mut now = snapshot();
        now.showing = PathBuf::from("/photos/something else.jpg");

        assert!(watching(&now).matches(&was), "only the cursor decides");
        assert!(was.diff(&now).is_empty());
    }

    /// The row keeps the whole name. Fitting it to the panel is the drawing's
    /// business, and the hover shows all of it.
    #[test]
    fn a_long_name_is_kept_whole_in_the_row() {
        let long = "a very long file name indeed that will not fit in any panel.jpg";

        let was = snapshot();
        let mut now = snapshot();
        now.cursor = 1;
        now.showing = PathBuf::from("/photos").join(long);

        assert_eq!(was.diff(&now)[0].label(), format!("Went forward to {long}"));
    }

    #[test]
    fn going_back_says_so_and_names_the_photograph() {
        let mut was = snapshot();
        was.cursor = 5;

        let mut now = snapshot();
        now.cursor = 0;
        now.showing = PathBuf::from("/photos/DSC0009.jpg");

        assert_eq!(was.diff(&now)[0].label(), "Went back to DSC0009.jpg");
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
            pan: Pan::NONE,
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
        let mut change = walked(0, 1, "a.jpg");
        change.absorb(&walked(1, 2, "b.jpg"));
        change.absorb(&walked(2, 9, "i.jpg"));

        match change {
            Change::Cursor { from, to, name } => {
                assert_eq!((from, to), (0, 9), "undo goes back to where it began");
                assert_eq!(name, "i.jpg", "and it is named by where it came to rest");
            }
            other => panic!("{other:?}"),
        }
    }

    /// A zoom folded into a pan would lose one of them; the caller keys on the
    /// slot, and this is the guard that says so.
    #[test]
    fn absorbing_a_different_slot_changes_nothing() {
        let mut change = walked(0, 1, "a.jpg");
        change.absorb(&Change::Mode(Mode::Image, Mode::Grid));

        assert!(matches!(change, Change::Cursor { from: 0, to: 1, .. }));
    }

    #[test]
    fn zooming_in_and_out_are_told_apart_from_panning() {
        let out = Place {
            zoom: 1.0,
            pan: Pan::NONE,
        };
        let close = Place {
            zoom: 4.0,
            pan: Pan::NONE,
        };
        let moved = Place {
            zoom: 1.0,
            pan: Pan(10.0, 0.0),
        };

        assert_eq!(zoomed(out, close, 250.0).label(), "Zoomed in to 250%");
        assert_eq!(zoomed(close, out, 70.4).label(), "Zoomed out to 70%");
        assert_eq!(zoomed(out, moved, 100.0).label(), "Panned");

        // Before the first frame there is no geometry to ask, and a percentage
        // of nothing would be a lie rather than a gap.
        assert_eq!(zoomed(out, close, 0.0).label(), "Zoomed in");
    }

    /// "Narrowed the folder" was true of eight different rules and useful for
    /// none of them.
    #[test]
    fn narrowing_says_which_rule_changed() {
        use crate::config::kinds::{FlagRule, SortBy};

        let plain = Narrowing::default();

        let mut suspended = plain.clone();
        suspended.suspended = true;
        assert_eq!(
            narrowing(&plain, &suspended),
            "Showed everything, keeping the rules"
        );
        assert_eq!(narrowing(&suspended, &plain), "Put the rules back");

        let mut sorted = plain.clone();
        sorted.sort = SortBy::Name;
        sorted.descending = true;
        assert!(
            narrowing(&plain, &sorted).starts_with("Sorted by "),
            "{}",
            narrowing(&plain, &sorted)
        );
        assert!(narrowing(&plain, &sorted).ends_with(", backwards"));

        let mut stars = plain.clone();
        stars.rules.min_stars = 3;
        assert_eq!(narrowing(&plain, &stars), "Showed 3 stars and up");

        let mut exact = plain.clone();
        exact.rules.min_stars = 2;
        exact.rules.max_stars = 2;
        assert_eq!(narrowing(&plain, &exact), "Showed only 2 stars");

        let mut flagged = plain.clone();
        flagged.rules.flag = FlagRule::Picked;
        assert!(narrowing(&plain, &flagged).starts_with("Showed "));

        let mut searched = plain.clone();
        searched.rules.name_contains = "tatra".to_string();
        assert_eq!(narrowing(&plain, &searched), "Searched the names for tatra");
        assert_eq!(narrowing(&searched, &plain), "Stopped searching the names");

        let mut keyword = plain.clone();
        keyword.rules.keyword = "Tatras".to_string();
        assert_eq!(
            narrowing(&plain, &keyword),
            "Showed only the keyword Tatras"
        );
    }

    /// Nothing to point at is still a sentence rather than an empty row.
    #[test]
    fn a_narrowing_with_nothing_to_name_still_says_something() {
        let plain = Narrowing::default();

        assert_eq!(narrowing(&plain, &plain), "Narrowed the folder");
    }

    /// A row says the value the setting was changed *to*, read back through
    /// the same accessor the window writes with.
    #[test]
    fn a_setting_says_what_it_was_set_to() {
        let was = Config::default();

        let mut counted = Config::default();
        counted.history.remember = 25;
        assert_eq!(
            Change::Settings(Box::new(was.clone()), Box::new(counted)).label(),
            "Set actions to remember to 25"
        );

        let mut switched = Config::default();
        switched.history.panel_visible = true;
        assert_eq!(
            Change::Settings(Box::new(was.clone()), Box::new(switched)).label(),
            "Turned on show the history panel"
        );
    }

    /// A settings row names the field that moved, off the registry, so the
    /// row in the panel says which setting rather than "a setting".
    #[test]
    fn a_settings_change_names_the_field_that_moved() {
        let was = Config::default();
        let mut now = Config::default();
        now.general.restore_session = !was.general.restore_session;

        let change = Change::Settings(Box::new(was), Box::new(now));

        assert_eq!(
            change.label(),
            "Turned off open where the last run left off"
        );
    }
}
