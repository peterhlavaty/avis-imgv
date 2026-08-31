//! The little pictures beside the file names.
//!
//! Deciding whether five frames are one bracket is a question about what they
//! look like, so the group panel has to show them. The pictures are the ones
//! the folder sweep already decoded to compare the frames with — the camera's
//! own thumbnails — so nothing is read or decoded twice.
//!
//! Textures are made when a row is first drawn and kept until the folder
//! changes. A collapsed group draws nothing and costs nothing.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use eframe::egui::{self, ColorImage, TextureHandle, TextureOptions};
use image::RgbaImage;

use crate::metadata::Orientation;

/// The sizes the panel offers, as the height of a row in points.
pub const SIZES: &[(&str, f32)] = &[
    ("Names only", 0.0),
    ("Small", 48.0),
    ("Medium", 96.0),
    ("Large", 180.0),
];

/// Textures for the thumbnails on screen.
#[derive(Default)]
pub struct Thumbnails {
    /// `None` for a file whose thumbnail could not be made, so it is not
    /// attempted again on every frame.
    made: HashMap<PathBuf, Option<TextureHandle>>,
}

impl Thumbnails {
    /// Forgets everything, for when the folder changes.
    pub fn clear(&mut self) {
        self.made.clear();
    }

    /// Draws the thumbnail of `path` at `height` points.
    ///
    /// Draws nothing at a height of zero, which is what the panel asks for
    /// when the user wants names alone.
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        path: &Path,
        image: Option<&Arc<RgbaImage>>,
        orientation: Orientation,
        height: f32,
    ) {
        if height <= 0.0 {
            return;
        }

        let Some(texture) = self.texture(ui.ctx(), path, image, orientation) else {
            // A gap the size of the missing picture, so the rows stay in line.
            ui.allocate_space(egui::vec2(height * 1.5, height));
            return;
        };

        let size = texture.size_vec2();
        let scale = height / size.y.max(1.0);

        ui.add(egui::Image::new(&texture).fit_to_exact_size(size * scale));
    }

    fn texture(
        &mut self,
        ctx: &egui::Context,
        path: &Path,
        image: Option<&Arc<RgbaImage>>,
        orientation: Orientation,
    ) -> Option<TextureHandle> {
        if let Some(made) = self.made.get(path) {
            return made.clone();
        }

        let made = image.map(|image| {
            // Turned here, in the pixels, rather than at draw time. The viewer
            // turns its images with the texture coordinates, which egui's own
            // `Image` widget cannot do — so the group panel showed every
            // portrait frame on its side, which is no way to decide whether
            // five of them are one bracket. It happens once per file and the
            // result is what gets cached.
            let upright = orientation.applied(image);
            let size = [upright.width() as usize, upright.height() as usize];
            let colours = ColorImage::from_rgba_unmultiplied(size, upright.as_raw());

            ctx.load_texture(path.to_string_lossy(), colours, TextureOptions::LINEAR)
        });

        self.made.insert(path.to_path_buf(), made.clone());
        made
    }
}

/// The label for a height, for the dropdown.
pub fn label(height: f32) -> &'static str {
    SIZES
        .iter()
        .find(|(_, size)| (size - height).abs() < f32::EPSILON)
        .map(|(label, _)| *label)
        .unwrap_or("Custom")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_size_has_a_name() {
        for (name, height) in SIZES {
            assert!(!name.is_empty());
            assert_eq!(label(*height), *name);
        }
    }

    #[test]
    fn names_only_is_a_height_of_nothing() {
        assert_eq!(SIZES[0].1, 0.0);
    }

    #[test]
    fn a_size_nobody_offers_is_still_named() {
        assert_eq!(label(123.0), "Custom");
    }

    #[test]
    fn a_size_of_nothing_draws_nothing_and_costs_nothing() {
        // The panel asks for zero when the user wants names alone, and the
        // store must not build a texture nobody is going to see.
        assert_eq!(SIZES[0].1, 0.0);
    }
}
