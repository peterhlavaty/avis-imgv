//! What is resident, keyed by position in the collection and bounded.
//!
//! The viewer keeps two of these: decoded photographs in RAM, and textures on
//! the GPU. They were written out twice, method for method — the same map, the
//! same running byte total, the same budget, the same shift when a photograph
//! joins or leaves the collection, and the same eviction of whatever is
//! furthest from where the user is looking.
//!
//! The differences are two, and both are arguments rather than separate types:
//! the GPU bounds the *count* of live textures as well as their size, because
//! a descriptor costs something even when the picture is small; and the two
//! measure themselves differently, which is [`Resident`].
//!
//! What is deliberately not here is the *uploading*. `cache::gpu` still owns
//! the device, the pipeline and the mipmap generator, because those are about
//! talking to an adapter and this is about remembering what is held. Keeping
//! them apart is what lets nearly all of the accounting be tested without one.

use std::collections::HashMap;

use super::policy;

/// Something whose residency is worth budgeting.
///
/// The two implementors measure themselves differently — a decoded photograph
/// knows its buffer, a texture knows what it asked the adapter for — and the
/// budget only ever wants the number.
pub trait Resident {
    /// What holding this costs, in bytes.
    fn byte_len(&self) -> usize;
}

/// A budgeted map from position in the collection to whatever is held there.
///
/// Never lets the thing just inserted be the thing evicted, so a single
/// photograph larger than the whole budget still appears.
pub struct Residency<T: Resident> {
    entries: HashMap<usize, T>,
    resident_bytes: usize,
    budget_bytes: usize,
    /// A ceiling on how many may be held, where one applies.
    ///
    /// The GPU has one because a live texture descriptor costs something
    /// whatever the picture's size; RAM has none, because bytes are the whole
    /// story there.
    capacity: Option<usize>,
}

impl<T: Resident> Residency<T> {
    /// Bounded by size alone.
    pub fn new(budget_bytes: usize) -> Residency<T> {
        Residency {
            entries: HashMap::new(),
            resident_bytes: 0,
            // Never nought: a budget that holds nothing would evict
            // everything the moment it arrived and draw an empty window.
            budget_bytes: budget_bytes.max(1),
            capacity: None,
        }
    }

    /// Bounded by size and by count, whichever is reached first.
    pub fn bounded(budget_bytes: usize, capacity: usize) -> Residency<T> {
        Residency {
            capacity: Some(capacity.max(1)),
            ..Residency::new(budget_bytes)
        }
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        self.entries.get(&index)
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.entries.get_mut(&index)
    }

    pub fn contains(&self, index: usize) -> bool {
        self.entries.contains_key(&index)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// What the resident entries add up to.
    pub fn resident_bytes(&self) -> usize {
        self.resident_bytes
    }

    /// The ceiling on that.
    pub fn budget_bytes(&self) -> usize {
        self.budget_bytes
    }

    /// How many may be held at once, where that is bounded.
    pub fn capacity(&self) -> Option<usize> {
        self.capacity
    }

    /// Changes the count ceiling, for a setting the user has just moved.
    ///
    /// Nothing is evicted here: the next insert brings both bounds back, and
    /// throwing textures away on the frame a slider moved would make dragging
    /// it a stutter.
    pub fn set_capacity(&mut self, capacity: usize) {
        self.capacity = Some(capacity.max(1));
    }

    /// Changes the size ceiling, likewise.
    pub fn set_budget_bytes(&mut self, budget_bytes: usize) {
        self.budget_bytes = budget_bytes.max(1);
    }

    /// Positions currently held, in no particular order.
    pub fn indices(&self) -> impl Iterator<Item = usize> + '_ {
        self.entries.keys().copied()
    }

