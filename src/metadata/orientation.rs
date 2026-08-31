//! The EXIF orientation tag.

/// EXIF orientation, describing how the stored pixels must be transformed to
/// be displayed upright.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Orientation {
    #[default]
    Normal,
    MirrorHorizontal,
    Rotate180,
    MirrorVertical,
    MirrorHorizontalRotate270,
    Rotate90Cw,
    MirrorHorizontalRotate90Cw,
    Rotate270Cw,
}

impl Orientation {
    /// Maps the raw EXIF value (1..=8); anything else means upright.
    pub fn from_exif(value: u32) -> Orientation {
        match value {
            2 => Orientation::MirrorHorizontal,
            3 => Orientation::Rotate180,
            4 => Orientation::MirrorVertical,
            5 => Orientation::MirrorHorizontalRotate270,
            6 => Orientation::Rotate90Cw,
            7 => Orientation::MirrorHorizontalRotate90Cw,
            8 => Orientation::Rotate270Cw,
            _ => Orientation::Normal,
        }
    }

    /// Whether applying this orientation swaps width and height.
    /// A copy of `image` with the turn and mirror actually applied.
    ///
    /// The viewer turns its images with texture coordinates, which costs
    /// nothing and is the right answer where the drawing is ours. Where it is
    /// not — egui's own `Image` widget in the group panel — the pixels have to
    /// be moved instead. An upright image is handed back unchanged, so the
    /// common case is one clone rather than a rotation.
    pub fn applied(self, image: &image::RgbaImage) -> std::borrow::Cow<'_, image::RgbaImage> {
        use image::imageops;
        use std::borrow::Cow;

        match self {
            Orientation::Normal => Cow::Borrowed(image),
            Orientation::MirrorHorizontal => Cow::Owned(imageops::flip_horizontal(image)),
            Orientation::Rotate180 => Cow::Owned(imageops::rotate180(image)),
            Orientation::MirrorVertical => Cow::Owned(imageops::flip_vertical(image)),
            Orientation::Rotate90Cw => Cow::Owned(imageops::rotate90(image)),
            Orientation::Rotate270Cw => Cow::Owned(imageops::rotate270(image)),
            // The two diagonal mirrors, each a quarter turn and a flip.
            Orientation::MirrorHorizontalRotate90Cw => {
                Cow::Owned(imageops::flip_horizontal(&imageops::rotate90(image)))
            }
            Orientation::MirrorHorizontalRotate270 => {
                Cow::Owned(imageops::flip_horizontal(&imageops::rotate270(image)))
            }
        }
    }

    pub fn transposes(self) -> bool {
        matches!(
            self,
            Orientation::MirrorHorizontalRotate270
                | Orientation::Rotate90Cw
                | Orientation::MirrorHorizontalRotate90Cw
                | Orientation::Rotate270Cw
        )
    }
}

#[cfg(test)]
mod orientation_tests {
    use super::*;
    use image::{Rgba, RgbaImage};

    /// A three by two image whose every pixel is different.
    ///
    /// Big enough and asymmetric enough that no two of the eight orientations
    /// can produce the same picture: a two by one with one marked corner
    /// cannot tell a horizontal mirror from a half turn.
    fn marked() -> RgbaImage {
        let mut image = RgbaImage::new(3, 2);
        for (x, y, pixel) in image.enumerate_pixels_mut() {
            *pixel = Rgba([(x * 40 + y * 7) as u8, x as u8, y as u8, 255]);
        }

        image
    }

    #[test]
    fn an_upright_image_is_not_copied() {
        let image = marked();
        let applied = Orientation::Normal.applied(&image);

        assert!(matches!(applied, std::borrow::Cow::Borrowed(_)));
    }

    /// A quarter turn swaps the sides, which is exactly what the panel needs
    /// and what it was not getting.
    #[test]
    fn a_quarter_turn_swaps_the_sides() {
        let image = marked();

        for orientation in [
            Orientation::Rotate90Cw,
            Orientation::Rotate270Cw,
            Orientation::MirrorHorizontalRotate90Cw,
            Orientation::MirrorHorizontalRotate270,
        ] {
            let turned = orientation.applied(&image);
            assert_eq!((turned.width(), turned.height()), (2, 3), "{orientation:?}");
            assert!(orientation.transposes(), "{orientation:?}");
        }
    }

    #[test]
    fn the_ones_that_do_not_transpose_keep_their_shape() {
        let image = marked();

        for orientation in [
            Orientation::MirrorHorizontal,
            Orientation::Rotate180,
            Orientation::MirrorVertical,
        ] {
            let turned = orientation.applied(&image);
            assert_eq!((turned.width(), turned.height()), (3, 2), "{orientation:?}");
            assert!(!orientation.transposes(), "{orientation:?}");
        }
    }

    /// Every orientation moves the marked corner somewhere different, so none
    /// of them is secretly another.
    #[test]
    fn every_orientation_is_its_own() {
        let image = marked();
        let mut seen = Vec::new();

        for value in 1..=8u32 {
            let applied = Orientation::from_exif(value).applied(&image);
            let corners = applied.as_raw().clone();

            assert!(!seen.contains(&corners), "orientation {value} is a repeat");
            seen.push(corners);
        }
    }

    #[test]
    fn a_turn_and_a_turn_back_is_the_original() {
        let image = marked();

        let there = Orientation::Rotate90Cw.applied(&image).into_owned();
        let back = Orientation::Rotate270Cw.applied(&there);

        assert_eq!(back.as_raw(), image.as_raw());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_the_eight_defined_values() {
        let expected = [
            Orientation::Normal,
            Orientation::MirrorHorizontal,
            Orientation::Rotate180,
            Orientation::MirrorVertical,
            Orientation::MirrorHorizontalRotate270,
            Orientation::Rotate90Cw,
            Orientation::MirrorHorizontalRotate90Cw,
            Orientation::Rotate270Cw,
        ];

        for (raw, orientation) in (1..=8).zip(expected) {
            assert_eq!(Orientation::from_exif(raw), orientation);
        }
    }

    #[test]
    fn anything_else_means_upright() {
        assert_eq!(Orientation::from_exif(0), Orientation::Normal);
        assert_eq!(Orientation::from_exif(42), Orientation::Normal);
        assert_eq!(Orientation::default(), Orientation::Normal);
    }

    #[test]
    fn only_the_quarter_turns_transpose() {
        assert!(Orientation::Rotate90Cw.transposes());
        assert!(Orientation::Rotate270Cw.transposes());
        assert!(Orientation::MirrorHorizontalRotate90Cw.transposes());
        assert!(Orientation::MirrorHorizontalRotate270.transposes());

        assert!(!Orientation::Normal.transposes());
        assert!(!Orientation::Rotate180.transposes());
        assert!(!Orientation::MirrorHorizontal.transposes());
        assert!(!Orientation::MirrorVertical.transposes());
    }
}
