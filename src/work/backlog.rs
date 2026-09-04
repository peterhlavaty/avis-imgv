//! The three ways this program holds work that has not been done yet.
//!
//! Each is a whole answer to "which of these matters most now", and they are
//! genuinely different answers rather than one with a parameter:
//!
//! - [`Ranked`] — nearest the cursor first, because the next photograph is the
//!   one about to be looked at;
//! - [`Newest`] — last asked for first, and only so many kept, because a
//!   preview nobody is still waiting for is not worth reading;
//! - [`Coalescing`] — one entry per file, later replacing earlier, because
//!   pressing `3` and then `4` on one photograph is one save and not two.

use std::collections::{BinaryHeap, HashMap, VecDeque};
use std::hash::Hash;

/// Where a pool keeps what it has been asked to do.
///
/// The queue discipline is the only thing the three pools in this program
/// disagreed about, and it is what this trait names. Everything else — the
/// lock, the condition variable, the shutdown flag, the join — was written out
/// three times identically.
pub trait Backlog: Send + 'static {
    /// One piece of work.
    type Item: Send + 'static;

    /// Takes one on.
    fn put(&mut self, item: Self::Item);

    /// Hands back whichever should be done next, if any.
    fn take(&mut self) -> Option<Self::Item>;

    /// Forgets everything waiting, for when the open folder changes.
    fn clear(&mut self);

    /// How many are waiting.
    fn len(&self) -> usize;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Nearest first, by the item's own order.
///
/// A min-heap: `Ord` on the item says what "nearest" means, so the caller
/// orders by distance from the cursor and this knows nothing about cursors.
pub struct Ranked<T: Ord + Send + 'static> {
    heap: BinaryHeap<std::cmp::Reverse<T>>,
}

impl<T: Ord + Send + 'static> Default for Ranked<T> {
    fn default() -> Self {
        Self {
            heap: BinaryHeap::new(),
        }
    }
}

impl<T: Ord + Send + 'static> Backlog for Ranked<T> {
    type Item = T;

    fn put(&mut self, item: T) {
        self.heap.push(std::cmp::Reverse(item));
    }

    fn take(&mut self) -> Option<T> {
        self.heap.pop().map(|std::cmp::Reverse(item)| item)
    }

    fn clear(&mut self) {
        self.heap.clear();
    }

    fn len(&self) -> usize {
        self.heap.len()
    }
}

/// Last asked for first, keeping at most `KEPT`.
///
/// The photograph the viewer is on now matters more than the one it was on a
/// moment ago, and a person who has walked past forty frames is not waiting
/// for the first of them. The bound is what stops a fast scroll queueing a
/// folder's worth of reads nobody will look at.
pub struct Newest<T: Send + 'static, const KEPT: usize> {
    queue: VecDeque<T>,
}

impl<T: Send + 'static, const KEPT: usize> Default for Newest<T, KEPT> {
    fn default() -> Self {
        Self {
            queue: VecDeque::new(),
        }
    }
}

impl<T: Send + 'static, const KEPT: usize> Backlog for Newest<T, KEPT> {
    type Item = T;

    fn put(&mut self, item: T) {
        self.queue.push_front(item);
        self.queue.truncate(KEPT);
    }

    fn take(&mut self) -> Option<T> {
        self.queue.pop_front()
    }

    fn clear(&mut self) {
        self.queue.clear();
    }

    fn len(&self) -> usize {
        self.queue.len()
    }
}

/// One entry per key, a later one replacing an earlier.
///
/// Rating a photograph three and then four is one sidecar to write, not two,
/// and writing the first would be writing a value the user has already changed
/// their mind about.
pub struct Coalescing<K: Eq + Hash + Clone + Send + 'static, V: Send + 'static> {
    /// Insertion-ordered: the map alone picks an arbitrary key, so a burst of
    /// edits was written back in whatever order the hasher happened to give,
    /// which is not the order they were made in.
    order: VecDeque<K>,
    held: HashMap<K, V>,
}

impl<K: Eq + Hash + Clone + Send + 'static, V: Send + 'static> Default for Coalescing<K, V> {
    fn default() -> Self {
        Self {
            order: VecDeque::new(),
            held: HashMap::new(),
        }
    }
}

