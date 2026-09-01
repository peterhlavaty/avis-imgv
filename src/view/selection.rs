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
        let (start, end) = if from <= position {
            (from, position)
        } else {
            (position, from)
        };

        for step in start..=end {
            if let Some(index) = visible.at(step) {
                self.chosen.insert(index);
            }
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

    #[test]
    fn clearing_forgets_the_anchor_too() {
        let mut selection = Selection::default();
        selection.toggle(2, 2);
        selection.clear();

        assert!(selection.is_empty());
        assert_eq!(selection.anchor(), None);
    }
}
