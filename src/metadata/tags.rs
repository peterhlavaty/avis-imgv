//! Tag identifiers and their human readable names.
//!
//! Names follow exiftool's descriptions so that existing configurations
//! (`metadata_tags`, `name_format`) keep working after the move to an
//! in-process EXIF reader.

/// Which directory a tag id belongs to. Ids overlap between directories, so
/// the kind is needed to resolve a name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IfdKind {
    /// IFD0/IFD1 and the TIFF tags shared with raw files.
    Root,
    Exif,
    Gps,
    Interop,
}

// Tags that point at other directories.
pub const EXIF_IFD_POINTER: u16 = 0x8769;
pub const GPS_IFD_POINTER: u16 = 0x8825;
pub const INTEROP_IFD_POINTER: u16 = 0xA005;
pub const SUB_IFDS: u16 = 0x014A;

// Tags read by the pipeline rather than merely displayed.
pub const ORIENTATION: u16 = 0x0112;
pub const ICC_PROFILE: u16 = 0x8773;
pub const XMP_PACKET: u16 = 0x02BC;
/// Windows Explorer's star rating, when there is no XMP packet.
pub const RATING: u16 = 0x4746;
pub const MAKER_NOTE: u16 = 0x927C;
pub const COLOR_SPACE: u16 = 0xA001;
pub const IMAGE_WIDTH: u16 = 0x0100;
pub const IMAGE_HEIGHT: u16 = 0x0101;
pub const EXIF_IMAGE_WIDTH: u16 = 0xA002;
pub const EXIF_IMAGE_HEIGHT: u16 = 0xA003;
pub const F_NUMBER: u16 = 0x829D;
pub const APERTURE_VALUE: u16 = 0x9202;
pub const EXPOSURE_TIME: u16 = 0x829A;
pub const SHUTTER_SPEED_VALUE: u16 = 0x9201;

// Tags describing where an embedded preview lives.
pub const JPEG_INTERCHANGE_FORMAT: u16 = 0x0201;
pub const JPEG_INTERCHANGE_FORMAT_LENGTH: u16 = 0x0202;
pub const STRIP_OFFSETS: u16 = 0x0111;
pub const STRIP_BYTE_COUNTS: u16 = 0x0117;
pub const PREVIEW_IMAGE_START: u16 = 0x0111;
pub const NEW_SUBFILE_TYPE: u16 = 0x00FE;
pub const COMPRESSION: u16 = 0x0103;

/// Names of tags found in IFD0/IFD1 and in raw sub-directories.
const ROOT_NAMES: &[(u16, &str)] = &[
    (0x00FE, "Subfile Type"),
    (0x0100, "Image Width"),
    (0x0101, "Image Height"),
    (0x0102, "Bits Per Sample"),
    (0x0103, "Compression"),
    (0x0106, "Photometric Interpretation"),
    (0x010E, "Image Description"),
    (0x010F, "Make"),
    (0x0110, "Camera Model Name"),
    (0x0111, "Strip Offsets"),
    (0x0112, "Orientation"),
    (0x0115, "Samples Per Pixel"),
    (0x0116, "Rows Per Strip"),
    (0x0117, "Strip Byte Counts"),
    (0x011A, "X Resolution"),
    (0x011B, "Y Resolution"),
    (0x011C, "Planar Configuration"),
    (0x0128, "Resolution Unit"),
    (0x0131, "Software"),
    (0x0132, "Modify Date"),
    (0x013B, "Artist"),
    (0x013E, "White Point"),
    (0x013F, "Primary Chromaticities"),
    (0x0201, "Thumbnail Offset"),
    (0x0202, "Thumbnail Length"),
    (0x0211, "Y Cb Cr Coefficients"),
    (0x0213, "Y Cb Cr Positioning"),
    (0x0214, "Reference Black White"),
    (0x02BC, "XMP Packet"),
    (0x4746, "Rating"),
    (0x4749, "Rating Percent"),
    (0x8298, "Copyright"),
    (0x8773, "Inter Color Profile"),
    (0xC612, "DNG Version"),
    (0xC614, "Unique Camera Model"),
];

