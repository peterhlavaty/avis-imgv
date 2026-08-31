//! The store itself: what is on disk, what is decoded, what is on the GPU.

mod decode;
mod detail;
mod previews;

pub use detail::Detail;

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;

use eframe::egui_wgpu::RenderState;

use crate::decoder::DecodeOptions;
use crate::metadata::Metadata;

use super::gpu::{GpuCache, GpuTexture};
use super::loader::{Focus, LoadResult, Loaded, Loader};
use super::policy;
use super::preview::{self, PreviewLoader};
use super::ram::RamCache;
use super::scanned::Scanned;
use super::{ImageState, StoreConfig, StoreStats};

/// Granularity of the screen size reported to the decoders.
///
/// Coarse on purpose: a resized window should not invalidate what is already
/// decoded, and half a step of extra resolution costs almost nothing.
const DISPLAY_EDGE_STEP: u32 = 512;

/// Share of the RAM budget spent on full resolution copies.
///
/// The screen sized copies are the point of the budget — they are what lets a
/// whole folder stay resident — so the copies kept for zooming get a slice
/// rather than a half. A quarter of four gigabytes still holds several 24
/// megapixel images, which is more than the three that are always wanted.
const FULL_RESOLUTION_SHARE: usize = 4;

/// Share of the adapter budget the camera thumbnails may take.
///
/// A small one: there are many of them, each is a few hundred kilobytes, and
/// they exist only to stand in for a photograph that is still decoding.
const PREVIEW_GPU_SHARE: usize = 8;

/// Share of the RAM budget the metadata read ahead may take.
///
/// Small: a folder's worth of tags is a few megabytes, and it is worth keeping
/// because reading it again means going back to the disk — but it must not be
/// able to crowd out the photographs it describes.
const SCANNED_SHARE: usize = 64;

/// One collection of images and everything cached about it.
pub struct ImageStore {
    paths: Vec<PathBuf>,
    /// Bumped on every collection change so in-flight work can be discarded.
    generation: u64,
    cursor: usize,
    config: StoreConfig,
    options: DecodeOptions,
    ram: RamCache,
    gpu: GpuCache,
    loader: Arc<Loader>,
    focus: Arc<Focus>,
    results: Receiver<LoadResult>,
    responder: Sender<LoadResult>,
    requested: HashSet<usize>,
    failed: HashSet<usize>,
    /// The images near the cursor at their own resolution, ready to be zoomed
    /// into. Bounded by its own slice of the budget, so an image the user
    /// zoomed into and left behind stays until something nearer needs the
    /// room.
    full: RamCache,
    full_results: Receiver<LoadResult>,
    full_responder: Sender<LoadResult>,
    full_requested: HashSet<usize>,
    /// The camera's thumbnails, which stand in for images still being decoded.
    previews: GpuCache,
    preview_loader: PreviewLoader,
    preview_results: Receiver<preview::Read>,
    preview_responder: Sender<preview::Read>,
    preview_requested: HashSet<usize>,
    /// Metadata read from the front of each file, available long before the
    /// image itself is, and bounded like everything else here.
    scanned: Scanned,
    /// The four preload windows, kept between frames.
    ///
    /// Each is wanted once a frame and is the same as last frame's for as long
    /// as nobody is navigating; recomputing all four every frame for every
    /// store was work done to arrive back where it started.
    windows: Windows,
}

/// The windows a store keeps: what to decode, what to upload, what to read the
/// front of, and what to hold at full resolution.
#[derive(Debug, Default)]
struct Windows {
    decode: policy::Window,
    upload: policy::Window,
    previews: policy::Window,
    full: policy::Window,
}

impl Windows {
    /// Forgets all four, for when the collection has changed underneath them.
    fn forget(&mut self) {
        self.decode.forget();
        self.upload.forget();
        self.previews.forget();
        self.full.forget();
    }
}

