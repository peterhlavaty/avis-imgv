//! Applying the EXIF orientation so every image reaches the GPU upright.

use image::{imageops, RgbaImage};

use crate::metadata::Orientation;

/// Bakes `orientation` into the pixels.
///
/// Doing it here rather than at draw time keeps the renderer free of per-image
/// transforms, and the cost is paid once on a worker thread.
///
/// See <https://magnushoff.com/articles/jpeg-orientation/> for the mapping.
pub fn apply(image: RgbaImage, orientation: Orientation) -> RgbaImage {
    match orientation {
        Orientation::Normal => image,
        Orientation::MirrorHorizontal => imageops::flip_horizontal(&image),
        Orientation::Rotate180 => imageops::rotate180(&image),
        Orientation::MirrorVertical => imageops::flip_vertical(&image),
        Orientation::Rotate90Cw => imageops::rotate90(&image),
        Orientation::Rotate270Cw => imageops::rotate270(&image),
        Orientation::MirrorHorizontalRotate90Cw => {
            imageops::rotate90(&imageops::flip_horizontal(&image))
        }
        Orientation::MirrorHorizontalRotate270 => {
            imageops::rotate270(&imageops::flip_horizontal(&image))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;

    /// A 2x1 image whose two pixels are distinguishable.
    fn probe() -> RgbaImage {
        let mut image = RgbaImage::new(2, 1);
        image.put_pixel(0, 0, Rgba([1, 0, 0, 255]));
        image.put_pixel(1, 0, Rgba([2, 0, 0, 255]));
        image
    }

    #[test]
    fn normal_leaves_pixels_alone() {
        let out = apply(probe(), Orientation::Normal);
        assert_eq!(out.get_pixel(0, 0)[0], 1);
        assert_eq!(out.get_pixel(1, 0)[0], 2);
    }

    #[test]
    fn mirroring_swaps_columns() {
        let out = apply(probe(), Orientation::MirrorHorizontal);
        assert_eq!(out.get_pixel(0, 0)[0], 2);
        assert_eq!(out.get_pixel(1, 0)[0], 1);
    }

    #[test]
    fn rotations_transpose_the_dimensions() {
        for orientation in [
            Orientation::Rotate90Cw,
            Orientation::Rotate270Cw,
            Orientation::MirrorHorizontalRotate90Cw,
            Orientation::MirrorHorizontalRotate270,
        ] {
            let out = apply(probe(), orientation);
            assert_eq!(
                (out.width(), out.height()),
                (1, 2),
                "{orientation:?} should transpose"
            );
            assert!(orientation.transposes());
        }
    }

    #[test]
    fn rotate_90_puts_the_first_pixel_on_top_right() {
        let out = apply(probe(), Orientation::Rotate90Cw);
        assert_eq!(out.get_pixel(0, 0)[0], 1);
        assert_eq!(out.get_pixel(0, 1)[0], 2);
    }

    #[test]
    fn every_orientation_preserves_the_pixel_count() {
        let image = RgbaImage::from_fn(7, 3, |x, y| Rgba([x as u8, y as u8, (x + y) as u8, 255]));

        for orientation in [
            Orientation::Normal,
            Orientation::MirrorHorizontal,
            Orientation::Rotate180,
            Orientation::MirrorVertical,
            Orientation::MirrorHorizontalRotate270,
            Orientation::Rotate90Cw,
            Orientation::MirrorHorizontalRotate90Cw,
            Orientation::Rotate270Cw,
        ] {
            let out = apply(image.clone(), orientation);
            assert_eq!(out.as_raw().len(), image.as_raw().len(), "{orientation:?}");
        }
    }

    #[test]
    fn opposite_rotations_cancel_out() {
        let image = RgbaImage::from_fn(5, 3, |x, y| Rgba([x as u8, y as u8, 0, 255]));

        let there = apply(image.clone(), Orientation::Rotate90Cw);
        let back = apply(there, Orientation::Rotate270Cw);

        assert_eq!(back, image);
    }
}
