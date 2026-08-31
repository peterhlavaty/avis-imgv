//! Turning a file on disk into pixels ready to be handed to the GPU.
//!
//! The whole pipeline runs on a worker thread and touches the file exactly
//! once: the bytes are read, metadata is parsed out of that same buffer, and
//! the image is decoded, resized, oriented and colour converted before being
//! stored as RGBA8 — the layout wgpu wants, so upload is a plain copy.

pub mod codec;
pub mod color;
pub mod preview;
pub mod raw;
pub mod resize;

use std::fmt;
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

use image::RgbaImage;

use crate::formats::Format;
use crate::metadata::{Metadata, Orientation};

/// Bytes per pixel in a decoded image.
pub const BYTES_PER_PIXEL: usize = 4;

/// Pixels at one resolution, tightly packed RGBA8.
pub struct Surface {
    /// `width * height * 4` bytes.
    pub pixels: Box<[u8]>,
    pub width: u32,
    pub height: u32,
}

impl Surface {
    fn from_image(image: RgbaImage) -> Surface {
        Surface {
            width: image.width(),
            height: image.height(),
            pixels: image.into_raw().into_boxed_slice(),
        }
    }

    pub fn byte_len(&self) -> usize {
        self.pixels.len()
    }
}

/// An image in memory, ready for texture upload.
///
/// The pixels held are not necessarily the image's own: browsing keeps a copy
/// no larger than the screen, because a 24 megapixel photograph is a hundred
/// megabytes and a monitor can show three of those megapixels. [`full_size`]
/// records what the image really is, so everything downstream — the layout,
/// the metadata, the zoom arithmetic — works in the image's own coordinates
/// whatever resolution arrived.
///
/// [`full_size`]: Self::full_size
pub struct DecodedImage {
    pub surface: Surface,
    /// Size of the image itself, which `surface` may be a reduction of.
    pub full_size: (u32, u32),
    /// How the pixels have to be turned to be shown upright.
    ///
    /// The turn is left to the GPU, which does it by sampling the texture in a
    /// different order rather than by copying ninety megabytes.
    pub orientation: Orientation,
    pub metadata: Metadata,
}

impl DecodedImage {
    /// Bytes this image occupies in RAM.
    pub fn byte_len(&self) -> usize {
        self.surface.byte_len()
    }

    pub fn width(&self) -> u32 {
        self.full_size.0
    }

    pub fn height(&self) -> u32 {
        self.full_size.1
    }

    pub fn size(&self) -> [u32; 2] {
        [self.full_size.0, self.full_size.1]
    }

    /// How much of the image's own resolution these pixels hold: one for the
    /// image itself, less for a copy made to fit a screen.
    pub fn resolution(&self) -> f32 {
        self.surface.width as f32 / self.full_size.0.max(1) as f32
    }

    /// Whether these are the image's own pixels rather than a reduction.
    pub fn is_full(&self) -> bool {
        self.surface.width >= self.full_size.0
    }

    /// Name shown in logs and as a texture label.
    pub fn file_name(&self) -> &str {
        self.metadata
            .tags
            .get("File Name")
            .map(String::as_str)
            .unwrap_or("image")
    }
}

impl fmt::Debug for DecodedImage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DecodedImage")
            .field("file_name", &self.file_name())
            .field("width", &self.width())
            .field("height", &self.height())
            .finish()
    }
}

/// How a single image should be decoded.
#[derive(Clone)]
pub struct DecodeOptions {
    /// Cap on the longest edge, used for thumbnails and to stay within the
    /// GPU's maximum texture size. `None` keeps the original resolution.
    pub max_edge: Option<u32>,
    /// Longest edge worth keeping, which is as many pixels as the screen can
    /// show. The full sized buffer is dropped once the copy is made; `None`
    /// keeps the image at its own resolution, for zooming into.
    pub display_edge: Option<u32>,
    /// Name of the display profile to convert into.
    pub output_profile: Arc<str>,
    /// What to do with camera raw files.
    pub raw: raw::Options,
}

impl DecodeOptions {
    pub fn new(output_profile: Arc<str>) -> Self {
        Self {
            max_edge: None,
            display_edge: None,
            output_profile,
            raw: raw::Options::default(),
        }
    }

    pub fn with_max_edge(mut self, max_edge: Option<u32>) -> Self {
        self.max_edge = max_edge;
        self
    }