impl ImageStore {
    /// Creates a store sharing `loader` with any other stores in the app.
    pub fn new(
        render_state: RenderState,
        loader: Arc<Loader>,
        config: StoreConfig,
        output_profile: Arc<str>,
    ) -> ImageStore {
        // Cloned before the options take ownership: the preview worker
        // converts the camera's thumbnail into the same profile the decoders
        // convert the photograph into.
        let preview_profile = Arc::clone(&output_profile);

        // The thumbnails that stand in for images still decoding get a small
        // slice: there are a lot of them and each is a few hundred kilobytes,
        // and taking room from the photographs to hold more of them would be
        // the wrong way round.
        let preview_budget = (config.gpu_budget_bytes / PREVIEW_GPU_SHARE).max(1);
        let gpu = GpuCache::new(
            render_state.clone(),
            config.gpu_resident,
            config
                .gpu_budget_bytes
                .saturating_sub(preview_budget)
                .max(1),
        );
        let previews = GpuCache::new(
            render_state,
            config.previews_resident.max(1),
            preview_budget,
        );
        let (responder, results) = channel();
        let (full_responder, full_results) = channel();
        let (preview_responder, preview_results) = preview::channel_pair();

        // A share of the RAM budget rather than a number of entries: one
        // photograph's tags may be a few hundred bytes and another's a
        // two-kilobyte colour profile, and this used to be held outside the
        // budget entirely and never released.
        let scanned_budget = (config.ram_budget_bytes / SCANNED_SHARE).max(1);

        let full_budget = if config.full_resolution_neighbours > 0 {
            config.ram_budget_bytes / FULL_RESOLUTION_SHARE
        } else {
            0
        };

        // A texture larger than the adapter allows cannot be uploaded at all,
        // so the hardware limit always wins over the configured one.
        let max_edge = Some(match config.max_edge {
            Some(configured) => configured.min(gpu.max_texture_edge()),
            None => gpu.max_texture_edge(),
        });

        ImageStore {
            paths: Vec::new(),
            generation: 0,
            cursor: 0,
            options: DecodeOptions::new(output_profile)
                .with_max_edge(max_edge)
                .with_raw(config.raw),
            ram: RamCache::new(config.ram_budget_bytes - full_budget),
            gpu,
            loader,
            focus: Arc::new(Focus::default()),
            results,
            responder,
            requested: HashSet::new(),
            failed: HashSet::new(),
            full: RamCache::new(full_budget),
            full_results,
            full_responder,
            full_requested: HashSet::new(),
            previews,
            preview_loader: PreviewLoader::new(Arc::clone(&preview_profile)),
            preview_results,
            preview_responder,
            preview_requested: HashSet::new(),
            windows: Windows::default(),
            scanned: Scanned::new(scanned_budget),
            config,
        }
    }

    pub fn len(&self) -> usize {
        self.paths.len()
    }

    pub fn is_empty(&self) -> bool {
        self.paths.is_empty()
    }

    pub fn path(&self, index: usize) -> Option<&Path> {
        self.paths.get(index).map(PathBuf::as_path)
    }

    pub fn index_of(&self, path: &Path) -> Option<usize> {
        self.paths.iter().position(|candidate| candidate == path)
    }

    /// Replaces the collection, discarding everything cached about the old one.
    pub fn set_paths(&mut self, paths: Vec<PathBuf>) {
        self.generation += 1;

        // A different collection of the same length is a different collection.
        // The kept windows are keyed by cursor, size and radius, none of which
        // has to change when a folder is read again — so they are told rather
        // than left to notice.
        self.windows.forget();
        self.paths = paths;
        self.cursor = 0;

        self.loader.clear();
        self.preview_loader.clear();
        self.ram.clear();
        self.full.clear();
        self.gpu.clear();
        self.previews.clear();
        self.requested.clear();
        self.full_requested.clear();
        self.preview_requested.clear();
        self.failed.clear();

        // Drain results belonging to the previous collection.
        while self.results.try_recv().is_ok() {}
        while self.full_results.try_recv().is_ok() {}
        while self.preview_results.try_recv().is_ok() {}

        self.focus.set_collection(self.generation, self.paths.len());
        tracing::info!("Store now holds {} images", self.paths.len());
    }

    /// Moves the point everything is cached around.
    pub fn set_cursor(&mut self, cursor: usize) {
        self.cursor = if self.paths.is_empty() {
            0
        } else {
            cursor.min(self.paths.len() - 1)
        };
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }

    /// Tells the store how many pixels the image can actually be shown across,
    /// so decoders can prepare a copy that size instead of a full one.
    ///
    /// Only ever raised, and in coarse steps: the value travels with every
    /// decode request, and letting it follow a window being dragged would make
    /// every image in flight the wrong size.
    pub fn set_display_edge(&mut self, edge: u32) {
        let wanted = edge.div_ceil(DISPLAY_EDGE_STEP) * DISPLAY_EDGE_STEP;

        if self
            .options
            .display_edge
            .is_none_or(|current| current < wanted)
        {
            self.options.display_edge = Some(wanted);
        }
    }

