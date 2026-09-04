//! Drawing a cached texture, orientation and all.
//!
//! A photograph taken sideways is stored sideways; the camera records how it
//! should be turned rather than turning the pixels. Doing that turn on the CPU
//! means copying ninety megabytes twice for a 24 megapixel frame — 86ms, a
//! third of the time it takes to open one. Doing it on the GPU means handing
//! four corners to the rasteriser in a different order, which is free.

use eframe::egui::{self, Color32, Mesh, Pos2, Rect, Shape, Vec2};

use crate::cache::gpu::GpuTexture;
use crate::metadata::Orientation;

/// Draws `texture` into `rect`, showing the part of it `uv` selects.
///
/// `uv` is in the coordinates of the image as displayed, so a caller works out
/// what to show without knowing which way the pixels are stored.
pub fn draw(ui: &egui::Ui, rect: Rect, texture: &GpuTexture, uv: Rect) {
    let mut mesh = Mesh::with_texture(texture.id);

    // Screen corners clockwise from the top left, and the point of the texture
    // each one samples.
    let corners = [
        (rect.left_top(), uv.left_top()),
        (rect.right_top(), uv.right_top()),
        (rect.right_bottom(), uv.right_bottom()),
        (rect.left_bottom(), uv.left_bottom()),
    ];

    let first = mesh.vertices.len() as u32;
    for (position, uv) in corners {
        mesh.vertices.push(egui::epaint::Vertex {
            pos: position,
            uv: to_texture(texture.orientation, uv),
            color: Color32::WHITE,
        });
    }

    mesh.indices
        .extend([first, first + 1, first + 2, first, first + 2, first + 3]);

    ui.painter().add(Shape::mesh(mesh));
}

/// Maps a point on the image as displayed onto the texture that holds it.
///
/// Both are normalised to the unit square, and every EXIF orientation is a
/// quarter turn, a mirror, or both — so this is always a permutation of the
/// two coordinates with some of them reversed.
pub fn to_texture(orientation: Orientation, point: Pos2) -> Pos2 {
    let (u, v) = (point.x, point.y);

    let (x, y) = match orientation {
        Orientation::Normal => (u, v),
        Orientation::MirrorHorizontal => (1.0 - u, v),
        Orientation::Rotate180 => (1.0 - u, 1.0 - v),
        Orientation::MirrorVertical => (u, 1.0 - v),
        Orientation::Rotate90Cw => (v, 1.0 - u),
        Orientation::Rotate270Cw => (1.0 - v, u),
        Orientation::MirrorHorizontalRotate90Cw => (1.0 - v, 1.0 - u),
        Orientation::MirrorHorizontalRotate270 => (v, u),
    };

    Pos2::new(x, y)
}

/// The size an image is shown at, given how its pixels are stored.
///
/// A quarter turn swaps the two, which every layout calculation needs to know
/// before it fits anything to the panel.
pub fn displayed_size(stored: Vec2, orientation: Orientation) -> Vec2 {
    if orientation.transposes() {
        Vec2::new(stored.y, stored.x)
    } else {
        stored
    }
}

/// The toolkit's vector, as something [`crate::fit`] can size.
///
/// Here rather than in `fit.rs` so that `fit.rs` names nothing from the
/// toolkit and could be lifted out as it stands. Within one crate that is a
/// convention rather than a rule — the trait is local everywhere, so the
/// orphan rule does not forbid the impl being written there — but it is the
/// convention that keeps the option open, and the day `fit` is its own crate
/// it is the only place this impl can go.
impl crate::fit::Edges for Vec2 {
    fn width(self) -> f32 {
        self.x
    }

    fn height(self) -> f32 {
        self.y
    }