    pub fn with_display_edge(mut self, display_edge: Option<u32>) -> Self {
        self.display_edge = display_edge;
        self
    }

    pub fn with_raw(mut self, raw: raw::Options) -> Self {
        self.raw = raw;
        self
    }
}

/// Why an image could not be loaded.
#[derive(Debug)]
pub enum DecodeError {
    Read(std::io::Error),
    /// No decoder handled the bytes, or a raw file had no usable preview.
    Unsupported(String),
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DecodeError::Read(e) => write!(f, "could not be read: {e}"),
            DecodeError::Unsupported(reason) => write!(f, "could not be decoded: {reason}"),
        }
    }
}

impl std::error::Error for DecodeError {}

/// Reads and fully prepares `path`.
pub fn load(path: &Path, options: &DecodeOptions) -> Result<DecodedImage, DecodeError> {
    let started = Instant::now();
    let bytes = std::fs::read(path).map_err(DecodeError::Read)?;
    let decoded = decode(&bytes, path, options)?;

    tracing::debug!(
        "{} -> decoded {}x{}{} in {}ms",
        decoded.file_name(),
        decoded.width(),
        decoded.height(),
        if decoded.is_full() {
            String::new()
        } else {
            format!(
                " (kept at {}x{})",
                decoded.surface.width, decoded.surface.height
            )
        },
        started.elapsed().as_millis()
    );

    Ok(decoded)
}

/// Runs the pipeline over bytes already in memory.
///
/// Split out from [`load`] so the pipeline is testable without touching disk.
pub fn decode(
    bytes: &[u8],
    path: &Path,
    options: &DecodeOptions,
) -> Result<DecodedImage, DecodeError> {
    let format = Format::from_path(path);
    let parsed = Metadata::parse(bytes, format);
    let (mut metadata, preview, pixels) = (parsed.metadata, parsed.preview, parsed.pixels);

    let mut image = match develop_raw(bytes, format, options) {
        Some(developed) => {
            // The developer applied the camera's orientation and handed back
            // sRGB, so neither is left for anyone else to do.
            metadata.already_developed();
            developed
        }
        // Otherwise a raw shows the preview the camera embedded, and every
        // other format decodes itself.
        None => match (preview, pixels) {
            (Some(preview), _) => codec::decode(preview, Some(Format::Jpeg))?,
            // A raw with no JPEG inside it at all: a DNG written by Camera
            // Raw carries its reduced-resolution copy as plain pixels, and
            // without this the file fell through to the TIFF decoder, which
            // reads the *first* directory — that same small copy — and
            // reported it as the photograph.
            (None, Some(pixels)) => codec::from_pixels(&pixels)
                .ok_or_else(|| DecodeError::Unsupported("unreadable raw preview".to_string()))?,
            (None, None) => codec::decode(bytes, format)?,
        },
    };

    // What the file could hold, not what these pixels happen to hold: a JPEG
    // and a raw cannot be transparent, and asking the pixels costs a pass over
    // all of them.
    let alpha = resize::Alpha::of(format);

    image = resize::to_max_edge(image, options.max_edge, alpha);
    color::convert(&mut image, &metadata, &options.output_profile);

    // A format that hands back upright pixels has nothing to turn, whatever
    // its metadata says.
    if format.is_some_and(Format::ignores_exif_orientation) {
        metadata.orientation = Orientation::Normal;
    }

    metadata.add_file_tags(path, bytes.len());

    let shown = (
        displayed_width(&image, metadata.orientation),
        displayed_height(&image, metadata.orientation),
    );

    metadata.add_size_tags(shown.0, shown.1);

    // A raw shown through an embedded copy is not the size of that copy.
    if let Some(full) = parsed.full_size {
        metadata.note_preview(shown, full);
    }

    Ok(into_decoded(image, metadata, options.display_edge, alpha))
}

/// Width the image is shown at, which a quarter turn swaps.
fn displayed_width(image: &RgbaImage, orientation: Orientation) -> u32 {
    if orientation.transposes() {
        image.height()
    } else {
        image.width()
    }
}

/// Height the image is shown at.
fn displayed_height(image: &RgbaImage, orientation: Orientation) -> u32 {
    if orientation.transposes() {
        image.width()
    } else {
        image.height()
    }
}

