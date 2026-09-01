//! What was done, in the order it was done, and how to get back to any of it.
//!
//! This began as `organize::journal`: a bounded list of inverse operations
//! covering the things that touch files. Two of its properties were the ones
//! worth keeping — that the reverse is recorded before the operation runs, and
//! that a whole selection marked at once comes back at once — and two were
//! worth losing. It could only go backwards, because it threw the forward half
//! away; and it covered files alone, so everything else the user did left no
//! trace at all.
//!
//! # Watching rather than being told
//!
//! Nothing here is called from the places that carry out commands. There are
//! five separate dispatchers in this program — [`crate::app`]'s `Command`,
//! `Verb`, `MenuAction`, `BarAction` and the image view's own — and the filter
//! bar changes what is shown without going through any of them. Recording at
//! each of the six would be six copies of one rule, and a seventh route added
//! later would silently record nothing at all.
//!
//! So the history watches the *state* instead, once at the foot of the frame,
//! and compares it with the state at the foot of the last one. Every route in
//! is covered by construction, including the ones that do not exist yet. What
//! that costs per frame is a comparison against a struct of scalars and two
//! short collections, with no allocation; a clone happens only on the frames
//! where something actually changed.
//!
//! # A tree, because nothing is overwritten
//!
//! Going back and then doing something different keeps both: the branch just
//! left stays whole and stays reachable. [`tree`] holds that shape and is
//! tested on its own.

use std::time::{Duration, SystemTime};

pub mod deed;
pub mod files;
pub mod snapshot;
pub mod tree;
pub mod watch;

pub use deed::{Class, Deed};
pub use files::{Done, Step, Way};
pub use snapshot::{Change, Panels, Slot, Snapshot, Watched};
pub use tree::{Node, NodeId, Tree};
pub use watch::Watcher;

/// One row of the history: what was done, and when.
#[derive(Debug, Clone)]
pub struct Entry {
    /// What was done.
    pub deed: Deed,
    /// When it was done, for the row that stands for it.
    pub at: SystemTime,
    /// What to call it, worked out once when it happened.
    ///
    /// Kept rather than derived because the panel draws every row it can fit
    /// on every frame it is open, and a `String` built per row per frame is an
    /// allocation per row per frame.
    pub label: String,
}

impl Entry {
    /// A row for a deed done now.
    pub fn new(deed: Deed) -> Entry {
        Entry {
            label: deed.label(),
            at: SystemTime::now(),
            deed,
        }
    }
}

/// Everything that has been done this run, and where in it we are.
#[derive(Debug)]
pub struct History {
    tree: Tree<Entry>,
    /// How many deeds to keep. Nought is all of them, which is the default:
    /// the list is made of paths and small documents, not of photographs.
    remember: usize,
}

impl Default for History {
    fn default() -> History {
        History::new()
    }
}

impl History {
    /// An empty history, sitting at the beginning.
    pub fn new() -> History {
        History {
            tree: Tree::new(Entry::new(Deed::Start)),
            remember: 0,
        }
    }

    /// An empty history that will keep at most this many deeds.
    pub fn with_limit(remember: usize) -> History {
        History {
            remember,
            ..History::new()
        }
    }

    /// How many deeds are kept; nought for all of them.
    pub fn set_remember(&mut self, remember: usize) {
        self.remember = remember;
        self.tree.trim(remember);
    }

    /// The shape, for the panel to draw.
    pub fn tree(&self) -> &Tree<Entry> {
        &self.tree
    }

    /// Where we are.
    pub fn cursor(&self) -> NodeId {
        self.tree.cursor()
    }

    /// The beginning, which is never run.
    pub fn root(&self) -> NodeId {
        self.tree.root()
    }

    /// One row, if it is still there.
    pub fn entry(&self, id: NodeId) -> Option<&Entry> {
        self.tree.value(id)
    }

    /// How many deeds are in it, not counting the beginning.
    pub fn len(&self) -> usize {
        self.tree.len()
    }

    /// Whether nothing has been done yet.
    pub fn is_empty(&self) -> bool {
        self.tree.is_empty()
    }

