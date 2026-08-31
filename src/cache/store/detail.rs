//! Full resolution residency: the copies that make zooming look like the
//! photograph rather than like a screenshot of it.
//!
//! Browsing keeps one screen sized copy of every image in the folder, which is
//! all a monitor can show and a tenth of the memory. That copy stops being
//! enough the moment the user magnifies past it, so the images around the
//! cursor are also decoded at their own resolution and held ready. Which of
//! the two is on the GPU follows how wide the image is actually being drawn.

use std::sync::Arc;

use super::super::loader::{ImageKey, Job};
use super::ImageStore;

/// Where a full resolution decode sits in the queue.
///
/// Behind the screen sized copies of the images within reach, which are what
/// browsing waits on, and ahead of the rest of the preload window.
const PRIORITY: usize = 4;

/// Which copy of an image should be on the GPU.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Detail {
    /// The copy made to fit the screen.
    Screen,
    /// The image's own pixels.
    Full,
}

/// Magnification past which the screen sized copy is being stretched.
const GROW_ABOVE: f32 = 1.05;

/// Magnification below which it is enough again.
///
/// Lower than [`GROW_ABOVE`] on purpose: a zoom that comes to rest near the
/// boundary would otherwise swap the texture on every frame.
const SHRINK_BELOW: f32 = 0.95;

/// Picks the copy to keep resident.
///
/// `covered` is how wide the screen sized copy can be drawn before it starts
/// being magnified, and `drawn` is how wide it is actually being drawn. Inside
/// the band between the two thresholds whatever is already up stays up.
pub fn wanted(resident: Detail, covered: f32, drawn: f32) -> Detail {
    if covered <= 0.0 {
        return resident;
    }

    match drawn / covered {
        ratio if ratio > GROW_ABOVE => Detail::Full,
        ratio if ratio < SHRINK_BELOW => Detail::Screen,
        _ => resident,
    }
}

/// Whether an image has two copies to choose between at all.
///
/// An image no larger than the screen was never reduced, so its screen sized
/// copy *is* the image. Swapping between the two would upload the same pixels
/// again — and again every frame, each time freeing the texture the frame had
/// already drawn with, which leaves the image invisible.
pub fn has_two_copies(screen_resolution: f32) -> bool {
    screen_resolution < 1.0
}

impl ImageStore {
    /// Tells the store how wide `index` is being drawn, in the pixels the
    /// screen actually has.
    ///
    /// This is the whole of the zoom policy: below the screen sized copy's own
    /// width it stays resident, above it the image's own pixels go up instead.
    /// Safe to call every frame — it only acts when the answer changes.
    pub fn set_drawn_width(&mut self, index: usize, drawn: f32) {
        let screen = self.screen_resolution(index);
        if !has_two_copies(screen) {
            return;
        }

        let Some(texture) = self.gpu.get(index) else {
            return;
        };

        let resident = if texture.is_full() {
            Detail::Full
        } else {
            Detail::Screen
        };

        // The screen sized copy drawn at this zoom, in the same pixels.
        let covered = texture.size.x * screen;

        match wanted(resident, covered, drawn) {
            Detail::Full if resident != Detail::Full => self.upload_full(index),
            Detail::Screen if resident != Detail::Screen => self.upload_screen(index),
            _ => {}
        }
    }

    /// How much of an image the screen sized copy holds, or one when there is
    /// no copy to compare against.
    fn screen_resolution(&self, index: usize) -> f32 {
        self.ram
            .get(index)
            .map_or(1.0, |image| image.resolution().min(1.0))
    }

    /// Puts the image's own pixels on the GPU, if they are held.
    ///
    /// Does nothing while the decode is still running; the screen sized copy
    /// stays up until then, so zooming is soft for a moment rather than empty.
    fn upload_full(&mut self, index: usize) {
        let Some(image) = self.full.get(index).cloned() else {
            return;
        };

        tracing::debug!("{} -> uploading at full resolution", image.file_name());
        self.gpu
            .upload(index, &image, self.cursor, self.paths.len());
        self.previews.remove(index);
    }

