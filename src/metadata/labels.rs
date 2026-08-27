//! Tables mapping the numeric value of a tag to the words exiftool prints.

/// Lookup table of `(numeric value, label)` pairs.
pub type Labels = &'static [(u32, &'static str)];

pub const ORIENTATION: Labels = &[
    (1, "Horizontal (normal)"),
    (2, "Mirror horizontal"),
    (3, "Rotate 180"),
    (4, "Mirror vertical"),
    (5, "Mirror horizontal and rotate 270 CW"),
    (6, "Rotate 90 CW"),
    (7, "Mirror horizontal and rotate 90 CW"),
    (8, "Rotate 270 CW"),
];

pub const RESOLUTION_UNIT: Labels = &[(1, "None"), (2, "inches"), (3, "cm")];

pub const COLOR_SPACE: Labels = &[(1, "sRGB"), (2, "Adobe RGB"), (0xFFFF, "Uncalibrated")];

pub const EXPOSURE_PROGRAM: Labels = &[
    (0, "Not Defined"),
    (1, "Manual"),
    (2, "Program AE"),
    (3, "Aperture-priority AE"),
    (4, "Shutter speed priority AE"),
    (5, "Creative (Slow speed)"),
    (6, "Action (High speed)"),
    (7, "Portrait"),
    (8, "Landscape"),
    (9, "Bulb"),
];

pub const METERING_MODE: Labels = &[
    (0, "Unknown"),
    (1, "Average"),
    (2, "Center-weighted average"),
    (3, "Spot"),
    (4, "Multi-spot"),
    (5, "Multi-segment"),
    (6, "Partial"),
    (255, "Other"),
];

pub const LIGHT_SOURCE: Labels = &[
    (0, "Unknown"),
    (1, "Daylight"),
    (2, "Fluorescent"),
    (3, "Tungsten (Incandescent)"),
    (4, "Flash"),
    (9, "Fine Weather"),
    (10, "Cloudy"),
    (11, "Shade"),
    (17, "Standard Light A"),
    (18, "Standard Light B"),
    (19, "Standard Light C"),
    (24, "ISO Studio Tungsten"),
    (255, "Other"),
];

pub const WHITE_BALANCE: Labels = &[(0, "Auto"), (1, "Manual")];

pub const EXPOSURE_MODE: Labels = &[(0, "Auto"), (1, "Manual"), (2, "Auto bracket")];

pub const SCENE_CAPTURE_TYPE: Labels = &[
    (0, "Standard"),
    (1, "Landscape"),
    (2, "Portrait"),
    (3, "Night"),
];

pub const NORMAL_LOW_HIGH: Labels = &[(0, "Normal"), (1, "Low"), (2, "High")];

pub const SENSING_METHOD: Labels = &[
    (1, "Not defined"),
    (2, "One-chip color area"),
    (3, "Two-chip color area"),
    (4, "Three-chip color area"),
    (5, "Color sequential area"),
    (7, "Trilinear"),
    (8, "Color sequential linear"),
];

pub const SUBJECT_DISTANCE_RANGE: Labels =
    &[(0, "Unknown"), (1, "Macro"), (2, "Close"), (3, "Distant")];

pub const GAIN_CONTROL: Labels = &[
    (0, "None"),
    (1, "Low gain up"),
    (2, "High gain up"),
    (3, "Low gain down"),
    (4, "High gain down"),
];

pub const CUSTOM_RENDERED: Labels = &[(0, "Normal"), (1, "Custom")];

pub const Y_CB_CR_POSITIONING: Labels = &[(1, "Centered"), (2, "Co-sited")];

pub const FILE_SOURCE: Labels = &[
    (1, "Film Scanner"),
    (2, "Reflection Print Scanner"),
    (3, "Digital Camera"),
];

pub const COMPRESSION: Labels = &[
    (1, "Uncompressed"),
    (6, "JPEG (old-style)"),
    (7, "JPEG"),
    (8, "Adobe Deflate"),
    (32773, "PackBits"),
];

/// The label for `value`, if the table has one.
pub fn lookup(labels: Labels, value: u32) -> Option<&'static str> {
    labels
        .iter()
        .find(|(key, _)| *key == value)
        .map(|(_, name)| *name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_a_label() {
        assert_eq!(lookup(ORIENTATION, 1), Some("Horizontal (normal)"));
        assert_eq!(lookup(ORIENTATION, 8), Some("Rotate 270 CW"));
        assert_eq!(lookup(COLOR_SPACE, 0xFFFF), Some("Uncalibrated"));
    }

    #[test]
    fn reports_a_value_the_table_does_not_cover() {
        assert_eq!(lookup(ORIENTATION, 99), None);
        assert_eq!(lookup(&[], 1), None);
    }

    #[test]
    fn tables_have_no_duplicate_keys() {
        for table in [ORIENTATION, EXPOSURE_PROGRAM, METERING_MODE, LIGHT_SOURCE] {
            let mut keys: Vec<u32> = table.iter().map(|(key, _)| *key).collect();
            let count = keys.len();
            keys.sort_unstable();
            keys.dedup();

            assert_eq!(keys.len(), count);
        }
    }
}
