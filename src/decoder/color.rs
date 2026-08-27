//! Colour management: converting an image into the display's profile.

use image::RgbaImage;
use lcms2::{Intent, PixelFormat, Profile, Transform};

use crate::metadata::{icc, Metadata};

/// Converts `image` in place from its embedded profile into `output_profile`.
///
/// Does nothing when the image carries no profile information, or when it is
/// already in the output profile — the common case, and skipping it saves a
/// full pass over the pixels.
pub fn convert(image: &mut RgbaImage, metadata: &Metadata, output_profile: &str) {
    let Some(description) = metadata.profile_description() else {
        return;
    };

    if description
        .to_lowercase()
        .contains(&output_profile.to_lowercase())
    {
        tracing::trace!("Input and output profiles both are {description}, skipping conversion");
        return;
    }

    let Some(input) = input_profile(metadata, description) else {
        return;
    };

    let Some(output) = icc::built_in(output_profile).and_then(open) else {
        tracing::error!("Badly configured output ICC profile -> {output_profile}");
        return;
    };

    let transform = match Transform::new(
        &input,
        PixelFormat::RGBA_8,
        &output,
        PixelFormat::RGBA_8,
        Intent::Perceptual,
    ) {
        Ok(transform) => transform,
        Err(e) => {
            tracing::error!("Failure building ICC transform -> {e}");
            return;
        }
    };

    // lcms2 accepts a flat `[u8]` for any pixel format as long as its length
    // is a whole number of pixels, which RGBA8 always is.
    let pixels: &mut [u8] = image;
    transform.transform_in_place(pixels);
}

/// Prefers the profile embedded in the file, falling back to the closest
/// profile we ship for the name the metadata reports.
fn input_profile(metadata: &Metadata, description: &str) -> Option<Profile> {
    metadata.icc.as_deref().and_then(open).or_else(|| {
        tracing::debug!("No usable embedded ICC profile, matching {description} by name");
        icc::built_in(description).and_then(open)
    })
}

fn open(bytes: &[u8]) -> Option<Profile> {
    Profile::new_icc(bytes)
        .map_err(|e| tracing::error!("Failure reading ICC profile -> {e}"))
        .ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;
    use std::collections::BTreeMap;

    fn metadata_with(description: &str, embedded: Option<&[u8]>) -> Metadata {
        Metadata {
            tags: BTreeMap::from([(
                crate::metadata::PROFILE_DESCRIPTION.to_string(),
                description.to_string(),
            )]),
            icc: embedded.map(<[u8]>::to_vec),
            ..Default::default()
        }
    }

    fn probe() -> RgbaImage {
        RgbaImage::from_pixel(2, 2, Rgba([200, 100, 50, 255]))
    }

    #[test]
    fn matching_profiles_are_left_untouched() {
        let mut image = probe();
        convert(
            &mut image,
            &metadata_with("sRGB IEC61966-2.1", None),
            "srgb",
        );
        assert_eq!(image.get_pixel(0, 0), &Rgba([200, 100, 50, 255]));
    }

    #[test]
    fn images_without_profile_information_are_left_untouched() {
        let mut image = probe();
        convert(&mut image, &Metadata::default(), "srgb");
        assert_eq!(image.get_pixel(0, 0), &Rgba([200, 100, 50, 255]));
    }

    #[test]
    fn a_known_wide_gamut_profile_is_converted() {
        let mut image = probe();
        convert(&mut image, &metadata_with("Adobe RGB (1998)", None), "srgb");

        let pixel = image.get_pixel(0, 0);
        assert_ne!(pixel, &Rgba([200, 100, 50, 255]), "colours should shift");
        assert_eq!(pixel[3], 255, "alpha must survive the transform");
    }

    #[test]
    fn an_embedded_profile_is_preferred() {
        let mut image = probe();
        let metadata = metadata_with("Some Vendor Profile", Some(icc::CLAY_RGB));
        convert(&mut image, &metadata, "srgb");

        assert_ne!(image.get_pixel(0, 0), &Rgba([200, 100, 50, 255]));
    }

    #[test]
    fn an_unknown_profile_name_is_a_no_op() {
        let mut image = probe();
        convert(&mut image, &metadata_with("ProPhoto RGB", None), "srgb");
        assert_eq!(image.get_pixel(0, 0), &Rgba([200, 100, 50, 255]));
    }

    #[test]
    fn a_corrupt_embedded_profile_falls_back_to_the_name() {
        let mut image = probe();
        let metadata = metadata_with("Adobe RGB (1998)", Some(b"not a profile"));
        convert(&mut image, &metadata, "srgb");

        assert_ne!(image.get_pixel(0, 0), &Rgba([200, 100, 50, 255]));
    }
}
