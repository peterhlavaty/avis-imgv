//! Which of the open photographs are being shown, and in what order.
//!
//! A filter must not throw away what has been decoded. The caches are keyed by
//! position in the store, so hiding a photograph by giving the store a shorter
//! list would move every position after it and discard the folder — which is
//! exactly what a filter applied while marking would do on every keystroke.
//!
//! So the store keeps everything and this says what to show: a list of store
//! positions, in the order the views walk them. Filtering and sorting a folder
//! of two thousand is then a vector of two thousand `usize`, rebuilt in under
//! a millisecond, and nothing is decoded twice.

/// The store positions on show, in order.
#[derive(Debug, Clone, Default)]
pub struct Visible {
    order: Vec<usize>,
    /// Whether `order` is simply every position in turn, which is the case
    /// whenever nothing is filtered or sorted and is worth knowing because it
    /// makes every lookup arithmetic instead of a search.
    everything: bool,
}

impl Visible {
    /// Every photograph, in the order the store holds them.
    pub fn everything(total: usize) -> Visible {
        Visible {
            order: (0..total).collect(),
            everything: true,
        }
    }

    /// A chosen set, in a chosen order.
    pub fn of(order: Vec<usize>, total: usize) -> Visible {
        let everything = order.len() == total && order.iter().enumerate().all(|(i, at)| i == *at);

        Visible { order, everything }
    }

    pub fn len(&self) -> usize {
        self.order.len()
    }

    pub fn is_empty(&self) -> bool {
        self.order.is_empty()
    }

    /// Whether anything is being held back.
    pub fn is_everything(&self) -> bool {
        self.everything
    }

    /// The store position shown at `position`.
    pub fn at(&self, position: usize) -> Option<usize> {
        self.order.get(position).copied()
    }

    /// Where a store position appears, if it appears at all.
    pub fn position_of(&self, index: usize) -> Option<usize> {
        if self.everything {
            return (index < self.order.len()).then_some(index);
        }

        self.order.iter().position(|at| *at == index)
    }

    /// Where a store position appears, or the nearest thing to it that does.
    ///
    /// What a filter needs the moment it hides the photograph on screen: the
    /// answer must be somewhere sensible rather than back at the beginning,
    /// because rejecting a frame with "hide rejected" on should leave the
    /// cursor looking at its neighbour.
    pub fn nearest(&self, index: usize) -> Option<usize> {
        if let Some(exact) = self.position_of(index) {
            return Some(exact);
        }

        self.order
            .iter()
            .enumerate()
            .min_by_key(|(_, at)| at.abs_diff(index))
            .map(|(position, _)| position)
    }

    /// The position after `position`, wrapping at the end.
    pub fn next(&self, position: usize) -> Option<usize> {
        (!self.is_empty()).then(|| (position + 1) % self.len())
    }

    /// The position before it, wrapping at the beginning.
    pub fn previous(&self, position: usize) -> Option<usize> {
        (!self.is_empty()).then(|| match position {
            0 => self.len() - 1,
            _ => position - 1,
        })
    }

    /// Drops a store position and shifts the ones after it down, keeping the
    /// list aligned with a store an image has just left.
    pub fn remove_shifting(&mut self, index: usize) {
        self.order.retain(|at| *at != index);

        for at in &mut self.order {
            if *at > index {
                *at -= 1;
            }
        }

        self.everything = self.order.iter().enumerate().all(|(i, at)| i == *at);
    }

    pub fn iter(&self) -> impl Iterator<Item = usize> + '_ {
        self.order.iter().copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn everything_is_every_position_in_turn() {
        let visible = Visible::everything(4);

        assert_eq!(visible.len(), 4);
        assert!(visible.is_everything());
        assert_eq!(visible.at(2), Some(2));
        assert_eq!(visible.position_of(3), Some(3));
        assert_eq!(visible.at(4), None);
        assert_eq!(visible.position_of(9), None);
    }

    #[test]
    fn a_chosen_set_maps_both_ways() {
        let visible = Visible::of(vec![3, 0, 7], 8);

        assert!(!visible.is_everything());
        assert_eq!(visible.at(0), Some(3));
        assert_eq!(visible.at(1), Some(0));
        assert_eq!(visible.position_of(7), Some(2));
        assert_eq!(visible.position_of(1), None);
    }

    #[test]
    fn a_full_list_in_order_recognises_itself() {
        assert!(Visible::of(vec![0, 1, 2], 3).is_everything());
        assert!(!Visible::of(vec![0, 2, 1], 3).is_everything());
        assert!(!Visible::of(vec![0, 1], 3).is_everything());
    }

    /// What a filter needs the moment it hides the photograph on screen.
    #[test]
    fn a_hidden_photograph_lands_on_its_nearest_neighbour() {
        let visible = Visible::of(vec![0, 1, 5, 6], 8);

        assert_eq!(visible.nearest(1), Some(1));
        // 4 is hidden; 5 is the nearest thing that is not.
        assert_eq!(visible.nearest(4), Some(2));
        assert_eq!(visible.nearest(7), Some(3));
    }

    #[test]
    fn nothing_visible_has_no_nearest() {
        assert_eq!(Visible::of(Vec::new(), 8).nearest(3), None);
    }

    #[test]
    fn stepping_wraps_at_both_ends() {
        let visible = Visible::of(vec![2, 4, 6], 8);

        assert_eq!(visible.next(0), Some(1));
        assert_eq!(visible.next(2), Some(0));
        assert_eq!(visible.previous(0), Some(2));
        assert_eq!(visible.previous(1), Some(0));
    }

    #[test]
    fn stepping_an_empty_list_goes_nowhere() {
        let visible = Visible::of(Vec::new(), 0);

        assert_eq!(visible.next(0), None);
        assert_eq!(visible.previous(0), None);
    }

    #[test]
    fn removing_shifts_what_comes_after() {
        let mut visible = Visible::of(vec![0, 2, 4], 5);
        visible.remove_shifting(2);

        assert_eq!(visible.iter().collect::<Vec<_>>(), vec![0, 3]);
    }

    #[test]
    fn removing_from_everything_leaves_everything() {
        let mut visible = Visible::everything(3);
        visible.remove_shifting(1);

        assert!(visible.is_everything());
        assert_eq!(visible.iter().collect::<Vec<_>>(), vec![0, 1]);
    }
}