    /// Files a deed as having just happened, under wherever the cursor is.
    ///
    /// Returns where it went, or nothing when the deed did nothing worth
    /// recording — an operation that touched no file leaves no row.
    pub fn record(&mut self, deed: Deed) -> Option<NodeId> {
        if deed.is_empty() {
            return None;
        }

        let id = self.tree.push(Entry::new(deed));
        self.tree.trim(self.remember);

        // A trim never drops the cursor, so the node just pushed is still
        // there; saying so keeps the caller from having to check.
        self.tree.get(id).is_some().then_some(id)
    }

    /// Files what a look at the program found, folding a gesture into one row.
    ///
    /// A nudge that carries on from the nudge before it — the wheel turned
    /// twice, an arrow held down — becomes the same row rather than a new one,
    /// keeping where the gesture began. Anything else is its own row.
    pub fn note(&mut self, changes: Vec<Change>, within: Duration) -> Option<NodeId> {
        if changes.is_empty() {
            return None;
        }

        let now = SystemTime::now();
        let at = self.tree.cursor();

        if at != self.tree.root() {
            let carries_on = self
                .tree
                .value(at)
                .and_then(|entry| match &entry.deed {
                    Deed::Changed(older) => Some((older.clone(), entry.at)),
                    _ => None,
                })
                .is_some_and(|(older, was)| watch::continues(&older, was, &changes, now, within));

            if carries_on {
                if let Some(entry) = self.tree.value_mut(at) {
                    if let Deed::Changed(older) = &mut entry.deed {
                        watch::fold(older, &changes);
                    }
                    entry.label = entry.deed.label();
                    entry.at = now;
                }

                return Some(at);
            }
        }

        self.record(Deed::Changed(changes))
    }

    /// Moves to a node without running anything, once the running is done.
    pub fn arrive(&mut self, id: NodeId) {
        self.tree.set_cursor(id);
    }

    /// What one press of undo would run, nearest first.
    ///
    /// It keeps walking back until it has taken back something of a class that
    /// is switched on. That is deliberately not the same as stepping over the
    /// others without running them: everything between here and there did
    /// happen, and leaving some of it applied while the cursor sits below it
    /// would make the history describe a state the program is not in. What the
    /// setting buys is that `Ctrl + Z` does not *stop* on a wheel notch — one
    /// press still lands on the rating, and the zoom goes back with it.
    pub fn plan_undo(&self, enabled: impl Fn(Class) -> bool) -> Vec<(NodeId, Way)> {
        let mut route = Vec::new();
        let mut at = self.tree.cursor();

        while at != self.tree.root() {
            let Some(node) = self.tree.get(at) else {
                break;
            };

            route.push((at, Way::Back));

            if enabled(node.value.deed.class()) {
                return route;
            }

            let Some(parent) = node.parent else {
                break;
            };

            at = parent;
        }

        // Nothing of a class worth stopping on: better to do nothing than to
        // rewind the whole run for a press that was meant to take one thing
        // back.
        Vec::new()
    }

    /// What one press of redo would run, in the order it would run it.
    pub fn plan_redo(&self, enabled: impl Fn(Class) -> bool) -> Vec<(NodeId, Way)> {
        let mut route = Vec::new();
        let mut at = self.tree.cursor();

        while let Some(next) = self.next_after(at) {
            let Some(node) = self.tree.get(next) else {
                break;
            };

            route.push((next, Way::Forward));

            if enabled(node.value.deed.class()) {
                return route;
            }

            at = next;
        }

        Vec::new()
    }

    /// The child a redo from this node would take.
    fn next_after(&self, id: NodeId) -> Option<NodeId> {
        let node = self.tree.get(id)?;

        node.preferred
            .filter(|child| self.tree.get(*child).is_some())
            .or_else(|| node.children.last().copied())
    }

    /// What getting to a chosen row would run.
    ///
    /// A click names the row it means, so no class is stepped over here: the
    /// answer to "take me back to this one" is that one and everything between.
    pub fn plan_go_to(&self, id: NodeId) -> Vec<(NodeId, Way)> {
        self.tree.route(id)
    }

    /// Where the cursor ends up after running a route.
    ///
    /// Going back lands on the parent of the last thing undone; going forward
    /// lands on the last thing done.
    pub fn landing(&self, route: &[(NodeId, Way)]) -> Option<NodeId> {
        let (last, way) = route.last().copied()?;

        match way {
            Way::Back => self.tree.get(last).and_then(|node| node.parent),
            Way::Forward => Some(last),
        }
    }

