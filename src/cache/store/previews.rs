//! The preview tier: what a file says about itself before it is decoded.
//!
//! Reading the front of a file gives its metadata and the thumbnail the camera
//! embedded, both within a couple of milliseconds. That is what puts something
//! on screen while the decoders work, and what fills the side panel.

use super::super::loader::ImageKey;
use super::ImageStore;

impl ImageStore {
    /// The images a thumbnail is worth reading for.
    ///
    /// Much narrower than the decode window: a thumbnail only earns its keep
    /// in the moment between an image being asked for and its decode landing,
    /// and reading the front of every file in a wide window would take disk
    /// bandwidth away from the decoders that actually need it.
    pub(super) fn preview_window(&mut self) -> &[usize] {
        let radius = self.config.previews_resident / 2;
        let (cursor, total) = (self.cursor, self.paths.len());

        self.windows.previews.get(cursor, total, radius)
    }

    /// Reads the front of the files around the cursor, which gives their
    /// metadata and a thumbnail long before a decoder gets to them.
    pub(super) fn request_previews(&mut self) {
        if self.config.previews_resident == 0 {
            return;
        }

        // Handed over so the loop can change the store it came from.
        let radius = self.config.previews_resident / 2;
        let (cursor, total) = (self.cursor, self.paths.len());
        self.windows.previews.get(cursor, total, radius);
        let window = self.windows.previews.take();

        for index in window.iter().copied() {
            // Once the real image is decoded a thumbnail is no longer wanted.
            if self.preview_requested.contains(&index) || self.ram.contains(index) {
                continue;
            }

            let Some(path) = self.paths.get(index) else {
                continue;
            };

            self.preview_requested.insert(index);
            self.preview_loader.submit(
                ImageKey {
                    generation: self.generation,
                    index,
                },
                path.clone(),
                self.preview_responder.clone(),
            );
        }

        self.windows.previews.give_back(window);
    }

    /// Takes in the previews that have been read.
    pub(super) fn collect_previews(&mut self) -> bool {
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
}