    /// One frame's worth of cache maintenance: collect finished work, queue
    /// what is missing, and move decoded images onto the GPU.
    ///
    /// Returns `true` when something became drawable, so the caller can ask for
    /// a repaint.
    pub fn tick(&mut self) -> bool {
        let collected = self.collect_results();
        let detailed = self.collect_full();
        let scanned = self.collect_previews();
        self.request_window();
        let uploaded = self.upload_window();

        collected || detailed || scanned || uploaded
    }

    /// State of one image, for the UI to decide between drawing and waiting.
    pub fn state(&self, index: usize) -> ImageState {
        if self.gpu.contains(index) {
            ImageState::Ready
        } else if self.ram.contains(index) {
            ImageState::Decoded
        } else if self.failed.contains(&index) {
            ImageState::Failed
        } else if self.previews.contains(index) {
            ImageState::Previewed
        } else {
            ImageState::Loading
        }
    }

    /// The texture for `index`, falling back to the camera's thumbnail.
    ///
    /// The thumbnail reports the size of the image it stands for, so the
    /// layout is already right and nothing moves when the real one lands.
    pub fn texture(&self, index: usize) -> Option<&GpuTexture> {
        self.gpu.get(index).or_else(|| self.previews.get(index))
    }

    /// Uploads `index` right now if it is decoded but not yet resident.
    ///
    /// Used for the image the user is looking at, which must not wait for its
    /// turn in the per-frame upload budget.
    pub fn texture_now(&mut self, index: usize) -> Option<&GpuTexture> {
        if !self.gpu.contains(index) {
            if let Some(image) = self.ram.get(index).cloned() {
                self.gpu
                    .upload(index, &image, self.cursor, self.paths.len());
            }
        }

        self.texture(index)
    }

    /// What is known about an image: from the decoded one if it is there, and
    /// from the front of the file if it is not.
    ///
    /// The second arrives within a couple of milliseconds of the image being
    /// asked for, so the side panel is never blank for long.
    /// How this photograph's tones are distributed, once it is decoded.
    ///
    /// From the decode rather than computed here: the worker already had the
    /// pixels, and the UI thread should not walk twenty-four million of them
    /// to draw a curve two hundred pixels wide.
    pub fn histogram(&self, index: usize) -> Option<&crate::decoder::histogram::Histogram> {
        Some(&self.ram.get(index)?.histogram)
    }

    pub fn metadata(&self, index: usize) -> Option<&Metadata> {
        if let Some(image) = self.ram.get(index) {
            return Some(&image.metadata);
        }

        self.scanned.get(self.paths.get(index)?)
    }

    /// Metadata from the whole file, rather than from the front of it.
    ///
    /// Anything that is going to be written back has to wait for this: a
    /// sidecar seeded from a truncated read would drop what it could not see.
    pub fn decoded_metadata(&self, index: usize) -> Option<&Metadata> {
        self.ram.get(index).map(|image| &image.metadata)
    }

    /// Forgets everything cached about one image so it is decoded again.
    pub fn reload(&mut self, index: usize) {
        self.ram.remove(index);
        self.full.remove(index);
        self.gpu.remove(index);
        self.previews.remove(index);
        self.requested.remove(&index);
        self.full_requested.remove(&index);
        self.preview_requested.remove(&index);
        self.failed.remove(&index);

        if let Some(path) = self.paths.get(index) {
            self.scanned.remove(path);
        }
    }

