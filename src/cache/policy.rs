//! The rules that decide what stays in memory and what goes.
//!
//! Both the RAM and the GPU cache keep the images nearest to where the user is
//! looking, in a collection that wraps around at its ends; sharing the policy
//! here keeps the two from drifting apart.

use std::collections::HashMap;

/// Shortest distance between two indices in a collection that wraps around.
pub fn distance(from: usize, to: usize, total: usize) -> usize {
    if total == 0 {
        return 0;
    }

    let forward = (to + total - from % total) % total;
    forward.min(total - forward)
}

/// The index furthest from `cursor`, which is the one to drop next.
///
/// `keep` is never chosen, so a freshly inserted entry cannot evict itself.
pub fn furthest(
    indices: impl Iterator<Item = usize>,
    cursor: usize,
    total: usize,
    keep: usize,
) -> Option<usize> {
    indices
        .filter(|index| *index != keep)
        .max_by_key(|index| distance(cursor, *index, total))
}

/// Indices within `radius` of `cursor`, nearest first, wrapping at the ends.
///
/// This is both the set worth having in memory and the order to load it in.
pub fn window(cursor: usize, total: usize, radius: usize) -> Vec<usize> {
    if total == 0 {
        return Vec::new();
    }

    let cursor = cursor % total;
    let radius = radius.min(total / 2);
    let mut indices = Vec::with_capacity((radius * 2 + 1).min(total));
    indices.push(cursor);

    for step in 1..=radius {
        // Forward first: most navigation goes that way.
        indices.push((cursor + step) % total);
        let backward = (cursor + total - step % total) % total;
        if !indices.contains(&backward) {
            indices.push(backward);
        }
    }

    indices
}

/// Share of the budget the window is allowed to fill.
///
/// The rest is headroom, and it is not optional. A window sized to exactly
/// what the budget holds evicts an image to make room for the next one and
/// then immediately asks for the evicted one again, because it is still in the
/// window — the cache spends all its time redecoding what it just threw away.
const WINDOW_SHARE: (usize, usize) = (3, 4);

/// Trims a preload radius to what a byte budget can comfortably hold.
///
/// Without this, a folder of 60 megapixel raws would be decoded only to be
/// evicted before it could be shown, and then requested again on the next
/// frame.
///
/// The average size of what is already resident stands in for the size of what
/// is not; with nothing resident yet the configured radius is used as is.
pub fn budgeted_radius(
    configured: usize,
    budget_bytes: usize,
    resident_bytes: usize,
    resident_count: usize,
) -> usize {
    if resident_count == 0 || resident_bytes == 0 {
        return configured;
    }

    let average = (resident_bytes / resident_count).max(1);
    let fits = budget_bytes / average * WINDOW_SHARE.0 / WINDOW_SHARE.1;

    // The radius reaches in both directions, hence the halving.
    configured.min(fits / 2)
}

/// Removes `index` and shifts every higher key down by one, keeping a map
/// keyed by position consistent after an image leaves the collection.
pub fn remove_and_shift<V>(entries: &mut HashMap<usize, V>, index: usize) -> Option<V> {
    let removed = entries.remove(&index);

    let shifted: Vec<usize> = entries.keys().copied().filter(|key| *key > index).collect();
    let mut moved: Vec<(usize, V)> = Vec::with_capacity(shifted.len());

    for key in shifted {
        if let Some(value) = entries.remove(&key) {
            moved.push((key - 1, value));
        }
    }

    entries.extend(moved);
    removed
}

