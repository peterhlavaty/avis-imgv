//! Looking once a frame, and deciding whether what changed is a new row.
//!
//! Two things stop the history filling up with noise.
//!
//! The first is that nothing is looked at while a drag is live. A zoom done
//! with the pointer down moves every frame it is held, and the row worth
//! keeping is the whole gesture: where it started and where it was let go.
//! That is the same rule the settings window already follows for the fields
//! the caches are built from — the change lands on the frame the gesture ends.
//!
//! The second is folding. Keys repeat, and a wheel notch is not a drag: forty
//! photographs walked past with the arrow held would be forty rows, and one
//! press of undo would be worth a sixtieth of a second. So a change to a slot
//! that arrives soon after a change to the same slot is folded into it,
//! keeping where it started.
//!
//! Both are about *rows*, never about what is applied. Nothing is suppressed:
//! the program does what it was told on the frame it was told, and this only
//! decides how much of it is one line in a list.

use std::time::{Duration, SystemTime};

use crate::config::Config;

use super::snapshot::{Change, Slot, Snapshot, Watched};

/// What the last look found, and whether the next one should record.
///
/// No `Debug`: it holds a whole configuration, and printing one would be both
/// unreadable and a reason for `Config` to derive `Debug` through every section
/// it carries.
#[derive(Default)]
pub struct Watcher {
    /// What the program looked like when it was last looked at.
    was: Option<Snapshot>,
    /// The configuration as it was when it was last looked at.
    ///
    /// Kept apart from the snapshot because it is compared differently: the
    /// snapshot is a dozen scalars and two collections, and this is several
    /// kilobytes that only the registry knows how to compare field by field.
    settings: Option<Box<Config>>,
    /// Whether the next look should record what it finds.
    ///
    /// Cleared for one look after the history itself has moved the program.
    /// Undoing changes the state, and a watcher that recorded that would file
    /// the undo as a new deed and never let go of the end of the list.
    resync: bool,
}

impl Watcher {
    /// A watcher that already knows what the configuration was at startup.
    ///
    /// Seeded here rather than on the first look, because the look at the
    /// configuration only happens on the frames it has been written: a watcher
    /// that seeded itself then would answer "nothing to compare with" to the
    /// first settings change of the session and quietly lose it.
    pub fn watching(settings: &Config) -> Watcher {
        Watcher {
            settings: Some(Box::new(settings.clone())),
            ..Watcher::default()
        }
    }

    /// Forgets what was seen, so the next look records nothing and starts again.
    ///
    /// Called after the history has moved the program itself. Undoing changes
    /// the state, and a watcher that recorded that would file the undo as a new
    /// deed and never let go of the end of the list.
    pub fn resync(&mut self) {
        self.resync = true;
    }

    /// Ends the frame, letting go of a resync once both looks have had it.
    ///
    /// Cleared here rather than by whichever look ran first, because a resync
    /// has to cover the snapshot *and* the configuration: an undo of a setting
    /// changes both, and one look swallowing the flag would leave the other to
    /// record the undo as a deed.
    pub fn done_looking(&mut self) {
        self.resync = false;
    }

    /// Looks, and answers with what should be recorded.
    ///
    /// Nothing on the first look, and nothing when the answer is the same as
    /// last time, which is almost every frame. The caller decides whether to
    /// look at all: while a gesture is live it does not, so that when the hand
    /// comes off, the comparison is against where the gesture began.
    pub fn look(&mut self, now: &Watched<'_>) -> Option<Vec<Change>> {
        let Some(was) = &self.was else {
            self.was = Some(now.taken());
            self.resync = false;
            return None;
        };

        if now.matches(was) {
            return None;
        }

        let taken = now.taken();
        let changes = was.diff(&taken);
        self.was = Some(taken);

        if self.resync {
            return None;
        }

        (!changes.is_empty()).then_some(changes)
    }

    /// Looks at the configuration, and answers with the change if it moved.
    ///
    /// Walked through the registry rather than compared as a struct, because
    /// `Config` also carries the parsed document it was read from — comparing
    /// *that* every frame would cost more than everything else here together,
    /// and it is not a setting anybody can undo. One pass is a hundred and
    /// eighty comparisons of a scalar or a short string, and allocates nothing
    /// until something has actually moved.
    pub fn look_at_settings(&mut self, now: &Config) -> Option<Change> {
        let Some(was) = &self.settings else {
            self.settings = Some(Box::new(now.clone()));
            return None;
        };

        if !crate::config::registry::rows()
            .iter()
            .filter(|row| !ALSO_IN_THE_SNAPSHOT.contains(&row.path))
            .any(|row| row.access.differs(was, now))
        {
            return None;
        }

        let before = self.settings.replace(Box::new(now.clone()));

        // A resync covers both looks: the history has just written the
        // configuration itself, and filing that as a deed of the user's would
        // put a row in the list nobody asked for.
        if self.resync {
            return None;
        }

        before.map(|before| Change::Settings(before, Box::new(now.clone())))
    }

    /// What the program looked like at the last look.
    pub fn last(&self) -> Option<&Snapshot> {
        self.was.as_ref()
    }
}

/// Settings the snapshot already watches, which must not be counted twice.
///
/// A handful of fields are both a piece of the view and a line in the
/// configuration file: the key that changes them writes them back, which is
/// what makes them survive the next launch. That leaves them visible to *both*
/// looks, and one act — a press of the key that shows the history panel —
/// arrives as two rows, "Opened the history" and "Changed show the history
/// panel", each of which undoes the same thing.
///
/// The snapshot wins, because it is the half that describes what the user did
/// rather than where it happened to be stored, and because it is the half that
/// puts the panel back rather than only the file.
pub const ALSO_IN_THE_SNAPSHOT: &[&str] = &[
    "grid_view.images_per_row",
    "grid_view.filmstrip_visible",
    "tags.advance_after_marking",
    "history.panel_visible",
];

