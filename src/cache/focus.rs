//! What the viewer is looking at, as the decode workers see it.
//!
//! A request can sit in the queue for as long as the images ahead of it take,
//! by which time the user may have scrolled past it. Rather than let the pool
//! work through a backlog nobody is waiting for, every worker checks this
//! before it starts: it is three atomics, so checking costs nothing.

use std::sync::atomic::{AtomicUsize, Ordering};

use super::loader::ImageKey;
use super::policy::distance;

/// What the viewer currently cares about, shared with the workers so they can
/// abandon requests that went stale while queued.
#[derive(Debug, Default)]
pub struct Focus {
    generation: AtomicUsize,
    cursor: AtomicUsize,
    total: AtomicUsize,
    window: AtomicUsize,
}

impl Focus {
    /// Records the collection the viewer moved to.
    pub fn set_collection(&self, generation: u64, total: usize) {
        self.generation
            .store(generation as usize, Ordering::Relaxed);
        self.total.store(total, Ordering::Relaxed);
    }

    /// Records where the viewer is and how far around it work is still useful.
    pub fn set_position(&self, cursor: usize, window: usize) {
        self.cursor.store(cursor, Ordering::Relaxed);
        self.window.store(window, Ordering::Relaxed);
    }

    /// Whether a queued request is still worth decoding.
    ///
    /// `radius` narrows the store's preload window for requests that only make
    /// sense close to the cursor, such as decoding an image at full resolution
    /// so it is ready to be zoomed into.
    pub fn accepts(&self, key: ImageKey, radius: Option<usize>) -> bool {
        if key.generation as usize != self.generation.load(Ordering::Relaxed) {
            return false;
        }

        let total = self.total.load(Ordering::Relaxed);
        if total == 0 {
            return true;
        }

        let window = self.window.load(Ordering::Relaxed);
        let wanted = match (window, radius) {
            (0, None) => return true,
            (0, Some(radius)) => radius,
            (window, None) => window,
            (window, Some(radius)) => window.min(radius),
        };

        distance(self.cursor.load(Ordering::Relaxed), key.index, total) <= wanted
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(generation: u64, index: usize) -> ImageKey {
        ImageKey { generation, index }
    }

    #[test]
    fn focus_rejects_other_generations() {
        let focus = Focus::default();
        focus.set_collection(7, 100);
        focus.set_position(0, 10);

        assert!(focus.accepts(key(7, 3), None));
        assert!(!focus.accepts(key(6, 3), None));
    }

    #[test]
    fn focus_rejects_indices_outside_the_window() {
        let focus = Focus::default();
        focus.set_collection(1, 100);
        focus.set_position(50, 5);

        assert!(focus.accepts(key(1, 55), None));
        assert!(!focus.accepts(key(1, 70), None));
    }

    #[test]
    fn a_radius_narrows_the_window_but_never_widens_it() {
        let focus = Focus::default();
        focus.set_collection(1, 100);
        focus.set_position(50, 20);

        // Work that only makes sense next to the cursor is abandoned as soon
        // as the viewer moves past it, long before the window would.
        assert!(focus.accepts(key(1, 51), Some(1)));
        assert!(!focus.accepts(key(1, 55), Some(1)));

        // And a wide radius does not resurrect what the window has dropped.
        assert!(!focus.accepts(key(1, 80), Some(50)));
    }

    #[test]
    fn a_radius_still_applies_when_there_is_no_window_yet() {
        let focus = Focus::default();
        focus.set_collection(1, 100);

        assert!(focus.accepts(key(1, 0), Some(1)));
        assert!(!focus.accepts(key(1, 40), Some(1)));
    }

    #[test]
    fn a_zero_window_accepts_everything() {
        let focus = Focus::default();
        focus.set_collection(1, 100);
        focus.set_position(0, 0);

        assert!(focus.accepts(key(1, 99), None));
    }
}