/// Makes room at `index`, moving everything at or past it up by one.
///
/// The mirror of [`remove_and_shift`], for a photograph appearing in the
/// middle of an open folder: a tethered shot lands at its sorted position, and
/// every cache entry above it now belongs one place further along. Shifting
/// them is what keeps the rest of the folder decoded — the alternative is
/// reading the folder again and throwing away every texture in it.
pub fn insert_and_shift<V>(entries: &mut HashMap<usize, V>, index: usize) {
    // Downwards, so a key never lands on one that has not moved yet.
    let mut shifted: Vec<usize> = entries
        .keys()
        .copied()
        .filter(|key| *key >= index)
        .collect();
    shifted.sort_unstable_by(|a, b| b.cmp(a));

    for key in shifted {
        if let Some(value) = entries.remove(&key) {
            entries.insert(key + 1, value);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The case that would draw a photograph under its neighbour's name: a
    /// frame lands in the middle of a folder and every cached entry above it
    /// now belongs one place further along.
    #[test]
    fn making_room_moves_everything_above_it_up() {
        let mut entries: HashMap<usize, &str> =
            [(0, "a"), (1, "b"), (2, "c")].into_iter().collect();

        insert_and_shift(&mut entries, 1);

        assert_eq!(entries.get(&0), Some(&"a"));
        assert_eq!(entries.get(&1), None);
        assert_eq!(entries.get(&2), Some(&"b"));
        assert_eq!(entries.get(&3), Some(&"c"));
    }

    #[test]
    fn making_room_at_the_end_moves_nothing() {
        let mut entries: HashMap<usize, &str> = [(0, "a"), (1, "b")].into_iter().collect();

        insert_and_shift(&mut entries, 5);

        assert_eq!(entries.get(&0), Some(&"a"));
        assert_eq!(entries.get(&1), Some(&"b"));
        assert_eq!(entries.len(), 2);
    }

    /// Nothing is lost on the way: shifting upwards has to move the highest
    /// key first or one lands on another that has not moved yet.
    #[test]
    fn making_room_loses_nothing() {
        let mut entries: HashMap<usize, usize> = (0..20).map(|i| (i, i)).collect();

        insert_and_shift(&mut entries, 3);

        assert_eq!(entries.len(), 20);
        for original in 0..20usize {
            let expected = if original >= 3 {
                original + 1
            } else {
                original
            };
            assert_eq!(entries.get(&expected), Some(&original), "{original}");
        }
    }

    /// Making room and then taking it away again leaves what was there.
    #[test]
    fn making_room_and_removing_it_is_a_round_trip() {
        let before: HashMap<usize, usize> = (0..8).map(|i| (i, i)).collect();
        let mut entries = before.clone();

        insert_and_shift(&mut entries, 4);
        remove_and_shift(&mut entries, 4);

        assert_eq!(entries, before);
    }

    #[test]
    fn distance_wraps_around() {
        assert_eq!(distance(0, 1, 10), 1);
        assert_eq!(distance(0, 9, 10), 1);
        assert_eq!(distance(9, 0, 10), 1);
        assert_eq!(distance(0, 5, 10), 5);
        assert_eq!(distance(3, 3, 10), 0);
        assert_eq!(distance(0, 0, 0), 0);
    }

    #[test]
    fn furthest_skips_the_protected_index() {
        assert_eq!(furthest([0, 1, 5].into_iter(), 0, 10, 9), Some(5));
        assert_eq!(furthest([0, 5].into_iter(), 0, 10, 5), Some(0));
        assert_eq!(furthest([5].into_iter(), 0, 10, 5), None);
        assert_eq!(furthest([].into_iter(), 0, 10, 5), None);
    }

    #[test]
    fn window_is_ordered_by_proximity() {
        assert_eq!(window(5, 100, 2), vec![5, 6, 4, 7, 3]);
    }

    #[test]
    fn window_wraps_at_the_ends() {
        assert_eq!(window(0, 10, 2), vec![0, 1, 9, 2, 8]);
        assert_eq!(window(9, 10, 1), vec![9, 0, 8]);
    }

    #[test]
    fn window_never_repeats_an_index() {
        let indices = window(1, 4, 10);
        let mut deduped = indices.clone();
        deduped.sort_unstable();
        deduped.dedup();

        assert_eq!(indices.len(), deduped.len());
        assert!(indices.len() <= 4);
    }

    #[test]
    fn window_of_an_empty_collection_is_empty() {
        assert!(window(0, 0, 5).is_empty());
    }

    #[test]
    fn the_radius_shrinks_to_fit_the_budget() {
        // Twenty images of 100 bytes fit in 2000; three quarters of that is
        // fifteen, so seven either side.
        assert_eq!(budgeted_radius(64, 2000, 500, 5), 7);
        // A generous budget leaves the configured radius alone.
        assert_eq!(budgeted_radius(8, 1_000_000, 500, 5), 8);
    }

    #[test]
    fn the_window_leaves_the_budget_room_to_spare() {
        // Whatever the budget, what the window asks for has to fit inside it
        // with room left, or every insert evicts something still wanted.
        for images in 2..200usize {
            let average = 1_000_000;
            let budget = images * average;
            let radius = budgeted_radius(usize::MAX, budget, average, 1);
            let window = radius * 2 + 1;

            assert!(
                window * average <= budget,
                "{images} images: a window of {window} does not fit"
            );
        }
    }

    #[test]
    fn an_empty_cache_keeps_the_configured_radius() {
        assert_eq!(budgeted_radius(32, 1000, 0, 0), 32);
        assert_eq!(budgeted_radius(32, 1000, 0, 5), 32);
    }

    #[test]
    fn a_budget_holding_one_image_stops_preloading() {
        assert_eq!(budgeted_radius(32, 100, 100, 1), 0);
    }

    #[test]
    fn removing_shifts_the_keys_above_it() {
        let mut entries: HashMap<usize, &str> =
            HashMap::from([(0, "a"), (1, "b"), (2, "c"), (3, "d")]);

        assert_eq!(remove_and_shift(&mut entries, 1), Some("b"));
        assert_eq!(entries.get(&0), Some(&"a"));
        assert_eq!(entries.get(&1), Some(&"c"));
        assert_eq!(entries.get(&2), Some(&"d"));
        assert_eq!(entries.get(&3), None);
    }

    #[test]
    fn removing_a_missing_key_still_shifts() {
        let mut entries: HashMap<usize, &str> = HashMap::from([(4, "e")]);
        assert_eq!(remove_and_shift(&mut entries, 1), None);
        assert_eq!(entries.get(&3), Some(&"e"));
    }
}
