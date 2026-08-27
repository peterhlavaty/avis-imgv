//! Byte buffers to pixels.

use image::{ImageFormat, RgbaImage};

use super::DecodeError;
use crate::formats::Format;

/// Decodes `bytes` into RGBA8.
///
/// `format` is only a hint: `image` sniffs the content itself, and a mismatched
/// extension is common enough that guessing is worth it but trusting is not.
pub fn decode(bytes: &[u8], format: Option<Format>) -> Result<RgbaImage, DecodeError> {
    if format == Some(Format::JpegXl) {
        return decode_jpeg_xl(bytes);
    }

    let decoded = match format.and_then(image_format) {
        Some(format) => image::load_from_memory_with_format(bytes, format)
            .or_else(|_| image::load_from_memory(bytes)),
        None => image::load_from_memory(bytes),
    };

    match decoded {
        Ok(image) => Ok(image.into_rgba8()),
        Err(e) => Err(DecodeError::Unsupported(e.to_string())),
    }
}

/// Maps our container classification onto the `image` crate's.
fn image_format(format: Format) -> Option<ImageFormat> {
    match format {
        Format::Jpeg => Some(ImageFormat::Jpeg),
        Format::Png => Some(ImageFormat::Png),
        Format::Webp => Some(ImageFormat::WebP),
        Format::Gif => Some(ImageFormat::Gif),
        Format::Bmp => Some(ImageFormat::Bmp),
        Format::Tiff => Some(ImageFormat::Tiff),
        // Raw files reach the decoder as their extracted JPEG preview.
        Format::Raw => Some(ImageFormat::Jpeg),
        Format::JpegXl => None,
    }
}

/// Decodes JPEG XL through libjxl.
///
/// Single threaded on purpose: the loader already decodes one image per
/// thread, so a parallel runner here would only fight it for cores.
#[cfg(feature = "jxl")]
fn decode_jpeg_xl(bytes: &[u8]) -> Result<RgbaImage, DecodeError> {
    let decoder = jpegxl_rs::decoder_builder()
        .build()
        .map_err(|e| DecodeError::Unsupported(format!("JPEG XL decoder: {e}")))?;

    let (metadata, pixels) = decoder
        .decode_with::<u8>(bytes)
        .map_err(|e| DecodeError::Unsupported(format!("JPEG XL: {e}")))?;

    let channels = pixels.len() / (metadata.width as usize * metadata.height as usize).max(1);

    let rgba = match channels {
        4 => RgbaImage::from_raw(metadata.width, metadata.height, pixels),
        3 => image::RgbImage::from_raw(metadata.width, metadata.height, pixels)
            .map(|rgb| image::DynamicImage::from(rgb).into_rgba8()),
        _ => None,
    };

    rgba.ok_or_else(|| DecodeError::Unsupported("unexpected JPEG XL channel layout".into()))
}

#[cfg(not(feature = "jxl"))]
fn decode_jpeg_xl(_bytes: &[u8]) -> Result<RgbaImage, DecodeError> {
    Err(DecodeError::Unsupported(
        "JPEG XL support is not compiled in; build with --features jxl".into(),
    ))
}

#[cfg(test)]
mod tests {
    use super::super::test_support::encode;
    use super::*;

    #[test]
    fn decodes_the_formats_we_advertise() {
        for (format, image_format) in [
            (Format::Png, ImageFormat::Png),
            (Format::Jpeg, ImageFormat::Jpeg),
            (Format::Bmp, ImageFormat::Bmp),
        ] {
            let bytes = encode(8, 4, [200, 100, 50, 255], image_format);
            let decoded = decode(&bytes, Some(format)).expect("decodes");
            assert_eq!((decoded.width(), decoded.height()), (8, 4));
        }
    }

    #[test]
    fn a_wrong_extension_still_decodes() {
        let bytes = encode(8, 4, [1, 2, 3, 255], ImageFormat::Png);
        assert!(decode(&bytes, Some(Format::Jpeg)).is_ok());
    }

    #[test]
    fn garbage_is_reported_not_panicked_on() {
        assert!(decode(b"", None).is_err());
        assert!(decode(b"garbage bytes here", Some(Format::Png)).is_err());
    }

    #[cfg(not(feature = "jxl"))]
    #[test]
    fn jpeg_xl_reports_the_missing_feature() {
        let error = decode(b"\xff\x0a", Some(Format::JpegXl)).unwrap_err();
        assert!(error.to_string().contains("--features jxl"));
    }
}
