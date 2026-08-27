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