    /// Removes an image from the collection, keeping the caches aligned with
    /// the new positions.
    ///
    /// The generation is bumped and the in-flight work discarded, because
    /// every position past `index` has just moved: a decode already on its way
    /// back would land one place along and be drawn under its neighbour's
    /// name, metadata and rating. The caches themselves are shifted rather
    /// than cleared, so what is already decoded stays decoded.
    pub fn remove(&mut self, index: usize) {
        if index >= self.paths.len() {
            return;
        }

        self.paths.remove(index);
        self.ram.remove_shifting(index);
        self.full.remove_shifting(index);
        self.gpu.remove_shifting(index);
        self.previews.remove_shifting(index);

        self.generation += 1;
        self.loader.clear();
        self.preview_loader.clear();
        self.requested.clear();
        self.full_requested.clear();
        self.preview_requested.clear();
        self.failed = shift_indices(&self.failed, index);

        // Answers to questions asked about the old positions.
        while self.results.try_recv().is_ok() {}
        while self.full_results.try_recv().is_ok() {}
        while self.preview_results.try_recv().is_ok() {}

        self.focus.set_collection(self.generation, self.paths.len());
        self.set_cursor(self.cursor);
    }

    /// Adds an image at `index`, keeping the caches aligned with the new
    /// positions.
    ///
    /// The counterpart of [`ImageStore::remove`], and it matters for the same
    /// reason: a file appearing in a watched folder used to mean reading the
    /// whole folder again, which threw away every decoded photograph and every
    /// thumbnail in it. A tethered shoot did that once a frame.
    ///
    /// The generation is bumped for the same reason a removal bumps it — every
    /// position at or past `index` has just moved, so a decode already on its
    /// way back would land one place along and be drawn under its neighbour's
    /// name, metadata and rating.
    pub fn insert(&mut self, index: usize, path: PathBuf) {
        let index = index.min(self.paths.len());

        self.paths.insert(index, path);
        self.ram.insert_shifting(index);
        self.full.insert_shifting(index);
        self.gpu.insert_shifting(index);
        self.previews.insert_shifting(index);

        self.generation += 1;
        self.loader.clear();
        self.preview_loader.clear();
        self.requested.clear();
        self.full_requested.clear();
        self.preview_requested.clear();
        self.failed = raise_indices(&self.failed, index);

        // Answers to questions asked about the old positions.
        while self.results.try_recv().is_ok() {}
        while self.full_results.try_recv().is_ok() {}
        while self.preview_results.try_recv().is_ok() {}

        self.focus.set_collection(self.generation, self.paths.len());
        self.set_cursor(self.cursor);
    }

    pub fn stats(&self) -> StoreStats {
        StoreStats {
            total: self.paths.len(),
            in_ram: self.ram.len(),
            at_full_resolution: self.full.len(),
            on_gpu: self.gpu.len(),
            resident_bytes: self.ram.resident_bytes() + self.full.resident_bytes(),
            budget_bytes: self.ram.budget_bytes() + self.full.budget_bytes(),
            gpu_bytes: self.gpu.resident_bytes(),
            gpu_budget_bytes: self.gpu.budget_bytes(),
            preview_bytes: self.previews.resident_bytes(),
            scanned_bytes: self.scanned.resident_bytes(),
            scanned_budget_bytes: self.scanned.budget_bytes(),
            loading: self.requested.len(),
            failed: self.failed.len(),
        }
    }
    /// Preload radius trimmed to what the budget can actually hold.
    fn effective_radius(&self) -> usize {
        policy::budgeted_radius(
            self.config.preload_radius,
            self.ram.budget_bytes(),
            self.ram.resident_bytes(),
            self.ram.len(),
        )
    }
}

/// Shifts a set of positions down past a removed index.
/// The same, for a photograph that has appeared.
fn raise_indices(indices: &HashSet<usize>, added: usize) -> HashSet<usize> {
    indices
        .iter()
        .map(|index| if *index >= added { index + 1 } else { *index })
        .collect()
}

fn shift_indices(indices: &HashSet<usize>, removed: usize) -> HashSet<usize> {
    indices
        .iter()
        .filter(|index| **index != removed)
        .map(|index| if *index > removed { index - 1 } else { *index })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shifting_drops_the_removed_index() {
        let indices = HashSet::from([0, 1, 2, 5]);
        let shifted = shift_indices(&indices, 1);

        assert_eq!(shifted, HashSet::from([0, 1, 4]));
    }

    #[test]
    fn shifting_leaves_lower_indices_alone() {
        let indices = HashSet::from([0, 1]);
        assert_eq!(shift_indices(&indices, 5), HashSet::from([0, 1]));
    }

    #[test]
    fn the_default_config_is_usable() {
        let config = StoreConfig::default();
        assert!(config.ram_budget_bytes > 0);
        assert!(config.gpu_resident > 0);
        assert!(!config.upload_budget.is_zero());
    }
}
