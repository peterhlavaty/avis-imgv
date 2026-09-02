//! Which photographs a command is about, when it is about more than one.
//!
//! Marking one frame at a time is what a viewer does; marking two hundred in
//! one keystroke is what a culling tool does, and the difference between the
//! two is a set. Every other program in the comparison has one — it is the
//! single most conspicuous thing this viewer was missing — and having it turns
//! "rate this" into "rate these" everywhere without any command having to know
//! that it happened.
//!
//! The set holds *store positions*, not paths and not positions in what is on
//! show. A path would go stale the moment a photograph were renamed; a
//! position in the shown order would go stale the moment a filter changed. A
//! store position is what the caches are already keyed by, so a selection made
//! before a filter is applied is still the same selection afterwards, and
//! narrowing a folder down does not silently throw away the frames somebody
//! spent a minute picking out.

use std::collections::BTreeSet;

use super::visible::Visible;

/// The photographs that have been picked out, as store positions.
///
/// Compared as well as cloned, because the history looks at it once a frame
/// and has to know whether it moved without allocating to find out.
#[derive(serde::Deserialize, serde::Serialize, Debug, Default, Clone, PartialEq, Eq)]
pub struct Selection {
    chosen: BTreeSet<usize>,
    /// Where the last run started, as a position in what is on show.
    ///
    /// Shift-extending is a *run from somewhere*, so it needs to remember
    /// where: extending twice in a row from the same anchor grows and shrinks
    /// one run rather than leaving a trail of separate ones behind.
    anchor: Option<usize>,
}

impl Selection {
    pub fn is_empty(&self) -> bool {
        self.chosen.is_empty()
    }

    pub fn len(&self) -> usize {
        self.chosen.len()
    }

    pub fn contains(&self, index: usize) -> bool {
        self.chosen.contains(&index)
    }