    /// Adds one, then evicts until both bounds are met again.
    ///
    /// Whatever was evicted is handed back, so a caller that has to do
    /// something about it — tell the drawing layer a texture has gone — can.
    /// The freshly inserted entry is never among them.
    pub fn insert(
        &mut self,
        index: usize,
        held: T,
        cursor: usize,
        total: usize,
    ) -> Vec<(usize, T)> {
        self.resident_bytes += held.byte_len();

        if let Some(replaced) = self.entries.insert(index, held) {
            self.resident_bytes = self.resident_bytes.saturating_sub(replaced.byte_len());
        }

        self.evict_until_within_bounds(index, cursor, total)
    }

    pub fn remove(&mut self, index: usize) -> Option<T> {
        let removed = self.entries.remove(&index);

        if let Some(gone) = &removed {
            self.resident_bytes = self.resident_bytes.saturating_sub(gone.byte_len());
        }

        removed
    }

    /// Drops what was at `index` and shifts everything above it down, for a
    /// photograph that has left the collection.
    pub fn remove_shifting(&mut self, index: usize) -> Option<T> {
        let removed = policy::remove_and_shift(&mut self.entries, index);

        if let Some(gone) = &removed {
            self.resident_bytes = self.resident_bytes.saturating_sub(gone.byte_len());
        }

        removed
    }

    /// Makes room for a photograph appearing at `index`.
    pub fn insert_shifting(&mut self, index: usize) {
        policy::insert_and_shift(&mut self.entries, index);
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.resident_bytes = 0;
    }

    /// Drops everything whose position `keep` refuses.
    ///
    /// For a collection that has been rebuilt rather than nudged: what the
    /// viewer holds is answered in one pass rather than one removal at a time.
    pub fn retain(&mut self, keep: impl Fn(usize) -> bool) {
        let mut freed = 0;

        self.entries.retain(|index, held| {
            let staying = keep(*index);

            if !staying {
                freed += held.byte_len();
            }

            staying
        });

        self.resident_bytes = self.resident_bytes.saturating_sub(freed);
    }

    /// Evicts whatever is furthest from the cursor until both bounds hold.
    ///
    /// Returns what went, newest bound first. `keep` is never evicted, which
    /// is what lets one photograph larger than the whole budget still appear.
    fn evict_until_within_bounds(
        &mut self,
        keep: usize,
        cursor: usize,
        total: usize,
    ) -> Vec<(usize, T)> {
        let mut evicted = Vec::new();

        while self.over_budget() {
            let Some(victim) = policy::furthest(self.indices(), cursor, total, keep) else {
                break;
            };

            let Some(gone) = self.remove(victim) else {
                break;
            };

            evicted.push((victim, gone));
        }

        evicted
    }

