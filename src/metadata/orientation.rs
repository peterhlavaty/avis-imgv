//! The EXIF orientation tag.

/// EXIF orientation, describing how the stored pixels must be transformed to
/// be displayed upright.
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, Copy, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
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

    /// The EXIF number for this orientation, which is what a sidecar holds.
    pub fn to_exif(self) -> u32 {
        match self {
            Orientation::Normal => 1,
            Orientation::MirrorHorizontal => 2,
            Orientation::Rotate180 => 3,
            Orientation::MirrorVertical => 4,
            Orientation::MirrorHorizontalRotate270 => 5,
            Orientation::Rotate90Cw => 6,
            Orientation::MirrorHorizontalRotate90Cw => 7,
            Orientation::Rotate270Cw => 8,
        }
    }

    /// This orientation as a mirror and a number of quarter turns.
    ///
    /// Every one of the eight is a horizontal mirror, then some number of
    /// quarter turns clockwise — which is what makes them composable at all:
    /// they are the eight symmetries of a rectangle, and this is the pair of
    /// coordinates that names one.
    fn parts(self) -> (u8, bool) {
        match self {
            Orientation::Normal => (0, false),
            Orientation::MirrorHorizontal => (0, true),
            Orientation::Rotate90Cw => (1, false),
            Orientation::MirrorHorizontalRotate270 => (1, true),
            Orientation::Rotate180 => (2, false),
            Orientation::MirrorVertical => (2, true),
            Orientation::Rotate270Cw => (3, false),
            Orientation::MirrorHorizontalRotate90Cw => (3, true),
        }
    }

    fn of_parts(turns: u8, mirrored: bool) -> Orientation {
        match (turns % 4, mirrored) {
            (0, false) => Orientation::Normal,
            (0, true) => Orientation::MirrorHorizontal,
            (1, false) => Orientation::Rotate90Cw,
            (1, true) => Orientation::MirrorHorizontalRotate270,
            (2, false) => Orientation::Rotate180,
            (2, true) => Orientation::MirrorVertical,
            (3, false) => Orientation::Rotate270Cw,
            _ => Orientation::MirrorHorizontalRotate90Cw,
        }
    }

    /// This orientation, and then `next`.
    ///
    /// The camera says how its pixels have to be turned to be upright; the
    /// user says how the upright photograph has to be turned again. One turn
    /// composed with the other is one orientation, so nothing downstream needs
    /// to know that two of them were involved — which is what keeps the
    /// rotation out of the pixels and out of the file.
    ///
    /// A mirror reverses the sense of a turn, which is where the subtraction
    /// comes from: mirroring and then turning clockwise is turning
    /// anticlockwise and then mirroring.
    pub fn then(self, next: Orientation) -> Orientation {
        let (first, mirrored_first) = self.parts();
        let (second, mirrored_second) = next.parts();

        let turns = if mirrored_second {
            (4 + second - first) % 4
        } else {
            (first + second) % 4
        };

        Orientation::of_parts(turns, mirrored_first != mirrored_second)
    }

    /// The turn that puts this one back.
    ///
    /// Wanted by undo: what is on the graphics card has already been turned,
    /// and putting the sidecar back means turning the card back by whatever
    /// the difference between the two orientations is. A mirrored one is its
    /// own inverse — mirroring twice is doing nothing — and a plain rotation
    /// is undone by the rest of the circle.
    pub fn inverse(self) -> Orientation {
        let (turns, mirrored) = self.parts();

        if mirrored {
            self
        } else {
            Orientation::of_parts(4 - turns, false)
        }
    }

    /// A quarter turn on its own, to be composed with what is already there.
    pub fn quarter(clockwise: bool) -> Orientation {
        if clockwise {
            Orientation::Rotate90Cw
        } else {
            Orientation::Rotate270Cw
        }
    }

    /// A quarter turn on top of this one.
    pub fn turned(self, clockwise: bool) -> Orientation {
        self.then(Orientation::quarter(clockwise))
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

    const ALL: [Orientation; 8] = [
        Orientation::Normal,
        Orientation::MirrorHorizontal,
        Orientation::Rotate180,
        Orientation::MirrorVertical,
        Orientation::MirrorHorizontalRotate270,
        Orientation::Rotate90Cw,
        Orientation::MirrorHorizontalRotate90Cw,
        Orientation::Rotate270Cw,
    ];

    /// Composition, against the pixels themselves.
    ///
    /// All sixty-four pairs: turning by one and then by the other has to give
    /// the same picture as turning once by the composition. This is the whole
    /// warrant for keeping a user's rotation out of the file — the camera's
    /// orientation and the user's are one orientation by the time anything
    /// draws.
    #[test]
    fn composing_two_turns_is_one_turn() {
        let image = marked();

        for first in ALL {
            for second in ALL {
                let composed = first.then(second).applied(&image);
                let once = first.applied(&image);
                let one_at_a_time = second.applied(&once);

                assert_eq!(
                    composed.as_ref(),
                    one_at_a_time.as_ref(),
                    "{first:?} then {second:?} is {:?}",
                    first.then(second)
                );
            }
        }
    }

    /// Four quarter turns is where you started, from any of the eight.
    #[test]
    fn four_quarter_turns_come_back() {
        for orientation in ALL {
            let mut turned = orientation;
            for _ in 0..4 {
                turned = turned.turned(true);
            }

            assert_eq!(turned, orientation);
        }
    }

    /// And one each way cancels, which is what makes the two keys a pair.
    #[test]
    fn a_turn_each_way_cancels() {
        for orientation in ALL {
            assert_eq!(orientation.turned(true).turned(false), orientation);
            assert_eq!(orientation.turned(false).turned(true), orientation);
        }
    }

    /// Every orientation undoes itself, which is what lets an undo turn the
    /// card back without knowing how it got where it is.
    #[test]
    fn every_orientation_has_an_inverse() {
        for orientation in ALL {
            assert_eq!(
                orientation.then(orientation.inverse()),
                Orientation::Normal,
                "{orientation:?}"
            );
        }
    }

    /// And the difference between two of them takes one to the other,
    /// whatever the camera said, because the camera's part cancels.
    #[test]
    fn the_difference_between_two_takes_one_to_the_other() {
        for was in ALL {
            for now in ALL {
                let difference = was.inverse().then(now);
                assert_eq!(was.then(difference), now, "{was:?} to {now:?}");
            }
        }
    }

    /// The EXIF numbers, which are what a sidecar carries.
    #[test]
    fn every_orientation_round_trips_through_its_number() {
        for orientation in ALL {
            assert_eq!(Orientation::from_exif(orientation.to_exif()), orientation);
        }
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
