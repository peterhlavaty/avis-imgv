//! Downscaling, using SIMD accelerated resampling.

use fast_image_resize::images::Image as FirImage;
use fast_image_resize::{PixelType, ResizeOptions, Resizer};
use image::RgbaImage;

/// Shrinks `image` so its longest edge is at most `max_edge`.
///
/// Images already within the limit are returned untouched, and upscaling is
/// never done here: enlarging is the GPU's job at draw time.
pub fn to_max_edge(image: RgbaImage, max_edge: Option<u32>) -> RgbaImage {
    let Some(max_edge) = max_edge else {
        return image;
    };

    match target_size(image.width(), image.height(), max_edge) {
        Some((width, height)) => resample(image, width, height),
        None => image,
    }
}

/// Dimensions that fit `max_edge` while preserving the aspect ratio, or `None`
/// when the image already fits.
fn target_size(width: u32, height: u32, max_edge: u32) -> Option<(u32, u32)> {
    let longest = width.max(height);
    if longest <= max_edge || max_edge == 0 {
        return None;
    }

    let scale = max_edge as f64 / longest as f64;
    let target = (
        ((width as f64 * scale).round() as u32).max(1),
        ((height as f64 * scale).round() as u32).max(1),
    );

    Some(target)
}

/// Resamples into a new buffer, falling back to the original image if the
/// resizer refuses the request.
fn resample(image: RgbaImage, width: u32, height: u32) -> RgbaImage {
    let (source_width, source_height) = (image.width(), image.height());

    // `RgbaImage` guarantees the buffer matches its dimensions, so wrapping it
    // cannot fail; the branch exists only to avoid an unwrap.
    let source = match FirImage::from_vec_u8(
        source_width,
        source_height,
        image.into_raw(),
        PixelType::U8x4,
    ) {
        Ok(source) => source,
        Err(e) => {
            tracing::error!("Failure wrapping image for resize -> {e}");
            return RgbaImage::new(source_width, source_height);
        }
    };

    let mut destination = FirImage::new(width, height, PixelType::U8x4);

    // Our alpha is opaque for every format that matters here, and the
    // premultiply round trip costs two full passes over the pixels.
    let options = ResizeOptions::new().use_alpha(false);

    let buffer = match Resizer::new().resize(&source, &mut destination, &options) {
        Ok(()) => (width, height, destination.into_vec()),
        Err(e) => {
            tracing::error!("Failure resizing image -> {e}");
            (source_width, source_height, source.into_vec())
        }
    };

    RgbaImage::from_raw(buffer.0, buffer.1, buffer.2)
        .unwrap_or_else(|| RgbaImage::new(source_width, source_height))
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;

    fn image(width: u32, height: u32) -> RgbaImage {
        RgbaImage::from_pixel(width, height, Rgba([12, 34, 56, 255]))
    }

    #[test]
    fn keeps_the_aspect_ratio() {
        let resized = to_max_edge(image(400, 200), Some(100));
        assert_eq!((resized.width(), resized.height()), (100, 50));

        let resized = to_max_edge(image(200, 400), Some(100));
        assert_eq!((resized.width(), resized.height()), (50, 100));
    }

    #[test]
    fn never_upscales() {
        let resized = to_max_edge(image(40, 20), Some(100));
        assert_eq!((resized.width(), resized.height()), (40, 20));
    }

    #[test]
    fn no_limit_is_a_no_op() {
        let resized = to_max_edge(image(40, 20), None);
        assert_eq!((resized.width(), resized.height()), (40, 20));
    }

    #[test]
    fn extreme_ratios_keep_at_least_one_pixel() {
        let resized = to_max_edge(image(1000, 1), Some(10));
        assert_eq!((resized.width(), resized.height()), (10, 1));
    }

    #[test]
    fn preserves_colour() {
        let resized = to_max_edge(image(40, 40), Some(10));
        assert_eq!(resized.get_pixel(5, 5), &Rgba([12, 34, 56, 255]));
    }
}
