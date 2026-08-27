//! Turning the configuration into the budgets each store runs on.

use crate::cache::StoreConfig;
use crate::config::{CacheConfig, GridViewConfig, ImageViewConfig};

/// Share of the RAM budget the thumbnail grid may use.
///
/// Thumbnails are small but numerous; an eighth holds several hundred of them
/// while leaving the full size view the room it needs.
const THUMBNAIL_SHARE: usize = 8;

/// Floor on the thumbnail budget, so a small overall budget still fills a
/// screen of the grid.
const MIN_THUMBNAIL_BYTES: usize = 64 * 1024 * 1024;

/// Rows of thumbnails assumed visible, on top of the configured preload.
const VISIBLE_ROWS: usize = 8;

/// Priority penalty on thumbnail decoding.
///
/// Both views share one decode pool and the grid keeps filling in while the
/// image view is on screen; this makes sure the photograph the user is looking
/// at is never behind a thumbnail in the queue.
const THUMBNAIL_PRIORITY_BIAS: usize = 10_000;

/// Budgets for the full size image view.
pub fn image_store(cache: &CacheConfig, view: &ImageViewConfig) -> StoreConfig {
    StoreConfig {
        ram_budget_bytes: split(cache.ram_budget_mb).0,
        gpu_resident: view.gpu_resident_images,
        preload_radius: view.nr_loaded_images,
        max_edge: non_zero(view.max_image_edge),
        uploads_per_frame: cache.uploads_per_frame,
        priority_bias: 0,
    }
}

/// Budgets for the thumbnail grid.
pub fn thumbnail_store(cache: &CacheConfig, view: &GridViewConfig) -> StoreConfig {
    StoreConfig {
        ram_budget_bytes: split(cache.ram_budget_mb).1,
        gpu_resident: view.gpu_resident_thumbnails,
        // The grid scrolls in rows, so its window is measured in rows too.
        preload_radius: (view.preloaded_rows + VISIBLE_ROWS) * view.images_per_row.max(1),
        max_edge: non_zero(view.thumbnail_resolution),
        uploads_per_frame: cache.uploads_per_frame,
        priority_bias: THUMBNAIL_PRIORITY_BIAS,
    }
}

/// Splits the configured budget into `(full size, thumbnails)` bytes.
fn split(ram_budget_mb: usize) -> (usize, usize) {
    let total = ram_budget_mb.max(1) * 1024 * 1024;
    let thumbnails = (total / THUMBNAIL_SHARE)
        .max(MIN_THUMBNAIL_BYTES)
        .min(total / 2);

    (total - thumbnails, thumbnails)
}

/// Reads a configured zero as "no limit".
fn non_zero(value: u32) -> Option<u32> {
    (value > 0).then_some(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn most_of_the_budget_goes_to_full_size_images() {
        let (images, thumbnails) = split(4096);

        assert!(images > thumbnails);
        assert_eq!(images + thumbnails, 4096 * 1024 * 1024);
    }

    #[test]
    fn a_tiny_budget_is_split_in_half() {
        let (images, thumbnails) = split(1);

        assert!(images > 0);
        assert!(thumbnails > 0);
        assert_eq!(images + thumbnails, 1024 * 1024);
    }

    #[test]
    fn a_zero_budget_is_treated_as_one_megabyte() {
        let (images, thumbnails) = split(0);
        assert_eq!(images + thumbnails, 1024 * 1024);
    }

    #[test]
    fn zero_means_no_size_limit() {
        assert_eq!(non_zero(0), None);
        assert_eq!(non_zero(4096), Some(4096));
    }

    #[test]
    fn the_grid_preloads_by_rows() {
        let cache = CacheConfig::default();
        let mut view = GridViewConfig {
            images_per_row: 5,
            preloaded_rows: 2,
            ..Default::default()
        };

        let wide = thumbnail_store(&cache, &view);
        view.images_per_row = 10;
        let wider = thumbnail_store(&cache, &view);

        assert_eq!(wider.preload_radius, wide.preload_radius * 2);
    }

    #[test]
    fn thumbnails_yield_to_full_size_images() {
        let cache = CacheConfig::default();
        let images = image_store(&cache, &ImageViewConfig::default());
        let thumbnails = thumbnail_store(&cache, &GridViewConfig::default());

        // The furthest full size image must still outrank the nearest
        // thumbnail.
        assert!(images.priority_bias + images.preload_radius < thumbnails.priority_bias);
    }

    #[test]
    fn the_two_stores_share_one_budget() {
        let cache = CacheConfig::default();
        let images = image_store(&cache, &ImageViewConfig::default());
        let thumbnails = thumbnail_store(&cache, &GridViewConfig::default());

        assert_eq!(
            images.ram_budget_bytes + thumbnails.ram_budget_bytes,
            cache.ram_budget_mb * 1024 * 1024
        );
    }
}
