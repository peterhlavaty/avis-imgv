//! The store itself: what is on disk, what is decoded, what is on the GPU.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;

use eframe::egui_wgpu::RenderState;

use crate::decoder::DecodeOptions;
use crate::metadata::Metadata;

use super::gpu::{GpuCache, GpuTexture};
use super::loader::{Focus, ImageKey, LoadResult, Loader};
use super::policy;
use super::ram::RamCache;
use super::{ImageState, StoreConfig, StoreStats};

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
}

impl ImageStore {
    /// Creates a store sharing `loader` with any other stores in the app.
    pub fn new(
        render_state: RenderState,
        loader: Arc<Loader>,
        config: StoreConfig,
        output_profile: Arc<str>,
    ) -> ImageStore {
        let gpu = GpuCache::new(render_state, config.gpu_resident);
        let (responder, results) = channel();

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
            options: DecodeOptions::new(output_profile).with_max_edge(max_edge),
            ram: RamCache::new(config.ram_budget_bytes),
            gpu,
            loader,
            focus: Arc::new(Focus::default()),
            results,
            responder,
            requested: HashSet::new(),
            failed: HashSet::new(),
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
        self.ram.clear();
        self.gpu.clear();
        self.requested.clear();
        self.failed.clear();

        // Drain results belonging to the previous collection.
        while self.results.try_recv().is_ok() {}

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

    /// One frame's worth of cache maintenance: collect finished work, queue
    /// what is missing, and move decoded images onto the GPU.
    ///
    /// Returns `true` when something became drawable, so the caller can ask for
    /// a repaint.
    pub fn tick(&mut self) -> bool {
        let collected = self.collect_results();
        self.request_window();
        let uploaded = self.upload_window();

        collected || uploaded
    }

    /// State of one image, for the UI to decide between drawing and waiting.
    pub fn state(&self, index: usize) -> ImageState {
        if self.gpu.contains(index) {
            ImageState::Ready
        } else if self.ram.contains(index) {
            ImageState::Decoded
        } else if self.failed.contains(&index) {
            ImageState::Failed
        } else {
            ImageState::Loading
        }
    }

    /// The texture for `index`, if one is resident.
    pub fn texture(&self, index: usize) -> Option<&GpuTexture> {
        self.gpu.get(index)
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

        self.gpu.get(index)
    }

    pub fn metadata(&self, index: usize) -> Option<&Metadata> {
        self.ram.get(index).map(|image| &image.metadata)
    }

    /// Forgets everything cached about one image so it is decoded again.
    pub fn reload(&mut self, index: usize) {
        self.ram.remove(index);
        self.gpu.remove(index);
        self.requested.remove(&index);
        self.failed.remove(&index);
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
        self.requested = shift_indices(&self.requested, index);
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

    /// Queues everything in the window that is neither cached nor in flight.
    fn request_window(&mut self) {
        let window = self.window();
        if window.is_empty() {
            return;
        }

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
                Ok(image) => {
                    self.ram
                        .insert(index, Arc::new(image), self.cursor, self.paths.len());
                    collected = true;
                }
                Err(_) => {
                    // The worker already logged the reason.
                    self.failed.insert(index);
                    collected = true;
                }
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

        let mut uploaded = 0;
        for index in wanted {
            if uploaded >= self.config.uploads_per_frame {
                break;
            }
            if self.gpu.contains(index) {
                continue;
            }

            let Some(image) = self.ram.get(index).cloned() else {
                continue;
            };

            self.gpu.upload(index, &image, self.cursor, total);
            uploaded += 1;
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
        assert!(config.uploads_per_frame > 0);
    }
}