    fn over_budget(&self) -> bool {
        self.resident_bytes > self.budget_bytes
            || self.capacity.is_some_and(|most| self.entries.len() > most)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A thing of a stated size, so the accounting can be tested without
    /// decoding anything or asking for a device.
    #[derive(Debug, PartialEq)]
    struct Weighs(usize);

    impl Resident for Weighs {
        fn byte_len(&self) -> usize {
            self.0
        }
    }

    fn residency() -> Residency<Weighs> {
        Residency::new(100)
    }

    #[test]
    fn what_is_put_in_is_what_comes_out() {
        let mut held = residency();

        held.insert(3, Weighs(10), 3, 10);

        assert!(held.contains(3));
        assert_eq!(held.get(3), Some(&Weighs(10)));
        assert_eq!(held.len(), 1);
        assert_eq!(held.resident_bytes(), 10);
    }

    #[test]
    fn removing_gives_the_bytes_back() {
        let mut held = residency();
        held.insert(1, Weighs(40), 1, 10);

        assert_eq!(held.remove(1), Some(Weighs(40)));
        assert_eq!(held.resident_bytes(), 0);
        assert!(held.is_empty());
    }

    #[test]
    fn replacing_an_entry_counts_it_once() {
        let mut held = residency();

        held.insert(1, Weighs(40), 1, 10);
        held.insert(1, Weighs(10), 1, 10);

        assert_eq!(held.len(), 1);
        assert_eq!(held.resident_bytes(), 10);
    }

    #[test]
    fn what_is_furthest_from_the_cursor_goes_first() {
        let mut held = Residency::new(30);

        held.insert(0, Weighs(10), 5, 10);
        held.insert(5, Weighs(10), 5, 10);
        held.insert(6, Weighs(10), 5, 10);
        // Over budget: 0 is furthest from the cursor at 5.
        let evicted = held.insert(4, Weighs(10), 5, 10);

        assert_eq!(evicted.len(), 1);
        assert_eq!(evicted[0].0, 0);
        assert!(!held.contains(0));
        assert!(held.contains(5));
    }

    /// One photograph larger than the whole budget still appears, because the
    /// thing just inserted is never the thing evicted.
    #[test]
    fn something_bigger_than_the_budget_is_still_held() {
        let mut held = Residency::new(50);

        let evicted = held.insert(2, Weighs(500), 2, 10);

        assert!(held.contains(2));
        assert!(evicted.is_empty());
        assert_eq!(held.resident_bytes(), 500);
    }

    /// The GPU's second bound: a live texture descriptor costs something
    /// whatever the picture's size.
    #[test]
    fn a_count_bound_evicts_even_when_the_bytes_fit() {
        let mut held: Residency<Weighs> = Residency::bounded(10_000, 2);

        held.insert(0, Weighs(1), 5, 10);
        held.insert(5, Weighs(1), 5, 10);
        let evicted = held.insert(6, Weighs(1), 5, 10);

        assert_eq!(held.len(), 2);
        assert_eq!(evicted.len(), 1);
        assert_eq!(evicted[0].0, 0, "the furthest from the cursor");
    }

    #[test]
    fn a_budget_of_nothing_still_holds_something() {
        let mut held: Residency<Weighs> = Residency::new(0);

        held.insert(1, Weighs(10), 1, 10);

        assert!(held.contains(1), "or the window would be empty");
    }

    #[test]
    fn a_photograph_leaving_shifts_what_is_above_it_down() {
        let mut held = residency();

        held.insert(1, Weighs(1), 1, 10);
        held.insert(3, Weighs(1), 1, 10);
        held.insert(5, Weighs(1), 1, 10);

        let gone = held.remove_shifting(3);

        assert_eq!(gone, Some(Weighs(1)));
        assert!(held.contains(1));
        assert!(held.contains(4), "5 has become 4");
        assert!(!held.contains(5));
        assert_eq!(held.resident_bytes(), 2);
    }

    #[test]
    fn a_photograph_arriving_shifts_what_is_at_or_above_it_up() {
        let mut held = residency();

        held.insert(1, Weighs(1), 1, 10);
        held.insert(2, Weighs(1), 1, 10);

        held.insert_shifting(2);

        assert!(held.contains(1));
        assert!(held.contains(3), "2 has become 3");
        assert!(
            !held.contains(2),
            "the new position is empty until it is filled"
        );
    }

    #[test]
    fn removing_something_that_is_not_there_costs_nothing() {
        let mut held = residency();
        held.insert(1, Weighs(10), 1, 10);

        assert_eq!(held.remove(9), None);
        assert_eq!(held.remove_shifting(9), None);
        assert_eq!(held.resident_bytes(), 10);
    }

    #[test]
    fn clearing_forgets_the_bytes_as_well_as_the_entries() {
        let mut held = residency();
        held.insert(1, Weighs(10), 1, 10);

        held.clear();

        assert!(held.is_empty());
        assert_eq!(held.resident_bytes(), 0);
    }

    #[test]
    fn retaining_keeps_what_it_is_told_to_and_the_bytes_follow() {
        let mut held = residency();

        for index in 0..4 {
            held.insert(index, Weighs(10), 0, 4);
        }

        held.retain(|index| index % 2 == 0);

        assert_eq!(held.len(), 2);
        assert_eq!(held.resident_bytes(), 20);
        assert!(held.contains(0) && held.contains(2));
    }

    #[test]
    fn an_empty_residency_evicts_nothing_and_does_not_spin() {
        let mut held: Residency<Weighs> = Residency::new(1);

        assert_eq!(held.remove(0), None);
        assert!(held.indices().next().is_none());
    }
}