/// Names of tags found in the EXIF sub-directory.
const EXIF_NAMES: &[(u16, &str)] = &[
    (0x829A, "Exposure Time"),
    (0x829D, "F Number"),
    (0x8822, "Exposure Program"),
    (0x8827, "ISO"),
    (0x8830, "Sensitivity Type"),
    (0x8832, "Recommended Exposure Index"),
    (0x9000, "Exif Version"),
    (0x9003, "Date/Time Original"),
    (0x9004, "Create Date"),
    (0x9010, "Offset Time"),
    (0x9011, "Offset Time Original"),
    (0x9012, "Offset Time Digitized"),
    (0x9101, "Components Configuration"),
    (0x9102, "Compressed Bits Per Pixel"),
    (0x9201, "Shutter Speed Value"),
    (0x9202, "Aperture Value"),
    (0x9203, "Brightness Value"),
    (0x9204, "Exposure Compensation"),
    (0x9205, "Max Aperture Value"),
    (0x9206, "Subject Distance"),
    (0x9207, "Metering Mode"),
    (0x9208, "Light Source"),
    (0x9209, "Flash"),
    (0x920A, "Focal Length"),
    (0x9286, "User Comment"),
    (0x9290, "Sub Sec Time"),
    (0x9291, "Sub Sec Time Original"),
    (0x9292, "Sub Sec Time Digitized"),
    (0xA000, "Flashpix Version"),
    (0xA001, "Color Space"),
    (0xA002, "Exif Image Width"),
    (0xA003, "Exif Image Height"),
    (0xA20E, "Focal Plane X Resolution"),
    (0xA20F, "Focal Plane Y Resolution"),
    (0xA210, "Focal Plane Resolution Unit"),
    (0xA217, "Sensing Method"),
    (0xA300, "File Source"),
    (0xA301, "Scene Type"),
    (0xA401, "Custom Rendered"),
    (0xA402, "Exposure Mode"),
    (0xA403, "White Balance"),
    (0xA404, "Digital Zoom Ratio"),
    (0xA405, "Focal Length In 35mm Format"),
    (0xA406, "Scene Capture Type"),
    (0xA407, "Gain Control"),
    (0xA408, "Contrast"),
    (0xA409, "Saturation"),
    (0xA40A, "Sharpness"),
    (0xA40C, "Subject Distance Range"),
    (0xA420, "Image Unique ID"),
    (0xA430, "Camera Owner Name"),
    (0xA431, "Serial Number"),
    (0xA432, "Lens Info"),
    (0xA433, "Lens Make"),
    (0xA434, "Lens Model"),
    (0xA435, "Lens Serial Number"),
];

/// Names of tags found in the GPS sub-directory.
const GPS_NAMES: &[(u16, &str)] = &[
    (0x0000, "GPS Version ID"),
    (0x0001, "GPS Latitude Ref"),
    (0x0002, "GPS Latitude"),
    (0x0003, "GPS Longitude Ref"),
    (0x0004, "GPS Longitude"),
    (0x0005, "GPS Altitude Ref"),
    (0x0006, "GPS Altitude"),
    (0x0007, "GPS Time Stamp"),
    (0x0009, "GPS Status"),
    (0x000A, "GPS Measure Mode"),
    (0x000C, "GPS Speed Ref"),
    (0x000D, "GPS Speed"),
    (0x0010, "GPS Img Direction Ref"),
    (0x0011, "GPS Img Direction"),
    (0x0012, "GPS Map Datum"),
    (0x001D, "GPS Date Stamp"),
];

const INTEROP_NAMES: &[(u16, &str)] = &[
    (0x0001, "Interoperability Index"),
    (0x0002, "Interoperability Version"),
];

/// Tags whose payload is uninteresting to display: version blobs, offsets into
/// the file, and vendor binary data.
const HIDDEN: &[u16] = &[
    XMP_PACKET,
    EXIF_IFD_POINTER,
    GPS_IFD_POINTER,
    INTEROP_IFD_POINTER,
    SUB_IFDS,
    MAKER_NOTE,
    ICC_PROFILE,
    0x0111, // Strip Offsets
    0x0117, // Strip Byte Counts
    0x0201, // Thumbnail Offset
    0x0202, // Thumbnail Length
];

/// Human readable name of `tag` within `kind`, if we know one.
pub fn name(kind: IfdKind, tag: u16) -> Option<&'static str> {
    let table = match kind {
        IfdKind::Root => ROOT_NAMES,
        IfdKind::Exif => EXIF_NAMES,
        IfdKind::Gps => GPS_NAMES,
        IfdKind::Interop => INTEROP_NAMES,
    };

    table
        .binary_search_by_key(&tag, |(id, _)| *id)
        .ok()
        .map(|i| table[i].1)
}

/// Whether a tag should be kept out of the displayed metadata map.
pub fn is_hidden(tag: u16) -> bool {
    HIDDEN.contains(&tag)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tables_are_sorted_for_binary_search() {
        for table in [ROOT_NAMES, EXIF_NAMES, GPS_NAMES, INTEROP_NAMES] {
            assert!(
                table.windows(2).all(|w| w[0].0 < w[1].0),
                "tag table must be sorted and free of duplicates"
            );
        }
    }

    #[test]
    fn resolves_names_per_directory() {
        assert_eq!(name(IfdKind::Root, 0x0110), Some("Camera Model Name"));
        assert_eq!(name(IfdKind::Exif, 0x9003), Some("Date/Time Original"));
        assert_eq!(name(IfdKind::Gps, 0x0002), Some("GPS Latitude"));
        // Same id, different directory, different meaning.
        assert_eq!(name(IfdKind::Root, 0x0002), None);
        assert_eq!(name(IfdKind::Exif, 0xFFFF), None);
    }

    #[test]
    fn pointer_and_binary_tags_are_hidden() {
        assert!(is_hidden(EXIF_IFD_POINTER));
        assert!(is_hidden(MAKER_NOTE));
        assert!(!is_hidden(ORIENTATION));
    }
}
