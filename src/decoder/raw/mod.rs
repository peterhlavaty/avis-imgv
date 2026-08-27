//! Developing camera raw files with LibRaw.
//!
//! A raw file holds a sensor mosaic, not a picture. Every camera also embeds a
//! JPEG preview, which is what the viewer shows by default because it costs
//! almost nothing; developing the sensor data instead gives the full
//! resolution and the full dynamic range, at the cost of a second or so per
//! image.
//!
//! Which one you get is [`Options::develop`].

#[cfg(feature = "libraw")]
mod ffi;

use image::RgbaImage;

use super::DecodeError;

/// How much work to spend turning the sensor mosaic into pixels.
///
/// The difference between these is several times the decoding time, which for
/// a folder of raws is the difference between waiting and not.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Demosaic {
    /// Bilinear interpolation: quick, and visibly softer.
    Fast,
    /// Patterned pixel grouping, a good compromise.
    #[default]
    Balanced,
    /// Adaptive homogeneity-directed, which is LibRaw's own default.
    Best,
}

impl Demosaic {
    /// The number LibRaw knows this algorithm by.
    pub fn algorithm(self) -> i32 {
        match self {
            Demosaic::Fast => 0,
            Demosaic::Balanced => 2,
            Demosaic::Best => 3,
        }
    }
}

/// What the developer should do with a raw file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Options {
    /// Develop the sensor data rather than showing the embedded preview.
    pub develop: bool,
    pub demosaic: Demosaic,
    /// Use the white balance the camera recorded. Without it colours come out
    /// noticeably wrong, so it is on unless you have a reason.
    pub camera_white_balance: bool,
    /// Stretch the histogram to use the whole range.
    pub auto_brighten: bool,
    /// 0 clips blown highlights, 1 leaves them unclipped, 2 blends, and 3
    /// upwards rebuild them.
    pub highlight_mode: u8,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            develop: false,
            demosaic: Demosaic::default(),
            camera_white_balance: true,
            auto_brighten: true,
            highlight_mode: 0,
        }
    }
}

/// Whether this build can develop raw files at all.
pub const fn available() -> bool {
    cfg!(feature = "libraw")
}

/// The LibRaw this was built against, for the log line at startup.
pub fn version() -> Option<String> {
    #[cfg(feature = "libraw")]
    {
        Some(ffi::version())
    }

    #[cfg(not(feature = "libraw"))]
    {
        None
    }
}

/// Develops the raw file in `bytes` into pixels.
///
/// LibRaw applies the camera's orientation itself and is asked for sRGB, so
/// what comes back needs neither of the pipeline's later steps.
#[cfg(feature = "libraw")]
pub fn develop(bytes: &[u8], options: &Options) -> Result<RgbaImage, DecodeError> {
    let mut processor = ffi::Processor::new().map_err(failed)?;

    // Eight bits is what the GPU takes, and asking for sixteen would only
    // double the work and the memory to throw half of it away.
    processor.set_output_bits(8);
    // sRGB, so the pipeline's colour conversion handles the rest as it does
    // for every other format.
    processor.set_output_color(1);
    processor.set_demosaic(options.demosaic.algorithm());
    processor.set_auto_brighten(options.auto_brighten);
    processor.set_highlight_mode(i32::from(options.highlight_mode));

    processor.open(bytes).map_err(failed)?;
    processor.unpack().map_err(failed)?;

    // The camera's multipliers are only known once the file has been read.
    if options.camera_white_balance {
        processor.use_camera_white_balance();
    }

    processor.process().map_err(failed)?;
    let developed = processor.take_image().map_err(failed)?;

    into_rgba(&developed)
}

#[cfg(not(feature = "libraw"))]
pub fn develop(_bytes: &[u8], _options: &Options) -> Result<RgbaImage, DecodeError> {
    Err(DecodeError::Unsupported(
        "developing raw files needs LibRaw; build with --features libraw".into(),
    ))
}

/// Copies LibRaw's output into the layout the rest of the pipeline uses.
#[cfg(feature = "libraw")]
fn into_rgba(developed: &ffi::Image) -> Result<RgbaImage, DecodeError> {
    let (width, height) = (developed.width(), developed.height());
    let pixels = developed.data();

    if developed.bits() != 8 {
        return Err(unexpected(format!(
            "{} bits per sample, expected 8",
            developed.bits()
        )));
    }

    let image = match developed.colors() {
        3 => image::RgbImage::from_raw(width, height, pixels.to_vec())
            .map(|rgb| image::DynamicImage::from(rgb).into_rgba8()),
        1 => image::GrayImage::from_raw(width, height, pixels.to_vec())
            .map(|gray| image::DynamicImage::from(gray).into_rgba8()),
        _ => None,
    };

    image.ok_or_else(|| {
        unexpected(format!(
            "{}x{} with {} channels in {} bytes",
            width,
            height,
            developed.colors(),
            pixels.len()
        ))
    })
}

#[cfg(feature = "libraw")]
fn failed(error: ffi::Error) -> DecodeError {
    DecodeError::Unsupported(error.to_string())
}

#[cfg(feature = "libraw")]
fn unexpected(what: String) -> DecodeError {
    DecodeError::Unsupported(format!("LibRaw produced {what}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_demosaic_settings_map_onto_libraws_numbers() {
        assert_eq!(Demosaic::Fast.algorithm(), 0);
        assert_eq!(Demosaic::Balanced.algorithm(), 2);
        assert_eq!(Demosaic::Best.algorithm(), 3);
    }

    #[test]
    fn the_defaults_show_the_preview_with_camera_colours() {
        let options = Options::default();

        assert!(!options.develop, "developing is the slow path, so opt in");
        assert!(options.camera_white_balance);
        assert_eq!(options.demosaic, Demosaic::Balanced);
    }

    #[test]
    fn something_that_is_not_a_raw_file_is_refused() {
        let error = develop(b"this is not a raw file", &Options::default()).unwrap_err();

        assert!(matches!(error, DecodeError::Unsupported(_)));
    }

    #[test]
    fn an_empty_buffer_is_refused() {
        assert!(develop(&[], &Options::default()).is_err());
    }

    #[test]
    fn the_build_says_whether_it_can_develop() {
        assert_eq!(available(), version().is_some());
    }
}
