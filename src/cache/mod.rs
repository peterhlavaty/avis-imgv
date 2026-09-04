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

pub mod focus;
pub mod gpu;
pub mod loader;
pub mod mipmap;
pub mod policy;
pub mod preview;
pub mod ram;
pub mod residency;
pub mod scanned;
pub mod store;

use std::time::Duration;

pub use store::ImageStore;

/// Budgets for one store.
///
/// Compared as a whole, so that the application can tell whether a change to
/// the configuration is one the stores have to be rebuilt for. Every field
/// here is read once, when the store is built.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StoreConfig {
    /// Ceiling on decoded pixels held in RAM.
    pub ram_budget_bytes: usize,
    /// How many textures stay resident on the GPU.
    pub gpu_resident: usize,
    /// What those textures may add up to, mip chains included.
    pub gpu_budget_bytes: usize,
    /// How many images either side of the cursor are decoded ahead of time.
    pub preload_radius: usize,
    /// Cap on the longest edge of a decoded image, before the GPU's own limit
    /// is applied.
    pub max_edge: Option<u32>,
    /// How many camera thumbnails to keep on the GPU, to stand in for images
    /// that are still being decoded. Zero turns previews off.
    pub previews_resident: usize,
    /// How far either side of the cursor images are also decoded at their own
    /// resolution, ready to be zoomed into. Zero turns that off.
    pub full_resolution_neighbours: usize,
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
            gpu_budget_bytes: 256 * 1024 * 1024,
            preload_radius: 32,
            max_edge: None,
            previews_resident: 16,
            full_resolution_neighbours: 1,
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
    /// Of those, how many are also held at their own resolution for zooming.
    pub at_full_resolution: usize,
    pub on_gpu: usize,
    /// Decoded pixels held in RAM, and the ceiling on them.
    pub resident_bytes: usize,
    pub budget_bytes: usize,
    /// What the adapter is holding, mip chains included, and its ceiling.
    ///
    /// Reported because it was not: the readout counted decoded pixels in RAM
    /// and called that the memory, while the textures — which are the same
    /// pixels again, plus a third for the mip chain — went unmentioned.
    pub gpu_bytes: usize,
    pub gpu_budget_bytes: usize,
    /// The camera thumbnails standing in for images still decoding.
    pub preview_bytes: usize,
    /// Metadata read ahead of the decoders, and its ceiling.
    pub scanned_bytes: usize,
    pub scanned_budget_bytes: usize,
    pub loading: usize,
    pub failed: usize,
}

impl StoreStats {
    /// Everything this store is holding, wherever it is holding it.
    pub fn held_bytes(&self) -> usize {
        self.resident_bytes + self.gpu_bytes + self.preview_bytes + self.scanned_bytes
    }
}
