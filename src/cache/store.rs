//! The store itself: what is on disk, what is decoded, what is on the GPU.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use std::time::Instant;

use eframe::egui_wgpu::RenderState;

use crate::decoder::DecodeOptions;
use crate::metadata::Metadata;

use super::gpu::{GpuCache, GpuTexture};
use super::loader::{Focus, ImageKey, LoadResult, Loaded, Loader};
use super::policy;
use super::preview::{self, PreviewLoader};
use super::ram::RamCache;
use super::{ImageState, StoreConfig, StoreStats};

/// Granularity of the screen size reported to the decoders.
///
/// Coarse on purpose: a resized window should not invalidate what is already
/// decoded, and half a step of extra resolution costs almost nothing.
const DISPLAY_EDGE_STEP: u32 = 512;

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
    /// The camera's thumbnails, which stand in for images still being decoded.
    previews: GpuCache,
    preview_loader: PreviewLoader,
    preview_results: Receiver<preview::Read>,
    preview_responder: Sender<preview::Read>,
    preview_requested: HashSet<usize>,
    /// Metadata read from the front of each file, available long before the
    /// image itself is.
    scanned: HashMap<PathBuf, Metadata>,
}

impl ImageStore {
    /// Creates a store sharing `loader` with any other stores in the app.
    pub fn new(
        render_state: RenderState,
        loader: Arc<Loader>,
        config: StoreConfig,
        output_profile: Arc<str>,
    ) -> ImageStore {
        let gpu = GpuCache::new(render_state.clone(), config.gpu_resident);
        let previews = GpuCache::new(render_state, config.previews_resident.max(1));
        let (responder, results) = channel();
        let (preview_responder, preview_results) = preview::channel_pair();

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
            ram: RamCache::new(config.ram_budget_bytes),
            gpu,
            loader,
            focus: Arc::new(Focus::default()),
            results,
            responder,
            requested: HashSet::new(),
            failed: HashSet::new(),
            previews,
            preview_loader: PreviewLoader::new(),
            preview_results,
            preview_responder,
            preview_requested: HashSet::new(),
            scanned: HashMap::new(),
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
        self.paths = paths;
        self.cursor = 0;

        self.loader.clear();
        self.preview_loader.clear();
        self.ram.clear();
        self.gpu.clear();
        self.previews.clear();
        self.requested.clear();
        self.preview_requested.clear();
        self.failed.clear();

        // Drain results belonging to the previous collection.
        while self.results.try_recv().is_ok() {}
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

    /// Replaces the resident texture for `index` with the image at its own
    /// resolution.
    ///
    /// Called when the view notices it is drawing an image larger than what
    /// has been uploaded. Does nothing if the full resolution is already up,
    /// so it is safe to call every frame.
    pub fn upload_full(&mut self, index: usize) {
        if self.gpu.get(index).is_some_and(|texture| texture.is_full()) {
            return;
        }

        let Some(image) = self.ram.get(index).cloned() else {
            return;
        };

        tracing::debug!("{} -> uploading at full resolution", image.file_name());
        self.gpu
            .upload_full(index, &image, self.cursor, self.paths.len());
        self.previews.remove(index);
    }

    /// One frame's worth of cache maintenance: collect finished work, queue
    /// what is missing, and move decoded images onto the GPU.
    ///
    /// Returns `true` when something became drawable, so the caller can ask for
    /// a repaint.
    pub fn tick(&mut self) -> bool {
        let collected = self.collect_results();
        let scanned = self.collect_previews();
        self.request_window();
        let uploaded = self.upload_window();

        collected || scanned || uploaded
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
        self.gpu.remove(index);
        self.previews.remove(index);
        self.requested.remove(&index);
        self.preview_requested.remove(&index);
        self.failed.remove(&index);

        if let Some(path) = self.paths.get(index) {
            self.scanned.remove(path);
        }
    }

    /// Removes an image from the collection, keeping the caches aligned with
    /// the new positions.
    pub fn remove(&mut self, index: usize) {
        if index >= self.paths.len() {
            return;
        }

        self.paths.remove(index);
        self.ram.remove_shifting(index);
        self.gpu.remove_shifting(index);
        self.previews.remove_shifting(index);
        self.requested = shift_indices(&self.requested, index);
        self.preview_requested = shift_indices(&self.preview_requested, index);
        self.failed = shift_indices(&self.failed, index);

        self.focus.set_collection(self.generation, self.paths.len());
        self.set_cursor(self.cursor);
    }

    pub fn stats(&self) -> StoreStats {
        StoreStats {
            total: self.paths.len(),
            in_ram: self.ram.len(),
            on_gpu: self.gpu.len(),
            resident_bytes: self.ram.resident_bytes(),
            budget_bytes: self.ram.budget_bytes(),
            loading: self.requested.len(),
            failed: self.failed.len(),
        }
    }

    fn window(&self) -> Vec<usize> {
        policy::window(self.cursor, self.paths.len(), self.effective_radius())
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

    /// The images a thumbnail is worth reading for.
    ///
    /// Much narrower than the decode window: a thumbnail only earns its keep
    /// in the moment between an image being asked for and its decode landing,
    /// and reading the front of every file in a wide window would take disk
    /// bandwidth away from the decoders that actually need it.
    fn preview_window(&self) -> Vec<usize> {
        policy::window(
            self.cursor,
            self.paths.len(),
            self.config.previews_resident / 2,
        )
    }

    /// Reads the front of the files around the cursor, which gives their
    /// metadata and a thumbnail long before a decoder gets to them.
    fn request_previews(&mut self) {
        if self.config.previews_resident == 0 {
            return;
        }

        for index in &self.preview_window() {
            // Once the real image is decoded a thumbnail is no longer wanted.
            if self.preview_requested.contains(index) || self.ram.contains(*index) {
                continue;
            }

            let Some(path) = self.paths.get(*index) else {
                continue;
            };

            self.preview_requested.insert(*index);
            self.preview_loader.submit(
                ImageKey {
                    generation: self.generation,
                    index: *index,
                },
                path.clone(),
                self.preview_responder.clone(),
            );
        }
    }

    /// Takes in the previews that have been read.
    fn collect_previews(&mut self) -> bool {
        let mut collected = false;
        let total = self.paths.len();

        while let Ok(read) = self.preview_results.try_recv() {
            if read.key.generation != self.generation {
                continue;
            }

            let index = read.key.index;
            if let Some(path) = self.paths.get(index) {
                self.scanned
                    .insert(path.clone(), read.preview.metadata.clone());
            }

            // The real image may have arrived while this was being read.
            if read.preview.has_image() && !self.gpu.contains(index) {
                self.previews
                    .upload_preview(index, &read.preview, self.cursor, total);
            }

            collected = true;
        }

        collected
    }

    /// Queues everything in the window that is neither cached nor in flight.
    fn request_window(&mut self) {
        let window = self.window();
        if window.is_empty() {
            return;
        }

        // Previews first: they are cheap, they run on a thread of their own,
        // and they are what puts something on screen.
        self.request_previews();

        // Requests decoded past the window are dropped by the workers.
        self.focus.set_position(self.cursor, window.len());

        for (priority, index) in window.into_iter().enumerate() {
            if self.ram.contains(index)
                || self.requested.contains(&index)
                || self.failed.contains(&index)
            {
                continue;
            }

            let Some(path) = self.paths.get(index) else {
                continue;
            };

            self.requested.insert(index);
            self.loader.submit(
                ImageKey {
                    generation: self.generation,
                    index,
                },
                priority + self.config.priority_bias,
                path.clone(),
                self.options.clone(),
                Arc::clone(&self.focus),
                self.responder.clone(),
            );
        }
    }

    /// Moves finished decodes into the RAM cache.
    fn collect_results(&mut self) -> bool {
        let mut collected = false;

        while let Ok(result) = self.results.try_recv() {
            if result.key.generation != self.generation {
                continue;
            }

            let index = result.key.index;
            self.requested.remove(&index);

            match result.outcome {
                Loaded::Decoded(image) => {
                    self.ram
                        .insert(index, Arc::new(image), self.cursor, self.paths.len());
                    collected = true;
                }
                Loaded::Failed(_) => {
                    // The worker already logged the reason.
                    self.failed.insert(index);
                    collected = true;
                }
                // Nothing was decoded, and taking it out of `requested` above
                // is the whole point: the image can be asked for again if it
                // is still wanted.
                Loaded::Abandoned => {}
            }
        }

        collected
    }

    /// Uploads the nearest decoded images that are not yet resident, within
    /// this frame's budget.
    fn upload_window(&mut self) -> bool {
        let total = self.paths.len();
        if total == 0 {
            return false;
        }

        // Nearest first, so the per-frame budget is spent where it shows.
        let wanted = policy::window(self.cursor, total, self.gpu.capacity() / 2);
        let resident: HashSet<usize> = wanted.iter().copied().collect();

        // Textures outside the window are dropped so capacity is spent on what
        // the user is about to see rather than on where they have been.
        self.gpu.retain(|index| resident.contains(&index));

        // Thumbnails are kept over the same narrow window they are read for.
        let previewed: HashSet<usize> = self.preview_window().into_iter().collect();
        self.previews.retain(|index| previewed.contains(&index));

        let started = Instant::now();
        let mut uploaded = 0;

        for index in wanted {
            if self.gpu.contains(index) {
                continue;
            }

            let Some(image) = self.ram.get(index).cloned() else {
                continue;
            };

            self.gpu.upload(index, &image, self.cursor, total);
            // The thumbnail has been superseded; its texture is dead weight.
            self.previews.remove(index);
            uploaded += 1;

            // Always upload one, so a budget smaller than a single image
            // still makes progress, and stop once the frame has spent enough.
            if started.elapsed() >= self.config.upload_budget {
                break;
            }
        }

        uploaded > 0
    }
}

/// Shifts a set of positions down past a removed index.
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
