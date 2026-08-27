//! RAM residency for decoded images, bounded by a byte budget.
//!
//! The viewer wants the whole folder in memory, but "the whole folder" can be
//! a hundred gigabytes of 60 megapixel raws. The budget turns that ambition
//! into a sliding window: everything fits until it doesn't, and then the
//! images furthest from where the user is looking are the ones dropped.

use std::collections::HashMap;
use std::sync::Arc;

use crate::decoder::DecodedImage;

use super::policy;

/// Decoded images held in RAM, evicted by distance from the viewer's position.
pub struct RamCache {
    entries: HashMap<usize, Arc<DecodedImage>>,
    resident_bytes: usize,
    budget_bytes: usize,
}

impl RamCache {
    pub fn new(budget_bytes: usize) -> RamCache {
        RamCache {
            entries: HashMap::new(),
            resident_bytes: 0,
            budget_bytes,
        }
    }

    pub fn get(&self, index: usize) -> Option<&Arc<DecodedImage>> {
        self.entries.get(&index)
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

    pub fn resident_bytes(&self) -> usize {
        self.resident_bytes
    }

    pub fn budget_bytes(&self) -> usize {
        self.budget_bytes
    }

    /// Adds an image, then evicts until the budget is met again.
    ///
    /// The freshly inserted image is never the one evicted, so a single image
    /// larger than the whole budget still displays.
    pub fn insert(&mut self, index: usize, image: Arc<DecodedImage>, cursor: usize, total: usize) {
        self.resident_bytes += image.byte_len();

        if let Some(replaced) = self.entries.insert(index, image) {
            self.resident_bytes = self.resident_bytes.saturating_sub(replaced.byte_len());
        }

        self.evict_until_within_budget(index, cursor, total);
    }

    pub fn remove(&mut self, index: usize) {
        if let Some(removed) = self.entries.remove(&index) {
            self.resident_bytes = self.resident_bytes.saturating_sub(removed.byte_len());
        }
    }

    /// Removes an image that has left the collection, shifting the entries
    /// above it down so the cache stays keyed by position.
    pub fn remove_shifting(&mut self, index: usize) {
        if let Some(removed) = policy::remove_and_shift(&mut self.entries, index) {
            self.resident_bytes = self.resident_bytes.saturating_sub(removed.byte_len());
        }
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.resident_bytes = 0;
    }

    /// Indices currently held, in no particular order.
    pub fn indices(&self) -> impl Iterator<Item = usize> + '_ {
        self.entries.keys().copied()
    }

    fn evict_until_within_budget(&mut self, keep: usize, cursor: usize, total: usize) {
        while self.resident_bytes > self.budget_bytes {
            let Some(victim) = self.furthest_from(cursor, total, keep) else {
                return;
            };

            tracing::debug!("RAM budget reached, evicting image {victim}");
            self.remove(victim);
        }
    }

    fn furthest_from(&self, cursor: usize, total: usize, keep: usize) -> Option<usize> {
        policy::furthest(self.entries.keys().copied(), cursor, total, keep)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metadata::Metadata;

    fn image(bytes: usize) -> Arc<DecodedImage> {
        Arc::new(DecodedImage {
            full: crate::decoder::Surface {
                pixels: vec![0u8; bytes].into_boxed_slice(),
                width: 1,
                height: (bytes / 4) as u32,
            },
            display: None,
            orientation: crate::metadata::Orientation::Normal,
            metadata: Metadata::default(),
        })
    }

    #[test]
    fn holds_images_until_the_budget_is_reached() {
        let mut cache = RamCache::new(400);
        for index in 0..4 {
            cache.insert(index, image(100), 0, 10);
        }

        assert_eq!(cache.len(), 4);
        assert_eq!(cache.resident_bytes(), 400);
    }

    #[test]
    fn evicts_the_image_furthest_from_the_cursor() {
        let mut cache = RamCache::new(300);
        for index in 0..3 {
            cache.insert(index, image(100), 0, 10);
        }

        // Index 5 is the furthest from cursor 0 in a collection of ten.
        cache.insert(5, image(100), 0, 10);

        assert!(cache.contains(0));
        assert!(cache.contains(5), "the new image is never the victim");
        assert_eq!(cache.len(), 3);
        assert!(cache.resident_bytes() <= 300);
    }

    #[test]
    fn eviction_follows_the_cursor_around_the_wrap() {
        let mut cache = RamCache::new(300);
        for index in [0, 1, 9] {
            cache.insert(index, image(100), 9, 10);
        }

        // From index 9, index 1 is two steps away and index 0 only one.
        cache.insert(5, image(100), 9, 10);

        assert!(!cache.contains(1));
        assert!(cache.contains(0));
        assert!(cache.contains(9));
    }

    #[test]
    fn an_image_larger_than_the_budget_still_fits() {
        let mut cache = RamCache::new(100);
        cache.insert(0, image(1000), 0, 1);

        assert!(cache.contains(0));
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn replacing_an_entry_does_not_double_count() {
        let mut cache = RamCache::new(10_000);
        cache.insert(0, image(100), 0, 10);
        cache.insert(0, image(200), 0, 10);

        assert_eq!(cache.len(), 1);
        assert_eq!(cache.resident_bytes(), 200);
    }

    #[test]
    fn removing_shifts_the_remaining_entries() {
        let mut cache = RamCache::new(10_000);
        for index in 0..3 {
            cache.insert(index, image(100), 0, 3);
        }

        cache.remove_shifting(0);

        assert_eq!(cache.len(), 2);
        assert_eq!(cache.resident_bytes(), 200);
        assert!(cache.contains(0));
        assert!(cache.contains(1));
        assert!(!cache.contains(2));
    }

    #[test]
    fn clearing_resets_the_accounting() {
        let mut cache = RamCache::new(10_000);
        cache.insert(0, image(100), 0, 10);
        cache.clear();

        assert!(cache.is_empty());
        assert_eq!(cache.resident_bytes(), 0);
    }
}
