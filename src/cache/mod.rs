//! The image store: one folder's worth of images, kept as close to the GPU as
//! the configured budgets allow.
//!
//! Three tiers, from cheap to instant:
//!
//! 1. paths on disk,
//! 2. decoded RGBA8 in RAM, produced by background workers ahead of the user,
//! 3. textures resident on the GPU for the images around the cursor.
//!
//! Drawing only ever reads tier three, so a frame never waits on I/O or on a
//! decoder.

pub mod gpu;
pub mod loader;
pub mod policy;
pub mod preview;
pub mod ram;
pub mod store;

use std::time::Duration;

pub use store::ImageStore;

/// Budgets for one store.
#[derive(Debug, Clone, Copy)]
pub struct StoreConfig {
    /// Ceiling on decoded pixels held in RAM.
    pub ram_budget_bytes: usize,
    /// How many textures stay resident on the GPU.
    pub gpu_resident: usize,
    /// How many images either side of the cursor are decoded ahead of time.
    pub preload_radius: usize,
    /// Cap on the longest edge of a decoded image, before the GPU's own limit
    /// is applied.
    pub max_edge: Option<u32>,
    /// How many camera thumbnails to keep on the GPU, to stand in for images
    /// that are still being decoded. Zero turns previews off.
    pub previews_resident: usize,
    /// How long one frame may spend moving images onto the GPU.
    ///
    /// A budget rather than a count, because the cost is the size of the
    /// image: a 24 megapixel texture takes about 12ms to upload, so uploading
    /// four of them would cost fifty milliseconds and drop a smooth sixty
    /// frames a second to twenty.
    pub upload_budget: Duration,
    /// Added to every request's priority.
    ///
    /// The decode pool is shared, so this is how a store that is merely
    /// warming up in the background yields to the one the user is looking at.
    pub priority_bias: usize,
    /// What to do with camera raw files.
    pub raw: crate::decoder::raw::Options,
}

impl Default for StoreConfig {
    fn default() -> Self {
        StoreConfig {
            ram_budget_bytes: 4 * 1024 * 1024 * 1024,
            gpu_resident: 8,
            preload_radius: 32,
            max_edge: None,
            previews_resident: 16,
            upload_budget: Duration::from_millis(8),
            priority_bias: 0,
            raw: crate::decoder::raw::Options::default(),
        }
    }
}

/// What the store can tell the UI about one image.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageState {
    /// A texture is ready to draw.
    Ready,
    /// The camera's thumbnail is showing while the real image is decoded.
    Previewed,
    /// Decoded and waiting for its turn to be uploaded.
    Decoded,
    /// Queued or being decoded.
    Loading,
    /// Tried and failed; it will not be retried until reloaded.
    Failed,
}

/// How full the store is, for the metrics overlay.
#[derive(Debug, Clone, Copy)]
pub struct StoreStats {
    pub total: usize,
    pub in_ram: usize,
    pub on_gpu: usize,
    pub resident_bytes: usize,
    pub budget_bytes: usize,
    pub loading: usize,
    pub failed: usize,
}