/// Whether a new set of changes is the one before it, continuing.
///
/// Only when every change in both is about the same slot, that slot is one a
/// gesture produces a stream of, and the two arrived close enough together in
/// time. Two slots at once is a deliberate act — a mode change that also moved
/// the cursor — and is never folded into anything.
pub fn continues(
    older: &[Change],
    at: SystemTime,
    newer: &[Change],
    now: SystemTime,
    within: Duration,
) -> bool {
    if within.is_zero() || older.len() != 1 || newer.len() != 1 {
        return false;
    }

    let (Some(older), Some(newer)) = (older.first(), newer.first()) else {
        return false;
    };

    if !newer.is_continuous() || older.slot() != newer.slot() {
        return false;
    }

    now.duration_since(at).is_ok_and(|gap| gap <= within)
}

/// Folds a newer set of changes into an older one, slot by slot.
pub fn fold(older: &mut [Change], newer: &[Change]) {
    for change in older.iter_mut() {
        if let Some(matching) = newer.iter().find(|newer| newer.slot() == change.slot()) {
            change.absorb(matching);
        }
    }
}

/// Which slots a set of changes is about, for a caller deciding what to call it.
pub fn slots(changes: &[Change]) -> Vec<Slot> {
    changes.iter().map(Change::slot).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::mode::Mode;

    fn moment(seconds: u64) -> SystemTime {
        SystemTime::UNIX_EPOCH + Duration::from_secs(seconds)
    }

    const HALF_A_SECOND: Duration = Duration::from_millis(500);

    #[test]
    fn a_walk_that_carries_on_is_one_row() {
        assert!(continues(
            &[Change::Cursor(0, 1)],
            moment(10),
            &[Change::Cursor(1, 2)],
            moment(10),
            HALF_A_SECOND,
        ));
    }

    #[test]
    fn a_walk_taken_up_again_later_is_a_new_row() {
        assert!(!continues(
            &[Change::Cursor(0, 1)],
            moment(10),
            &[Change::Cursor(1, 2)],
            moment(30),
            HALF_A_SECOND,
        ));
    }

    /// A mode change is a decision, not a nudge, and never joins the row in
    /// front of it however fast it followed.
    #[test]
    fn a_decision_is_never_folded_into_a_nudge() {
        assert!(!continues(
            &[Change::Cursor(0, 1)],
            moment(10),
            &[Change::Mode(Mode::Image, Mode::Grid)],
            moment(10),
            HALF_A_SECOND,
        ));

        assert!(!continues(
            &[Change::Mode(Mode::Image, Mode::Grid)],
            moment(10),
            &[Change::Mode(Mode::Grid, Mode::Image)],
            moment(10),
            HALF_A_SECOND,
        ));
    }

    /// Zooming and walking at once is two slots, so it is its own row: folding
    /// it into a walk would lose the zoom.
    #[test]
    fn two_slots_at_once_are_never_folded() {
        assert!(!continues(
            &[Change::Cursor(0, 1)],
            moment(10),
            &[Change::Cursor(1, 2), Change::Columns(4, 5)],
            moment(10),
            HALF_A_SECOND,
        ));
    }

    /// A window of nothing switches folding off altogether, which is what
    /// somebody who wants every notch in the list is asking for.
    #[test]
    fn a_window_of_nothing_folds_nothing() {
        assert!(!continues(
            &[Change::Cursor(0, 1)],
            moment(10),
            &[Change::Cursor(1, 2)],
            moment(10),
            Duration::ZERO,
        ));
    }

    #[test]
    fn folding_keeps_the_beginning() {
        let mut older = vec![Change::Cursor(3, 4)];
        fold(&mut older, &[Change::Cursor(4, 12)]);

        assert!(matches!(older[0], Change::Cursor(3, 12)));
    }

    #[test]
    fn folding_leaves_a_slot_the_newer_set_says_nothing_about() {
        let mut older = vec![Change::Cursor(3, 4), Change::Columns(4, 5)];
        fold(&mut older, &[Change::Cursor(4, 12)]);

        assert!(matches!(older[0], Change::Cursor(3, 12)));
        assert!(matches!(older[1], Change::Columns(4, 5)));
    }

    /// The names have to be paths the registry knows, or the list would stop
    /// excluding anything the day one of them was renamed — silently, and back
    /// to two rows for one press.
    #[test]
    fn everything_excluded_is_a_real_setting() {
        for path in ALSO_IN_THE_SNAPSHOT {
            assert!(
                crate::config::registry::row(path).is_some(),
                "{path} is not a setting"
            );
        }
    }

    /// A key that shows a panel writes the file as well, so both looks see it.
    /// Only one of them may make a row.
    #[test]
    fn a_setting_the_snapshot_watches_makes_no_settings_row() {
        let mut watcher = Watcher::watching(&Config::default());

        let mut now = Config::default();
        now.history.panel_visible = !now.history.panel_visible;
        now.grid_view.filmstrip_visible = !now.grid_view.filmstrip_visible;

        assert!(watcher.look_at_settings(&now).is_none());
    }

    /// And anything else still does.
    #[test]
    fn an_ordinary_setting_still_makes_a_row() {
        let mut watcher = Watcher::watching(&Config::default());

        let mut now = Config::default();
        now.history.remember = 25;

        assert!(watcher.look_at_settings(&now).is_some());
    }

    #[test]
    fn the_slots_of_a_set_are_reported_in_order() {
        let changes = vec![Change::Cursor(0, 1), Change::Columns(4, 5)];

        assert_eq!(slots(&changes), vec![Slot::Cursor, Slot::Columns]);
    }
}
