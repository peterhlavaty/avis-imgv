//! Turning the configuration into the settings each store runs on.

use std::time::Duration;

use crate::cache::StoreConfig;
use crate::config::{
    CacheConfig, GridViewConfig, ImageViewConfig, RawConfig, RawQuality, RawSource,
};
use crate::decoder::raw;

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

/// Settings for the full size image view.
pub fn image_store(cache: &CacheConfig, view: &ImageViewConfig, raw: &RawConfig) -> StoreConfig {
    StoreConfig {
        ram_budget_bytes: split(cache.ram_budget_mb).0,
        gpu_resident: view.gpu_resident_images,
        gpu_budget_bytes: gpu_split(cache.gpu_budget_mb).0,
        preload_radius: view.nr_loaded_images,
        max_edge: non_zero(view.max_image_edge),
        previews_resident: cache.previews_resident,
        full_resolution_neighbours: cache.full_resolution_neighbours,
        upload_budget: Duration::from_millis(cache.upload_budget_ms),
        priority_bias: 0,
        raw: raw_options(raw),
    }
}

/// Settings for the thumbnail grid.
pub fn thumbnail_store(cache: &CacheConfig, view: &GridViewConfig) -> StoreConfig {
    StoreConfig {
        ram_budget_bytes: split(cache.ram_budget_mb).1,
        gpu_resident: view.gpu_resident_thumbnails,
        gpu_budget_bytes: gpu_split(cache.gpu_budget_mb).1,
        // The grid scrolls in rows, so its window is measured in rows too.
        preload_radius: (view.preloaded_rows + VISIBLE_ROWS) * view.images_per_row.max(1),
        max_edge: non_zero(view.thumbnail_resolution),
        // The grid draws thumbnails already; standing in for one with a
        // smaller thumbnail would buy nothing, and nothing in it is zoomed.
        previews_resident: 0,
        full_resolution_neighbours: 0,
        upload_budget: Duration::from_millis(cache.upload_budget_ms),
        priority_bias: THUMBNAIL_PRIORITY_BIAS,
        // Thumbnails always come from the embedded preview. Developing a
        // whole folder of raws to fill a contact sheet would take minutes.
        raw: raw::Options::default(),
    }
}

/// Translates the raw settings into what the developer takes.
pub fn raw_options(raw: &RawConfig) -> raw::Options {
    raw::Options {
        develop: raw.source == RawSource::Develop,
        demosaic: match raw.quality {
            RawQuality::Fast => raw::Demosaic::Fast,
            RawQuality::Balanced => raw::Demosaic::Balanced,
            RawQuality::Best => raw::Demosaic::Best,
        },
        camera_white_balance: raw.camera_white_balance,
        auto_brighten: raw.auto_brighten,
        highlight_mode: raw.highlight_mode,
    }
}