    /// The picked photographs, in the collection's own order.
    pub fn iter(&self) -> impl Iterator<Item = usize> + '_ {
        self.chosen.iter().copied()
    }

    pub fn clear(&mut self) {
        self.chosen.clear();
        self.anchor = None;
    }

    /// Picks a photograph out, or puts it back.
    ///
    /// Toggling also drops the anchor here, because the next run should start
    /// from what was just touched rather than from wherever the last one did.
    pub fn toggle(&mut self, index: usize, position: usize) {
        if !self.chosen.remove(&index) {
            self.chosen.insert(index);
        }

        self.anchor = Some(position);
    }

    /// Picks one photograph out and puts everything else back.
    ///
    /// What a plain click means everywhere: the click that used to leave the
    /// contact sheet altogether.
    pub fn only(&mut self, index: usize, position: usize) {
        self.chosen.clear();
        self.chosen.insert(index);
        self.anchor = Some(position);
    }

    /// Picks a photograph out, leaving whatever else is picked alone.
    ///
    /// For the rubber band, which adds what it crosses to what it started
    /// from rather than deciding one cell at a time.
    pub fn add(&mut self, index: usize) {
        self.chosen.insert(index);
    }

    /// Everything on show, or nothing if that is already what is picked.
    ///
    /// One key doing both is how every file manager behaves, and it saves the
    /// second key nobody would remember.
    pub fn all_or_none(&mut self, visible: &Visible) {
        if self.chosen.len() == visible.len() && !visible.is_empty() {
            self.clear();
            return;
        }

        self.chosen = visible.iter().collect();
        self.anchor = Some(0);
    }

    /// Picks out the run between where the last one started and `position`.
    ///
    /// The anchor stays put, so holding shift and walking back up the sheet
    /// shrinks the run rather than selecting the way back as well. What was
    /// picked before the run began is left alone: shift-extending adds, it
    /// does not replace.
    pub fn extend_to(&mut self, visible: &Visible, position: usize) {
        let from = *self.anchor.get_or_insert(position);
        self.pick_the_run(visible, from, position);
    }

    /// Picks out the run between `position` and the nearest already-picked
    /// frame, keeping everything that was picked before.
    ///
    /// The strip's shift-click rather than the sheet's. The sheet extends from
    /// an anchor, which is what a list of files does and what somebody
    /// building one run at a time wants; the strip is looked at while the
    /// photograph is on screen, where the run being asked for is nearly always
    /// the gap between what is already picked and the frame just pointed at —
    /// and after two or three runs the anchor is somewhere the person has
    /// forgotten. Both are defensible, so both are here rather than one being
    /// bent into the other.
    ///
    /// Nearest is measured in the shown order and never wraps: a run from the
    /// last frame of a folder to the first is not a run anybody meant. At an
    /// equal distance either side the earlier one wins, because a tie has to
    /// be settled somewhere and going back is the half a walk has already
    /// seen.
    pub fn extend_from_nearest(&mut self, visible: &Visible, position: usize) {
        let from = self.nearest_picked(visible, position);
        self.anchor = Some(position);

        // Nothing picked out at all: the run is the one frame clicked.
        let from = from.unwrap_or(position);
        self.pick_the_run(visible, from, position);
    }

    /// Picks out every frame between two positions in the shown order, ends
    /// included, leaving whatever else is picked alone.
    ///
    /// Positions off the end are stepped over rather than refused: a strip and
    /// a filter can disagree for one frame.
    fn pick_the_run(&mut self, visible: &Visible, from: usize, to: usize) {
        let (start, end) = if from <= to { (from, to) } else { (to, from) };

        for step in start..=end {
            if let Some(index) = visible.at(step) {
                self.chosen.insert(index);
            }
        }
    }

    /// The picked-out frame closest to `position` in the shown order.
    ///
    /// Walked outwards rather than swept, so the answer costs the distance to
    /// it rather than the size of the folder in the case that matters — the
    /// run being asked for is usually a short one.
    fn nearest_picked(&self, visible: &Visible, position: usize) -> Option<usize> {
        let shown = visible.len();
        if position >= shown {
            return None;
        }

        let holds = |at: usize| {
            visible
                .at(at)
                .is_some_and(|index| self.chosen.contains(&index))
        };

        (0..shown).find_map(|step| {
            let left = position.checked_sub(step).filter(|&left| holds(left));
            let right =
                (position + step < shown && holds(position + step)).then(|| position + step);

            left.or(right)
        })
    }

    /// Puts the photograph on screen into an empty set, before something else
    /// joins it.
    ///
    /// The set is either empty — which every command already reads as "the one
    /// being looked at" — or it holds the photograph on screen along with the
    /// rest of them. Without this, picking out a second frame would quietly
    /// drop the first, and a command meant for two would run on one.
    pub fn start_at(&mut self, index: usize, position: usize) {
        if self.chosen.is_empty() {
            self.chosen.insert(index);
            self.anchor = Some(position);
        }
    }

    /// Puts the set down once it has come back to being the photograph on
    /// screen and nothing else.
    ///
    /// That is the state it started in, and leaving it as a set of one would
    /// draw a picked-out colour round the frame somebody has just finished
    /// unpicking.
    pub fn settle_on(&mut self, index: usize) {
        if self.chosen.len() == 1 && self.chosen.contains(&index) {
            self.clear();
        }
    }

    /// Where a shift-extend would start from.
    pub fn anchor(&self) -> Option<usize> {
        self.anchor
    }

    /// Remembers where the next run starts, without picking anything out.
    pub fn anchor_at(&mut self, position: usize) {
        self.anchor = Some(position);
    }

    /// Drops a photograph that has left the collection, and closes the gap.
    ///
    /// The store positions above it all move down by one when a path is taken
    /// out of the list, so a selection that did not move with them would come
    /// to mean the photographs next door.
    pub fn remove_shifting(&mut self, index: usize) {
        self.chosen = self
            .chosen
            .iter()
            .filter(|&&chosen| chosen != index)
            .map(|&chosen| if chosen > index { chosen - 1 } else { chosen })
            .collect();
    }

    /// Makes room for a photograph that has appeared at `index`.
    ///
    /// The new one is not picked out — nobody asked for it — but everything
    /// from `index` up has moved along one, and a set that did not move with
    /// them would come to mean the photographs next door.
    pub fn insert_shifting(&mut self, index: usize) {
        self.chosen = self
            .chosen
            .iter()
            .map(|&chosen| if chosen >= index { chosen + 1 } else { chosen })
            .collect();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A plain click in the contact sheet: this one, and nothing else.
    #[test]
    fn picking_one_out_puts_the_others_back() {
        let mut selection = Selection::default();
        selection.add(3);
        selection.add(7);

        selection.only(5, 5);

        assert_eq!(selection.len(), 1);
        assert!(selection.contains(5));
        assert_eq!(selection.anchor(), Some(5), "and a run starts from here");
    }

    /// The rubber band adds to what was already picked, so dragging a second
    /// run does not throw away the first.
    #[test]
    fn adding_leaves_what_was_there() {
        let mut selection = Selection::default();
        selection.add(1);
        selection.add(2);
        selection.add(2);

        assert_eq!(selection.len(), 2);
        assert!(selection.contains(1) && selection.contains(2));
    }

    fn everything(total: usize) -> Visible {
        Visible::everything(total)
    }

    #[test]
    fn nothing_is_picked_to_begin_with() {
        let selection = Selection::default();

        assert!(selection.is_empty());
        assert_eq!(selection.len(), 0);
        assert!(!selection.contains(0));
    }

    #[test]
    fn toggling_picks_out_and_puts_back() {
        let mut selection = Selection::default();

        selection.toggle(3, 3);
        assert!(selection.contains(3));

        selection.toggle(3, 3);
        assert!(!selection.contains(3));
    }

    #[test]
    fn the_set_is_read_back_in_the_collections_order() {
        let mut selection = Selection::default();
        selection.toggle(7, 7);
        selection.toggle(2, 2);
        selection.toggle(5, 5);

        assert_eq!(selection.iter().collect::<Vec<_>>(), vec![2, 5, 7]);
    }

    #[test]
    fn a_run_is_picked_out_between_the_anchor_and_the_cursor() {
        let visible = everything(10);
        let mut selection = Selection::default();

        selection.toggle(2, 2);
        selection.extend_to(&visible, 5);

        assert_eq!(selection.iter().collect::<Vec<_>>(), vec![2, 3, 4, 5]);
    }

    /// Walking back up the sheet with shift held shrinks the run; it does not
    /// select the way back as a second run.
    #[test]
    fn extending_backwards_runs_from_the_same_anchor() {
        let visible = everything(10);
        let mut selection = Selection::default();

        selection.toggle(5, 5);
        selection.extend_to(&visible, 8);
        selection.extend_to(&visible, 3);

        // Everything either side of the anchor that was walked over.
        assert_eq!(selection.iter().collect::<Vec<_>>(), vec![3, 4, 5, 6, 7, 8]);
        assert_eq!(selection.anchor(), Some(5));
    }

    /// The run is over what is on show, so a filtered sheet extends over the
    /// photographs that are actually next to each other on it.
    #[test]
    fn a_run_follows_the_shown_order_rather_than_the_collections() {
        let visible = Visible::of(vec![9, 4, 1, 6], 10);
        let mut selection = Selection::default();

        selection.anchor_at(1);
        selection.extend_to(&visible, 3);

        assert_eq!(selection.iter().collect::<Vec<_>>(), vec![1, 4, 6]);
    }

    #[test]
    fn one_key_selects_everything_and_then_nothing() {
        let visible = everything(4);
        let mut selection = Selection::default();

        selection.all_or_none(&visible);
        assert_eq!(selection.len(), 4);

        selection.all_or_none(&visible);
        assert!(selection.is_empty());
    }

    /// Selecting everything on a narrowed sheet picks out what is on show and
    /// not the whole folder, which is the reason to narrow it first.
    #[test]
    fn selecting_everything_means_everything_shown() {
        let visible = Visible::of(vec![1, 3], 10);
        let mut selection = Selection::default();

        selection.all_or_none(&visible);

        assert_eq!(selection.iter().collect::<Vec<_>>(), vec![1, 3]);
    }

    /// The case that would quietly mark the wrong photographs: a frame is
    /// deleted, everything above it shifts down, and the set has to shift too.
    #[test]
    fn a_photograph_leaving_takes_its_place_with_it() {
        let mut selection = Selection::default();
        for index in [1, 4, 7] {
            selection.toggle(index, index);
        }

        selection.remove_shifting(4);

        assert_eq!(selection.iter().collect::<Vec<_>>(), vec![1, 6]);
    }

    #[test]
    fn a_photograph_appearing_moves_the_set_along() {
        let mut selection = Selection::default();
        for index in [0, 3, 7] {
            selection.toggle(index, index);
        }

        selection.insert_shifting(3);

        // The one that was at 3 is now at 4; the new arrival is not picked.
        assert_eq!(selection.iter().collect::<Vec<_>>(), vec![0, 4, 8]);
    }

    #[test]
    fn a_photograph_appearing_at_the_end_moves_nothing() {
        let mut selection = Selection::default();
        selection.toggle(1, 1);
        selection.insert_shifting(9);

        assert_eq!(selection.iter().collect::<Vec<_>>(), vec![1]);
    }

    #[test]
    fn removing_below_the_set_still_shifts_it() {
        let mut selection = Selection::default();
        selection.toggle(5, 5);
        selection.remove_shifting(0);

        assert_eq!(selection.iter().collect::<Vec<_>>(), vec![4]);
    }

    /// The strip's shift-click with only the photograph on screen picked out:
    /// the run is from it to what was clicked, which is the ordinary case.
    #[test]
    fn a_run_reaches_back_to_the_only_frame_picked_out() {
        let visible = everything(20);
        let mut selection = Selection::default();
        selection.start_at(5, 5);

        selection.extend_from_nearest(&visible, 10);

        assert_eq!(
            selection.iter().collect::<Vec<_>>(),
            vec![5, 6, 7, 8, 9, 10]
        );
    }

    /// And the case the anchor gets wrong: a second run, asked for from the
    /// end of the first. Everything picked before is kept.
    #[test]
    fn a_second_run_starts_from_the_end_of_the_first() {
        let visible = everything(30);
        let mut selection = Selection::default();
        selection.start_at(5, 5);
        selection.extend_from_nearest(&visible, 10);

        selection.extend_from_nearest(&visible, 14);

        assert_eq!(
            selection.iter().collect::<Vec<_>>(),
            (5..=14).collect::<Vec<_>>()
        );
    }

    /// Nearest, with gaps: 15 is five away and 3 is seven, so the run goes up.
    #[test]
    fn a_run_reaches_the_nearest_of_several_picked_out() {
        let visible = everything(30);
        let mut selection = Selection::default();
        selection.add(3);
        selection.add(15);

        selection.extend_from_nearest(&visible, 10);

        assert_eq!(
            selection.iter().collect::<Vec<_>>(),
            vec![3, 10, 11, 12, 13, 14, 15]
        );
    }

    /// Backwards is a run like any other.
    #[test]
    fn a_run_reaches_backwards_when_that_is_the_nearer_side() {
        let visible = everything(30);
        let mut selection = Selection::default();
        selection.add(20);

        selection.extend_from_nearest(&visible, 17);

        assert_eq!(selection.iter().collect::<Vec<_>>(), vec![17, 18, 19, 20]);
    }

    /// The end of the strip is the end of it. A run from the last frame to the
    /// first would pick out the whole folder, which nobody asked for.
    #[test]
    fn a_run_never_wraps_round_the_end() {
        let visible = everything(30);
        let mut selection = Selection::default();
        selection.add(1);

        selection.extend_from_nearest(&visible, 28);

        assert_eq!(
            selection.iter().collect::<Vec<_>>(),
            (1..=28).collect::<Vec<_>>(),
            "the run reaches back to 1 rather than forward over the end"
        );
    }

    /// A tie has to be settled somewhere: the earlier frame wins.
    #[test]
    fn a_run_equally_far_either_way_goes_back() {
        let visible = everything(30);
        let mut selection = Selection::default();
        selection.add(5);
        selection.add(15);

        selection.extend_from_nearest(&visible, 10);

        assert_eq!(
            selection.iter().collect::<Vec<_>>(),
            vec![5, 6, 7, 8, 9, 10, 15]
        );
    }

    /// The run follows what is on show, so a filtered strip runs over the
    /// frames that are actually next to each other on it.
    #[test]
    fn a_run_follows_the_shown_order() {
        let visible = Visible::of(vec![9, 4, 1, 6, 2], 10);
        let mut selection = Selection::default();
        selection.add(9);

        selection.extend_from_nearest(&visible, 3);

        assert_eq!(selection.iter().collect::<Vec<_>>(), vec![1, 4, 6, 9]);
    }

    /// Nothing picked out at all: the run is the one frame that was clicked.
    #[test]
    fn a_run_from_nothing_picks_out_the_one_frame() {
        let visible = everything(10);
        let mut selection = Selection::default();

        selection.extend_from_nearest(&visible, 4);

        assert_eq!(selection.iter().collect::<Vec<_>>(), vec![4]);
    }

    /// A position off the end of what is on show changes nothing rather than
    /// panicking: a strip and a filter can disagree for one frame.
    #[test]
    fn a_run_past_the_end_of_the_strip_picks_out_nothing() {
        let visible = everything(4);
        let mut selection = Selection::default();
        selection.add(1);

        selection.extend_from_nearest(&visible, 9);

        assert_eq!(selection.iter().collect::<Vec<_>>(), vec![1]);
    }

    /// The invariant every command depends on: an empty set means "the one
    /// being looked at", so the second frame picked out has to bring the first
    /// with it.
    #[test]
    fn picking_out_a_second_frame_keeps_the_one_on_screen() {
        let mut selection = Selection::default();

        selection.start_at(5, 5);
        selection.toggle(9, 9);

        assert_eq!(selection.iter().collect::<Vec<_>>(), vec![5, 9]);
    }

    /// And it only ever seeds an empty set: a set that has already been built
    /// is not quietly given the photograph on screen.
    #[test]
    fn a_set_that_already_holds_something_is_left_alone() {
        let mut selection = Selection::default();
        selection.add(2);

        selection.start_at(5, 5);

        assert_eq!(selection.iter().collect::<Vec<_>>(), vec![2]);
    }

    /// Unpicking the way back to one frame comes back to no set at all, which
    /// is where it started.
    #[test]
    fn coming_back_to_the_frame_on_screen_puts_the_set_down() {
        let mut selection = Selection::default();
        selection.start_at(5, 5);
        selection.toggle(9, 9);

        selection.toggle(9, 9);
        selection.settle_on(5);

        assert!(selection.is_empty());
        assert_eq!(selection.anchor(), None);
    }

    /// A set of one that is *not* the photograph on screen is a real set and
    /// stays.
    #[test]
    fn a_set_of_one_somewhere_else_is_left_standing() {
        let mut selection = Selection::default();
        selection.add(9);

        selection.settle_on(5);

        assert_eq!(selection.iter().collect::<Vec<_>>(), vec![9]);
    }

    #[test]
    fn clearing_forgets_the_anchor_too() {
        let mut selection = Selection::default();
        selection.toggle(2, 2);
        selection.clear();

        assert!(selection.is_empty());
        assert_eq!(selection.anchor(), None);
    }
}