impl<K: Eq + Hash + Clone + Send + 'static, V: Send + 'static> Backlog for Coalescing<K, V> {
    type Item = (K, V);

    fn put(&mut self, (key, value): (K, V)) {
        if self.held.insert(key.clone(), value).is_none() {
            self.order.push_back(key);
        }
    }

    fn take(&mut self) -> Option<(K, V)> {
        while let Some(key) = self.order.pop_front() {
            if let Some(value) = self.held.remove(&key) {
                return Some((key, value));
            }
        }

        None
    }

    fn clear(&mut self) {
        self.order.clear();
        self.held.clear();
    }

    fn len(&self) -> usize {
        self.held.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ranked_hands_back_the_nearest_first() {
        let mut backlog = Ranked::default();

        backlog.put(5);
        backlog.put(1);
        backlog.put(3);

        assert_eq!(backlog.take(), Some(1));
        assert_eq!(backlog.take(), Some(3));
        assert_eq!(backlog.take(), Some(5));
        assert_eq!(backlog.take(), None);
    }

    #[test]
    fn newest_hands_back_the_last_asked_for() {
        let mut backlog: Newest<u32, 8> = Newest::default();

        backlog.put(1);
        backlog.put(2);
        backlog.put(3);

        assert_eq!(backlog.take(), Some(3));
        assert_eq!(backlog.take(), Some(2));
    }

    /// A fast scroll must not queue a folder's worth of reads nobody will
    /// look at, so the oldest fall off the end.
    #[test]
    fn newest_keeps_only_so_many() {
        let mut backlog: Newest<u32, 3> = Newest::default();

        for n in 0..10 {
            backlog.put(n);
        }

        assert_eq!(backlog.take(), Some(9));
        assert_eq!(backlog.take(), Some(8));
        assert_eq!(backlog.take(), Some(7));
        assert_eq!(backlog.take(), None);
    }

    #[test]
    fn coalescing_keeps_one_entry_per_key() {
        let mut backlog = Coalescing::default();

        backlog.put(("a.jpg", 3));
        backlog.put(("b.jpg", 1));
        backlog.put(("a.jpg", 4));

        assert_eq!(backlog.take(), Some(("a.jpg", 4)));
        assert_eq!(backlog.take(), Some(("b.jpg", 1)));
        assert_eq!(backlog.take(), None);
    }

    /// Rating a photograph three and then four is one save, and it is the
    /// four that is written — never the three.
    #[test]
    fn a_later_edit_replaces_an_earlier_one_in_place() {
        let mut backlog = Coalescing::default();

        backlog.put(("a.jpg", 3));
        backlog.put(("a.jpg", 4));

        assert_eq!(backlog.take(), Some(("a.jpg", 4)));
        assert!(backlog.is_empty());
    }

    /// The order edits were made in, not the order a hasher happens to give:
    /// the map alone picked an arbitrary key, so a burst of edits was written
    /// back in no particular order.
    #[test]
    fn coalescing_writes_in_the_order_the_edits_were_made() {
        let mut backlog = Coalescing::default();

        for name in ["c", "a", "b", "d"] {
            backlog.put((name, 1));
        }

        let taken: Vec<_> = std::iter::from_fn(|| backlog.take())
            .map(|(key, _)| key)
            .collect();

        assert_eq!(taken, vec!["c", "a", "b", "d"]);
    }

    /// A key edited again after its first entry keeps the place it had, so one
    /// photograph fiddled with repeatedly cannot starve the others.
    #[test]
    fn a_replaced_entry_keeps_its_place_in_the_queue() {
        let mut backlog = Coalescing::default();

        backlog.put(("a", 1));
        backlog.put(("b", 1));
        backlog.put(("a", 2));

        assert_eq!(backlog.take(), Some(("a", 2)));
        assert_eq!(backlog.take(), Some(("b", 1)));
    }

    #[test]
    fn everything_can_be_forgotten() {
        let mut ranked = Ranked::default();
        let mut newest: Newest<u32, 4> = Newest::default();
        let mut coalescing = Coalescing::default();

        ranked.put(1);
        newest.put(1);
        coalescing.put(("a", 1));

        ranked.clear();
        newest.clear();
        coalescing.clear();

        assert!(ranked.is_empty());
        assert!(newest.is_empty());
        assert!(coalescing.is_empty());
    }
}