/// Splits the adapter budget into `(full size, thumbnails)` bytes.
///
/// The same shape as the RAM split and for the same reason: the contact sheet
/// holds many small textures and the image view a few large ones, and letting
/// either take the whole budget starves the other.
fn gpu_split(gpu_budget_mb: usize) -> (usize, usize) {
    let total = gpu_budget_mb.max(1) * 1024 * 1024;
    let thumbnails = (total / THUMBNAIL_SHARE).min(total / 2).max(1);

    (total - thumbnails, thumbnails)
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

    /// The sixteen fields a store is built from, and the reason none of them
    /// needs a restart.
    ///
    /// Every one of these is read once, when the store is built, so the way to
    /// apply a change to it is to build the store again — and the way the
    /// application knows to is that the settings it derives are no longer the
    /// settings the store is running on. A field that stopped reaching the
    /// derived settings would be a field that silently stopped taking effect,
    /// which is the whole fault this stage is about, so it is asserted here
    /// rather than left to be noticed.
    #[test]
    fn every_field_a_store_is_built_from_reaches_it() {
        use crate::config::{Config, RawQuality, RawSource};

        /// One field, and the hand that moves it.
        type Moves = (&'static str, fn(&mut Config));

        let moved: Vec<Moves> = vec![
            ("cache.ram_budget_mb", |c| c.cache.ram_budget_mb += 512),
            ("cache.gpu_budget_mb", |c| c.cache.gpu_budget_mb += 256),
            ("cache.previews_resident", |c| {
                c.cache.previews_resident += 8
            }),
            ("cache.full_resolution_neighbours", |c| {
                c.cache.full_resolution_neighbours += 1
            }),
            ("cache.upload_budget_ms", |c| c.cache.upload_budget_ms += 1),
            ("image_view.nr_loaded_images", |c| {
                c.image_view.nr_loaded_images += 1
            }),
            ("image_view.gpu_resident_images", |c| {
                c.image_view.gpu_resident_images += 1
            }),
            ("image_view.max_image_edge", |c| {
                c.image_view.max_image_edge = 4096
            }),
            ("grid_view.thumbnail_resolution", |c| {
                c.grid_view.thumbnail_resolution += 64
            }),
            ("grid_view.gpu_resident_thumbnails", |c| {
                c.grid_view.gpu_resident_thumbnails += 16
            }),
            ("grid_view.preloaded_rows", |c| {
                c.grid_view.preloaded_rows += 1
            }),
            ("grid_view.images_per_row", |c| {
                c.grid_view.images_per_row += 1
            }),
            ("raw.source", |c| c.raw.source = RawSource::Develop),
            ("raw.quality", |c| c.raw.quality = RawQuality::Best),
            ("raw.camera_white_balance", |c| {
                c.raw.camera_white_balance = !c.raw.camera_white_balance
            }),
            ("raw.auto_brighten", |c| {
                c.raw.auto_brighten = !c.raw.auto_brighten
            }),
            ("raw.highlight_mode", |c| c.raw.highlight_mode += 1),
        ];

        let was = Config::default();
        let before = (
            image_store(&was.cache, &was.image_view, &was.raw),
            thumbnail_store(&was.cache, &was.grid_view),
        );

        for (name, move_it) in moved {
            let mut now = Config::default();
            move_it(&mut now);

            let after = (
                image_store(&now.cache, &now.image_view, &now.raw),
                thumbnail_store(&now.cache, &now.grid_view),
            );

            assert!(
                after != before,
                "{name} changed and no store noticed, so it would wait for a restart"
            );
        }
    }

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
        let images = image_store(&cache, &ImageViewConfig::default(), &RawConfig::default());
        let thumbnails = thumbnail_store(&cache, &GridViewConfig::default());

        // The furthest full size image must still outrank the nearest
        // thumbnail.
        assert!(images.priority_bias + images.preload_radius < thumbnails.priority_bias);
    }

    #[test]
    fn the_grid_never_develops_raws() {
        let developing = RawConfig {
            source: RawSource::Develop,
            ..Default::default()
        };

        assert!(
            image_store(
                &CacheConfig::default(),
                &ImageViewConfig::default(),
                &developing
            )
            .raw
            .develop
        );
        assert!(
            !thumbnail_store(&CacheConfig::default(), &GridViewConfig::default())
                .raw
                .develop
        );
    }

    #[test]
    fn the_quality_setting_reaches_the_developer() {
        let best = RawConfig {
            quality: RawQuality::Best,
            ..Default::default()
        };

        assert_eq!(raw_options(&best).demosaic, raw::Demosaic::Best);
        assert_eq!(
            raw_options(&RawConfig::default()).demosaic,
            raw::Demosaic::Balanced
        );
    }

    #[test]
    fn the_two_stores_share_one_budget() {
        let cache = CacheConfig::default();
        let images = image_store(&cache, &ImageViewConfig::default(), &RawConfig::default());
        let thumbnails = thumbnail_store(&cache, &GridViewConfig::default());

        assert_eq!(
            images.ram_budget_bytes + thumbnails.ram_budget_bytes,
            cache.ram_budget_mb * 1024 * 1024
        );
    }
}
