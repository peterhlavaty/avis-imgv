//! Tags the viewer works out rather than reads.
//!
//! exiftool calls these composite tags. They are what the configuration
//! actually names — `Aperture` and `Shutter Speed` rather than `F Number` and
//! `Exposure Time` — plus what can only be known once the file is on disk and
//! decoded.

use std::path::Path;

use super::{value, Metadata};

impl Metadata {
    /// Adds the tags derived from the file itself rather than its contents.
    pub fn add_file_tags(&mut self, path: &Path, byte_len: usize) {
        if let Some(name) = path.file_name() {
            self.insert("File Name", name.to_string_lossy());
        }
        if let Some(parent) = path.parent() {
            self.insert("Directory", parent.to_string_lossy());
        }

        self.insert("File Size", format_byte_size(byte_len));
    }

    /// Records the dimensions of the decoded image.
    pub fn add_size_tags(&mut self, width: u32, height: u32) {
        self.insert("Image Size", format!("{width}x{height}"));
        self.insert(
            "Megapixels",
            value::format_f64(((width as f64 * height as f64) / 1_000_000.0 * 10.0).round() / 10.0),
        );
    }

    /// Tags exiftool computes rather than reads, and which the default name
    /// format relies on.
    pub(super) fn add_composite_tags(&mut self) {
        if let Some(aperture) = self
            .tags
            .get("F Number")
            .or_else(|| self.tags.get("Aperture Value"))
            .cloned()
        {
            self.insert("Aperture", aperture);
        }

        if let Some(shutter) = self
            .tags
            .get("Exposure Time")
            .or_else(|| self.tags.get("Shutter Speed Value"))
            .cloned()
        {
            self.insert("Shutter Speed", shutter);
        }
    }
}

fn format_byte_size(bytes: usize) -> String {
    const UNITS: &[(f64, &str)] = &[(1e9, "GB"), (1e6, "MB"), (1e3, "kB")];
    let bytes = bytes as f64;

    for (scale, unit) in UNITS {
        if bytes >= *scale {
            return format!(
                "{} {unit}",
                value::format_f64((bytes / scale * 10.0).round() / 10.0)
            );
        }
    }

    format!("{} bytes", bytes as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_and_size_tags_are_added() {
        let mut metadata = Metadata::default();
        metadata.add_file_tags(Path::new("/photos/trip/DSCF0001.JPG"), 5_400_000);
        metadata.add_size_tags(6000, 4000);

        assert_eq!(
            metadata.tags.get("File Name").map(String::as_str),
            Some("DSCF0001.JPG")
        );
        assert!(metadata
            .tags
            .get("Directory")
            .is_some_and(|d| d.ends_with("trip")));
        assert_eq!(
            metadata.tags.get("File Size").map(String::as_str),
            Some("5.4 MB")
        );
        assert_eq!(
            metadata.tags.get("Image Size").map(String::as_str),
            Some("6000x4000")
        );
        assert_eq!(
            metadata.tags.get("Megapixels").map(String::as_str),
            Some("24")
        );
    }

    #[test]
    fn byte_sizes_are_scaled_to_something_readable() {
        assert_eq!(format_byte_size(512), "512 bytes");
        assert_eq!(format_byte_size(5_400), "5.4 kB");
        assert_eq!(format_byte_size(5_400_000), "5.4 MB");
        assert_eq!(format_byte_size(5_400_000_000), "5.4 GB");
    }

    #[test]
    fn a_composite_tag_prefers_the_plain_value_over_the_apex_one() {
        let mut metadata = Metadata::default();
        metadata.insert("F Number", "5.6");
        metadata.insert("Aperture Value", "5.7");
        metadata.add_composite_tags();

        assert_eq!(
            metadata.tags.get("Aperture").map(String::as_str),
            Some("5.6")
        );
    }

    #[test]
    fn a_file_with_neither_gets_no_composite_tags() {
        let mut metadata = Metadata::default();
        metadata.add_composite_tags();

        assert!(!metadata.tags.contains_key("Aperture"));
        assert!(!metadata.tags.contains_key("Shutter Speed"));
    }
}