    /// Puts the screen sized copy back, once the image is no longer magnified.
    ///
    /// Not for quality — the mip chain handles that — but for room: a hundred
    /// megabyte texture is a tenth of what the GPU cache is allowed to hold,
    /// and holding it for an image being shown small starves the ones the user
    /// is about to reach.
    fn upload_screen(&mut self, index: usize) {
        let Some(image) = self.ram.get(index).cloned() else {
            return;
        };

        // Never replace a texture with the same pixels: the one on the GPU is
        // the one this frame has already been drawn with, and freeing it mid
        // frame leaves nothing to draw.
        if self
            .gpu
            .get(index)
            .is_some_and(|texture| texture.resolution >= image.resolution())
        {
            return;
        }

        self.gpu
            .upload(index, &image, self.cursor, self.paths.len());
    }

    /// Decodes the images within reach at their own resolution.
    ///
    /// The radius on the job is what makes this affordable: browsing at speed
    /// abandons these before a worker starts them, so the cost is only paid
    /// once the user settles on something.
    pub(super) fn request_full(&mut self) {
        if self.config.full_resolution_neighbours == 0 {
            return;
        }

        // Handed over so the loop can change the store it came from.
        let radius = self.config.full_resolution_neighbours;
        let (cursor, total) = (self.cursor, self.paths.len());
        self.windows.full.get(cursor, total, radius);
        let window = self.windows.full.take();

        for index in window.iter().copied() {
            if self.full.contains(index)
                || self.full_requested.contains(&index)
                || self.failed.contains(&index)
            {
                continue;
            }

            let Some(path) = self.paths.get(index) else {
                continue;
            };

            self.full_requested.insert(index);
            self.loader.submit(Job {
                key: ImageKey {
                    generation: self.generation,
                    index,
                },
                priority: self.config.priority_bias + PRIORITY,
                radius: Some(self.config.full_resolution_neighbours),
                path: path.clone(),
                // No cap: this copy exists precisely to be looked at closely.
                options: self.options.clone().with_display_edge(None),
                focus: Arc::clone(&self.focus),
                responder: self.full_responder.clone(),
            });
        }

        self.windows.full.give_back(window);
    }

    /// Takes in the full resolution decodes that have finished.
    pub(super) fn collect_full(&mut self) -> bool {
        let mut collected = false;

        while let Ok(result) = self.full_results.try_recv() {
            if result.key.generation != self.generation {
                continue;
            }

            let index = result.key.index;
            self.full_requested.remove(&index);

            match result.outcome {
                super::Loaded::Decoded(image) => {
                    self.full
                        .insert(index, Arc::new(image), self.cursor, self.paths.len());
                    collected = true;
                }
                // Marked failed rather than merely dropped: the window
                // asks for these every frame, and a file that cannot be
                // decoded would be asked for forever.
                super::Loaded::Failed(_) => {
                    self.failed.insert(index);
                }
                // Abandoned while the viewer moved on. Taking it out of
                // `full_requested` above is enough: it will be asked for
                // again if it is still wanted.
                super::Loaded::Abandoned => {}
            }
        }

        collected
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_screen_copy_is_enough_at_the_size_it_was_made_for() {
        assert_eq!(wanted(Detail::Screen, 2048.0, 2048.0), Detail::Screen);
    }

    #[test]
    fn magnifying_past_it_asks_for_the_image_itself() {
        assert_eq!(wanted(Detail::Screen, 2048.0, 4000.0), Detail::Full);
    }

    #[test]
    fn zooming_back_out_gives_the_screen_copy_back() {
        assert_eq!(wanted(Detail::Full, 2048.0, 800.0), Detail::Screen);
    }

    #[test]
    fn a_zoom_resting_on_the_boundary_does_not_swap_every_frame() {
        // Just above and just below the copy's own width, from either state.
        assert_eq!(wanted(Detail::Screen, 2048.0, 2090.0), Detail::Screen);
        assert_eq!(wanted(Detail::Full, 2048.0, 2000.0), Detail::Full);
    }

    #[test]
    fn an_image_no_larger_than_the_screen_has_nothing_to_swap_to() {
        // Its screen sized copy is the image, so asking for one or the other
        // is asking for the same texture twice.
        assert!(!has_two_copies(1.0));
        assert!(has_two_copies(2048.0 / 6000.0));
    }

    #[test]
    fn nothing_measured_yet_changes_nothing() {
        assert_eq!(wanted(Detail::Screen, 0.0, 4000.0), Detail::Screen);
        assert_eq!(wanted(Detail::Full, 0.0, 10.0), Detail::Full);
    }
}