    /// Forgets everything and starts again from the beginning.
    pub fn clear(&mut self) {
        self.tree = Tree::new(Entry::new(Deed::Start));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    /// Everything is on, which is the default.
    fn all(_: Class) -> bool {
        true
    }

    /// A deed that touches files, so it is not empty and is recorded.
    fn binned(name: &str) -> Deed {
        Deed::Files(Step::Binned(vec![PathBuf::from(name)]))
    }

    /// A walk of the folder, which is the continuous kind.
    fn walked(from: usize, to: usize) -> Vec<Change> {
        vec![Change::Cursor(from, to)]
    }

    const HALF_A_SECOND: Duration = Duration::from_millis(500);

    #[test]
    fn a_new_history_has_nothing_to_undo() {
        let history = History::new();

        assert!(history.is_empty());
        assert!(history.plan_undo(all).is_empty());
        assert!(history.plan_redo(all).is_empty());
    }

    /// An operation that touched no file leaves no row, so the list is of
    /// things that happened rather than of keys that were pressed.
    #[test]
    fn a_deed_that_did_nothing_is_not_recorded() {
        let mut history = History::new();

        assert_eq!(history.record(Deed::Files(Step::Binned(vec![]))), None);
        assert!(history.is_empty());
    }

    #[test]
    fn undo_then_redo_comes_back_to_where_it_was() {
        let mut history = History::new();
        let a = history.record(binned("a.jpg")).unwrap();

        let route = history.plan_undo(all);
        assert_eq!(route, vec![(a, Way::Back)]);

        history.arrive(history.landing(&route).unwrap());
        assert_eq!(history.cursor(), history.root());

        let route = history.plan_redo(all);
        assert_eq!(route, vec![(a, Way::Forward)]);
        history.arrive(history.landing(&route).unwrap());
        assert_eq!(history.cursor(), a);
    }

    /// The promise that nothing is overwritten: having gone back, doing
    /// something else keeps what was there.
    #[test]
    fn going_back_and_doing_something_else_keeps_both() {
        let mut history = History::new();
        let a = history.record(binned("a.jpg")).unwrap();
        history.arrive(history.root());
        let b = history.record(binned("b.jpg")).unwrap();

        assert_eq!(history.len(), 2);
        assert!(history.entry(a).is_some(), "the first is still there");
        assert!(history.entry(b).is_some());
    }

    /// A press that can reach nothing of a class worth stopping on does
    /// nothing, rather than rewinding the whole run looking for one.
    #[test]
    fn undo_with_every_class_switched_off_does_nothing() {
        let mut history = History::new();
        history.record(binned("a.jpg"));
        history.record(binned("b.jpg"));

        assert!(history.plan_undo(|_| false).is_empty());
        assert!(history.plan_redo(|_| false).is_empty());
    }

    /// With the class on, one press takes back one deed and lands on the one
    /// before it.
    #[test]
    fn undo_stops_on_the_first_deed_of_a_class_that_is_on() {
        let mut history = History::new();
        let first = history.record(binned("a.jpg")).unwrap();
        let second = history.record(binned("b.jpg")).unwrap();

        let route = history.plan_undo(all);

        assert_eq!(route, vec![(second, Way::Back)]);
        assert_eq!(history.landing(&route), Some(first));
    }

    #[test]
    fn a_route_to_a_row_lands_on_that_row() {
        let mut history = History::new();
        let a = history.record(binned("a.jpg")).unwrap();
        let b = history.record(binned("b.jpg")).unwrap();
        let c = history.record(binned("c.jpg")).unwrap();

        let route = history.plan_go_to(a);
        assert_eq!(route, vec![(c, Way::Back), (b, Way::Back)]);
        assert_eq!(history.landing(&route), Some(a));
    }

    #[test]
    fn a_limit_forgets_the_oldest_and_nought_forgets_nothing() {
        let mut history = History::new();
        history.set_remember(2);

        let a = history.record(binned("a.jpg")).unwrap();
        history.record(binned("b.jpg")).unwrap();
        history.record(binned("c.jpg")).unwrap();

        assert_eq!(history.len(), 2);
        assert!(history.entry(a).is_none());

        let mut history = History::new();
        for i in 0..300 {
            history.record(binned(&format!("{i}.jpg")));
        }
        assert_eq!(history.len(), 300, "nought is all of them");
    }

    /// Lowering the limit takes effect at once rather than at the next deed.
    #[test]
    fn shortening_the_limit_trims_what_is_already_there() {
        let mut history = History::new();
        for i in 0..10 {
            history.record(binned(&format!("{i}.jpg")));
        }

        history.set_remember(3);

        assert_eq!(history.len(), 3);
    }

    #[test]
    fn a_row_remembers_what_it_was_called_when_it_happened() {
        let mut history = History::new();
        let a = history.record(binned("a.jpg")).unwrap();

        assert_eq!(history.entry(a).unwrap().label, "Sent 1 file(s) to the bin");
    }

    /// A wheel turned twice is one row, and undoing it goes back to where the
    /// gesture began rather than to the middle of it.
    #[test]
    fn a_walk_carried_on_is_one_row_that_remembers_its_beginning() {
        let mut history = History::new();

        let first = history.note(walked(0, 1), HALF_A_SECOND).unwrap();
        let again = history.note(walked(1, 2), HALF_A_SECOND).unwrap();
        let more = history.note(walked(2, 9), HALF_A_SECOND).unwrap();

        assert_eq!(first, again);
        assert_eq!(first, more);
        assert_eq!(history.len(), 1, "one gesture, one row");

        match &history.entry(first).unwrap().deed {
            Deed::Changed(changes) => match changes.as_slice() {
                [Change::Cursor(from, to)] => assert_eq!((*from, *to), (0, 9)),
                other => panic!("{other:?}"),
            },
            other => panic!("{other:?}"),
        }
    }

    /// Switching the folding off lists every notch, which is what somebody who
    /// sets the window to nought is asking for.
    #[test]
    fn a_window_of_nothing_gives_every_notch_its_own_row() {
        let mut history = History::new();

        history.note(walked(0, 1), Duration::ZERO);
        history.note(walked(1, 2), Duration::ZERO);

        assert_eq!(history.len(), 2);
    }

    /// A decision is never folded into the nudge in front of it, however fast
    /// it followed: they are different slots.
    #[test]
    fn a_different_slot_starts_a_new_row() {
        let mut history = History::new();

        let walk = history.note(walked(0, 1), HALF_A_SECOND).unwrap();
        let columns = history
            .note(vec![Change::Columns(4, 6)], HALF_A_SECOND)
            .unwrap();

        assert_ne!(walk, columns);
        assert_eq!(history.len(), 2);
    }

    /// The whole point of the classes: one press lands on the rating rather
    /// than on the twenty photographs walked past since.
    #[test]
    fn undo_with_the_view_switched_off_walks_past_it_to_the_rating() {
        let mut history = History::new();
        let rating = history.record(binned("a.jpg")).unwrap();
        let walk = history.note(walked(0, 1), Duration::ZERO).unwrap();
        let more = history.note(walked(1, 2), Duration::ZERO).unwrap();

        let route = history.plan_undo(|class| class != Class::View);

        assert_eq!(
            route,
            vec![(more, Way::Back), (walk, Way::Back), (rating, Way::Back)],
            "everything between is run; only where it stops is different"
        );
        assert_eq!(
            history.landing(&route),
            Some(history.root()),
            "and it lands before the rating, which is what taking it back means"
        );

        // With the view switched on, the same press stops at the first notch.
        let route = history.plan_undo(|_| true);
        assert_eq!(route, vec![(more, Way::Back)]);
    }

    /// A row carrying a settings change is a settings row, so switching that
    /// class off steps over it.
    #[test]
    fn a_row_is_classed_by_what_it_carries() {
        let mut history = History::new();

        let view = history.note(walked(0, 1), Duration::ZERO).unwrap();
        assert_eq!(history.entry(view).unwrap().deed.class(), Class::View);

        let settings = history
            .note(
                vec![Change::Settings(Box::default(), Box::default())],
                Duration::ZERO,
            )
            .unwrap();
        assert_eq!(
            history.entry(settings).unwrap().deed.class(),
            Class::Settings
        );
    }

    #[test]
    fn a_look_that_found_nothing_is_not_a_row() {
        let mut history = History::new();

        assert_eq!(history.note(Vec::new(), HALF_A_SECOND), None);
        assert!(history.is_empty());
    }

    #[test]
    fn clearing_goes_back_to_the_beginning() {
        let mut history = History::new();
        history.record(binned("a.jpg"));

        history.clear();

        assert!(history.is_empty());
        assert_eq!(history.cursor(), history.root());
    }
}