    fn of(width: f32, height: f32) -> Self {
        Vec2::new(width, height)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{imageops, Rgba, RgbaImage};

    const EVERY_ORIENTATION: [Orientation; 8] = [
        Orientation::Normal,
        Orientation::MirrorHorizontal,
        Orientation::Rotate180,
        Orientation::MirrorVertical,
        Orientation::MirrorHorizontalRotate270,
        Orientation::Rotate90Cw,
        Orientation::MirrorHorizontalRotate90Cw,
        Orientation::Rotate270Cw,
    ];

    /// Turning the pixels, which is what the GPU mapping has to agree with.
    ///
    /// See <https://magnushoff.com/articles/jpeg-orientation/>.
    fn reference(image: &RgbaImage, orientation: Orientation) -> RgbaImage {
        match orientation {
            Orientation::Normal => image.clone(),
            Orientation::MirrorHorizontal => imageops::flip_horizontal(image),
            Orientation::Rotate180 => imageops::rotate180(image),
            Orientation::MirrorVertical => imageops::flip_vertical(image),
            Orientation::Rotate90Cw => imageops::rotate90(image),
            Orientation::Rotate270Cw => imageops::rotate270(image),
            Orientation::MirrorHorizontalRotate90Cw => {
                imageops::rotate90(&imageops::flip_horizontal(image))
            }
            Orientation::MirrorHorizontalRotate270 => {
                imageops::rotate270(&imageops::flip_horizontal(image))
            }
        }
    }

    /// Every pixel distinguishable from every other.
    fn probe(width: u32, height: u32) -> RgbaImage {
        RgbaImage::from_fn(width, height, |x, y| Rgba([x as u8, y as u8, 0, 255]))
    }

    /// Samples `image` at a point given in normalised coordinates.
    fn sample(image: &RgbaImage, point: Pos2) -> Rgba<u8> {
        let x = ((point.x * image.width() as f32) as u32).min(image.width() - 1);
        let y = ((point.y * image.height() as f32) as u32).min(image.height() - 1);

        *image.get_pixel(x, y)
    }

    #[test]
    fn the_mapping_agrees_with_turning_the_pixels() {
        let stored = probe(7, 5);

        for orientation in EVERY_ORIENTATION {
            let turned = reference(&stored, orientation);

            // Every point of the displayed image must sample the pixel the
            // turned image has there.
            for row in 0..turned.height() {
                for column in 0..turned.width() {
                    let point = Pos2::new(
                        (column as f32 + 0.5) / turned.width() as f32,
                        (row as f32 + 0.5) / turned.height() as f32,
                    );

                    assert_eq!(
                        sample(&stored, to_texture(orientation, point)),
                        *turned.get_pixel(column, row),
                        "{orientation:?} at ({column}, {row})"
                    );
                }
            }
        }
    }

    #[test]
    fn the_mapping_stays_inside_the_texture() {
        for orientation in EVERY_ORIENTATION {
            for corner in [
                Pos2::ZERO,
                Pos2::new(1.0, 0.0),
                Pos2::new(1.0, 1.0),
                Pos2::new(0.0, 1.0),
            ] {
                let mapped = to_texture(orientation, corner);

                assert!(
                    (0.0..=1.0).contains(&mapped.x),
                    "{orientation:?} {mapped:?}"
                );
                assert!(
                    (0.0..=1.0).contains(&mapped.y),
                    "{orientation:?} {mapped:?}"
                );
            }
        }
    }

    #[test]
    fn an_upright_image_is_mapped_unchanged() {
        let point = Pos2::new(0.25, 0.75);

        assert_eq!(to_texture(Orientation::Normal, point), point);
    }

    #[test]
    fn a_quarter_turn_swaps_the_displayed_dimensions() {
        let stored = Vec2::new(4000.0, 6000.0);

        assert_eq!(
            displayed_size(stored, Orientation::Rotate90Cw),
            Vec2::new(6000.0, 4000.0)
        );
        assert_eq!(displayed_size(stored, Orientation::Normal), stored);
        assert_eq!(displayed_size(stored, Orientation::Rotate180), stored);
    }

    #[test]
    fn the_displayed_size_matches_what_turning_produces() {
        let stored = probe(7, 5);

        for orientation in EVERY_ORIENTATION {
            let turned = reference(&stored, orientation);
            let expected = Vec2::new(turned.width() as f32, turned.height() as f32);

            assert_eq!(
                displayed_size(Vec2::new(7.0, 5.0), orientation),
                expected,
                "{orientation:?}"
            );
        }
    }
}
