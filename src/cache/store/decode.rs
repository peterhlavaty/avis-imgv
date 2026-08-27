//! Feeding the decode pool and moving what comes back onto the GPU.

use std::collections::HashSet;
use std::sync::Arc;
use std::time::Instant;

use super::super::loader::{ImageKey, Job, Loaded};
use super::super::policy;
use super::ImageStore;

impl ImageStore {
    /// Queues everything in the window that is neither cached nor in flight.
    pub(super) fn request_window(&mut self) {
        let window = self.window();
        if window.is_empty() {
            return;
        }

        // Previews first: they are cheap, they run on a thread of their own,
        // and they are what puts something on screen.
        self.request_previews();

        // Then the copies that make zooming worth doing. They carry their own
        // radius, so browsing at speed abandons them before they cost
        // anything.
        self.request_full();

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
            self.loader.submit(Job {
                key: ImageKey {
                    generation: self.generation,
                    index,
                },
                priority: priority + self.config.priority_bias,
                radius: None,
                path: path.clone(),
                options: self.options.clone(),
                focus: Arc::clone(&self.focus),
                responder: self.responder.clone(),
            });
        }
    }

    /// Moves finished decodes into the RAM cache.
    pub(super) fn collect_results(&mut self) -> bool {
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
    pub(super) fn upload_window(&mut self) -> bool {
        let total = self.paths.len();
        if total == 0 {
            return false;
        }

        // Nearest first, so the per-frame budget is spent where it shows.
        //
        // One short of the capacity, not half of it: a window of exactly
        // as many textures as the cache holds evicts one to make room for
        // the next and then wants the evicted one back, and the uploads
        // never stop.
        let wanted = policy::window(self.cursor, total, (self.gpu.capacity() - 1) / 2);
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
