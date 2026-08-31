//! Holding the clipping and focus masks, and drawing them over the picture.
//!
//! One texture at a time: the mask is about the photograph being looked at, it
//! is the same size as the copy on screen, and keeping one per photograph in
//! the folder would double what the cache holds for a thing that is off most
//! of the time.
//!
//! Built when the overlay is switched on or the photograph changes, which is a
//! single pass over the pixels of the *displayed* copy — a couple of megapixels
//! at the cap, so a few milliseconds — rather than on every decode.

use std::path::{Path, PathBuf};

use eframe::egui::{self, TextureHandle, TextureOptions};

use crate::decoder::overlays::Overlay;

/// The mask for the photograph on screen, if there is one.
#[derive(Default)]
pub struct Marks {
    texture: Option<TextureHandle>,
    /// What the held texture is of, so it is not rebuilt every frame.
    built_for: Option<(PathBuf, Overlay)>,
}

impl Marks {
    /// Builds the mask if it is not the one already held.
    ///
    /// `pixels` are the displayed copy's, RGBA. The photograph's own
    /// orientation is not applied: the mask is drawn through the same texture
    /// coordinates as the photograph, so it is turned by the same rule.
    pub fn prepare(
        &mut self,
        ctx: &egui::Context,
        overlay: Overlay,
        path: &Path,
        pixels: &[u8],
        width: u32,
        height: u32,
    ) {
        if overlay == Overlay::Off {
            self.forget();
            return;
        }

        let wanted = (path.to_path_buf(), overlay);
        if self.built_for.as_ref() == Some(&wanted) {
            return;
        }

        self.built_for = Some(wanted);
        self.texture = crate::decoder::overlays::mask(overlay, pixels, width, height).map(|mask| {
            let size = [mask.width() as usize, mask.height() as usize];
            let colours = egui::ColorImage::from_rgba_unmultiplied(size, mask.as_raw());

            // Nearest rather than linear: a mask is a statement about
            // particular pixels, and smearing it between them turns "these
            // three are blown" into a pink haze that means nothing.
            ctx.load_texture("overlay-mask", colours, TextureOptions::NEAREST)
        });
    }

    /// Drops what is held, for when the overlay is switched off.
    pub fn forget(&mut self) {
        self.texture = None;
        self.built_for = None;
    }

    /// The texture to paint, if there is one.
    pub fn texture_id(&self) -> Option<egui::TextureId> {
        self.texture.as_ref().map(TextureHandle::id)
    }

    /// Whether a mask is being held.
    pub fn is_showing(&self) -> bool {
        self.texture.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nothing_is_held_to_begin_with() {
        let marks = Marks::default();

        assert!(!marks.is_showing());
        assert!(marks.built_for.is_none());
    }

    /// Switching the overlay off drops the texture rather than leaving a
    /// megabyte or two behind for something nobody is looking at.
    #[test]
    fn switching_it_off_lets_the_texture_go() {
        let mut marks = Marks {
            texture: None,
            built_for: Some((PathBuf::from("/photos/a.jpg"), Overlay::Clipping)),
        };

        marks.forget();
        assert!(marks.built_for.is_none());
    }

    /// The mask is keyed by the photograph *and* the overlay, so switching
    /// between clipping and peaking on one picture rebuilds it.
    #[test]
    fn the_key_is_the_photograph_and_the_overlay() {
        let one = (PathBuf::from("/photos/a.jpg"), Overlay::Clipping);
        let same = (PathBuf::from("/photos/a.jpg"), Overlay::Clipping);
        let other_overlay = (PathBuf::from("/photos/a.jpg"), Overlay::Peaking);
        let other_photograph = (PathBuf::from("/photos/b.jpg"), Overlay::Clipping);

        assert_eq!(one, same);
        assert_ne!(one, other_overlay);
        assert_ne!(one, other_photograph);
    }
}