/// Develops a raw file, or returns nothing so the preview is used instead.
///
/// A failure is deliberately not fatal: a raw LibRaw cannot read still has a
/// preview worth showing, and a folder should not become unopenable because
/// one file in it is unusual.
fn develop_raw(bytes: &[u8], format: Option<Format>, options: &DecodeOptions) -> Option<RgbaImage> {
    if format != Some(Format::Raw) || !options.raw.develop {
        return None;
    }

    match raw::develop(bytes, &options.raw) {
        Ok(developed) => Some(developed),
        Err(e) => {
            tracing::warn!("Showing the embedded preview instead: {e}");
            None
        }
    }
}

fn into_decoded(
    image: RgbaImage,
    metadata: Metadata,
    display_edge: Option<u32>,
    alpha: resize::Alpha,
) -> DecodedImage {
    let full_size = (image.width(), image.height());

    // Reduced here, on the worker, rather than at upload time: there are eight
    // workers and one UI thread, and the UI thread is the one that has to keep
    // up with the user. The full sized buffer is dropped with `image`, which
    // is the point — a folder of these is what fills memory.
    let surface = match display_edge.and_then(|edge| resize::reduced(&image, edge, alpha)) {
        Some(reduced) => Surface::from_image(reduced),
        None => Surface::from_image(image),
    };

    DecodedImage {
        orientation: metadata.orientation,
        surface,
        full_size,
        metadata,
    }
}

#[cfg(test)]
pub(crate) mod test_support {
    use image::{DynamicImage, ImageFormat, Rgba, RgbaImage};
    use std::io::Cursor;

    /// Encodes a small solid colour image in `format`.
    ///
    /// JPEG has no alpha channel, so the image is flattened for it.
    pub fn encode(width: u32, height: u32, colour: [u8; 4], format: ImageFormat) -> Vec<u8> {
        let image = DynamicImage::from(RgbaImage::from_pixel(width, height, Rgba(colour)));
        let image = match format {
            ImageFormat::Jpeg => DynamicImage::from(image.into_rgb8()),
            _ => image,
        };

        let mut out = Cursor::new(Vec::new());
        image.write_to(&mut out, format).expect("encodes");
        out.into_inner()
    }
}

#[cfg(test)]
mod tests {
    use super::test_support::encode;
    use super::*;
    use ::image::ImageFormat;

    fn options() -> DecodeOptions {
        DecodeOptions::new(Arc::from("srgb"))
    }

    #[test]
    fn decodes_to_rgba() {
        let bytes = encode(4, 2, [10, 20, 30, 255], ImageFormat::Png);
        let decoded = decode(&bytes, Path::new("a.png"), &options()).unwrap();

        assert_eq!(decoded.size(), [4, 2]);
        assert_eq!(decoded.byte_len(), 4 * 2 * BYTES_PER_PIXEL);
        assert_eq!(&decoded.surface.pixels[..4], &[10, 20, 30, 255]);
    }

    #[test]
    fn records_file_and_size_tags() {
        let bytes = encode(6, 3, [0, 0, 0, 255], ImageFormat::Png);
        let decoded = decode(&bytes, Path::new("/photos/a.png"), &options()).unwrap();

        assert_eq!(decoded.file_name(), "a.png");
        assert_eq!(
            decoded.metadata.tags.get("Image Size").map(String::as_str),
            Some("6x3")
        );
    }

    #[test]
    fn the_camera_orientation_travels_with_the_pixels() {
        // A JPEG with no EXIF is upright, and the pixels are left alone.
        let bytes = encode(4, 2, [10, 20, 30, 255], ImageFormat::Png);
        let decoded = decode(&bytes, Path::new("a.png"), &options()).unwrap();

        assert_eq!(decoded.orientation, Orientation::Normal);
        assert_eq!(decoded.size(), [4, 2]);
    }

    #[test]
    fn honours_the_max_edge() {
        let bytes = encode(40, 20, [255, 255, 255, 255], ImageFormat::Png);
        let options = options().with_max_edge(Some(10));
        let decoded = decode(&bytes, Path::new("a.png"), &options).unwrap();

        assert_eq!(decoded.size(), [10, 5]);
    }

    #[test]
    fn reports_undecodable_files() {
        let error = decode(b"not an image", Path::new("a.png"), &options()).unwrap_err();
        assert!(matches!(error, DecodeError::Unsupported(_)));
    }

    #[test]
    fn reports_missing_files() {
        let error = load(Path::new("does-not-exist.png"), &options()).unwrap_err();
        assert!(matches!(error, DecodeError::Read(_)));
    }
}
