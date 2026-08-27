//! Hand written bindings to the LibRaw C API.
//!
//! Only the C API is used, never the C++ classes: it is a small, stable
//! surface, and it lets `libraw_data_t` stay an opaque pointer. That matters,
//! because the layout of that struct changes between LibRaw releases while the
//! setters and getters do not.
//!
//! Every unsafe block in the raw decoder is in this file; [`Processor`] is the
//! safe interface the rest of the module uses.

use std::ffi::CStr;
use std::fmt;
use std::os::raw::{c_char, c_float, c_int, c_uint, c_void};
use std::ptr::NonNull;

/// LibRaw's handle. Opaque on purpose — see the module comment.
#[repr(C)]
pub struct DataHandle {
    _private: [u8; 0],
}

/// A developed image, still owned by LibRaw.
///
/// `data` is a C flexible array member; its real length is `data_size`.
#[repr(C)]
struct ProcessedImage {
    /// `LibRaw_image_formats`: 1 is a JPEG, 2 is a bitmap.
    kind: c_int,
    height: u16,
    width: u16,
    colors: u16,
    bits: u16,
    data_size: c_uint,
    data: [u8; 1],
}

/// `LIBRAW_IMAGE_BITMAP`, the only form we ask for.
const IMAGE_BITMAP: c_int = 2;

/// `LIBRAW_SUCCESS`.
const SUCCESS: c_int = 0;

// The library to link against is chosen by build.rs, which knows where LibRaw
// was found and whether it is the thread safe build.
unsafe extern "C" {
    fn libraw_init(flags: c_uint) -> *mut DataHandle;
    fn libraw_close(handle: *mut DataHandle);
    fn libraw_open_buffer(handle: *mut DataHandle, buffer: *const c_void, size: usize) -> c_int;
    fn libraw_unpack(handle: *mut DataHandle) -> c_int;
    fn libraw_dcraw_process(handle: *mut DataHandle) -> c_int;
    fn libraw_dcraw_make_mem_image(
        handle: *mut DataHandle,
        error: *mut c_int,
    ) -> *mut ProcessedImage;
    fn libraw_dcraw_clear_mem(image: *mut ProcessedImage);
    fn libraw_strerror(code: c_int) -> *const c_char;
    fn libraw_version() -> *const c_char;

    fn libraw_set_demosaic(handle: *mut DataHandle, value: c_int);
    fn libraw_set_output_color(handle: *mut DataHandle, value: c_int);
    fn libraw_set_output_bps(handle: *mut DataHandle, value: c_int);
    fn libraw_set_no_auto_bright(handle: *mut DataHandle, value: c_int);
    fn libraw_set_highlight(handle: *mut DataHandle, value: c_int);
    fn libraw_set_user_mul(handle: *mut DataHandle, index: c_int, value: c_float);
    fn libraw_get_cam_mul(handle: *mut DataHandle, index: c_int) -> c_float;
}

/// Whatever LibRaw had to say about a failure.
#[derive(Debug)]
pub struct Error {
    pub code: i32,
    pub message: String,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} (LibRaw {})", self.message, self.code)
    }
}

impl Error {
    fn from_code(code: c_int) -> Error {
        // Safety: LibRaw returns a static string for any code.
        let message = unsafe { CStr::from_ptr(libraw_strerror(code)) }
            .to_string_lossy()
            .into_owned();

        Error { code, message }
    }

    fn other(message: &str) -> Error {
        Error {
            code: 0,
            message: message.to_string(),
        }
    }
}

/// The version of LibRaw this was linked against.
pub fn version() -> String {
    // Safety: LibRaw returns a static string.
    unsafe { CStr::from_ptr(libraw_version()) }
        .to_string_lossy()
        .into_owned()
}

/// One LibRaw instance, which is one image being developed.
///
/// Instances share nothing, so every decode worker can hold its own.
pub struct Processor {
    handle: NonNull<DataHandle>,
}

impl Processor {
    pub fn new() -> Result<Processor, Error> {
        // Safety: no flags are passed, and the result is checked for null.
        let handle = unsafe { libraw_init(0) };

        NonNull::new(handle)
            .map(|handle| Processor { handle })
            .ok_or_else(|| Error::other("could not create a LibRaw instance"))
    }

    /// Reads the raw file held in `bytes`.
    ///
    /// LibRaw copies what it needs, so the buffer does not have to outlive the
    /// call.
    pub fn open(&mut self, bytes: &[u8]) -> Result<(), Error> {
        // Safety: the pointer and length describe a slice borrowed for the
        // duration of the call.
        check(unsafe {
            libraw_open_buffer(
                self.handle.as_ptr(),
                bytes.as_ptr().cast::<c_void>(),
                bytes.len(),
            )
        })
    }

    /// Decodes the sensor data. Must follow [`Processor::open`].
    pub fn unpack(&mut self) -> Result<(), Error> {
        // Safety: the handle is valid until this is dropped.
        check(unsafe { libraw_unpack(self.handle.as_ptr()) })
    }

    /// Demosaics and converts. Must follow [`Processor::unpack`].
    pub fn process(&mut self) -> Result<(), Error> {
        // Safety: as above.
        check(unsafe { libraw_dcraw_process(self.handle.as_ptr()) })
    }

    /// Takes the developed image out of LibRaw.
    pub fn take_image(&mut self) -> Result<Image, Error> {
        let mut error = SUCCESS;

        // Safety: `error` is written by LibRaw, and the returned pointer is
        // checked before it is used.
        let image = unsafe { libraw_dcraw_make_mem_image(self.handle.as_ptr(), &mut error) };

        let Some(image) = NonNull::new(image) else {
            return Err(Error::from_code(error));
        };

        let image = Image { image };
        if image.header().kind != IMAGE_BITMAP {
            return Err(Error::other("LibRaw returned a JPEG rather than a bitmap"));
        }

        Ok(image)
    }

    /// Uses the white balance the camera recorded rather than a fixed one.
    ///
    /// LibRaw's C API has no setter for `use_camera_wb`, so the camera's own
    /// multipliers are read back and set as the user ones, which comes to the
    /// same thing.
    pub fn use_camera_white_balance(&mut self) {
        for index in 0..MULTIPLIERS {
            // Safety: four multipliers is the size of the array in every
            // LibRaw version, and both calls bound the index themselves.
            unsafe {
                let multiplier = libraw_get_cam_mul(self.handle.as_ptr(), index);
                libraw_set_user_mul(self.handle.as_ptr(), index, multiplier);
            }
        }
    }

    /// Demosaic algorithm: 0 is bilinear, 2 is PPG, 3 is AHD.
    pub fn set_demosaic(&mut self, algorithm: i32) {
        // Safety: LibRaw falls back to its default for an unknown value.
        unsafe { libraw_set_demosaic(self.handle.as_ptr(), algorithm) }
    }

    /// Output colour space: 0 is raw, 1 is sRGB, 2 is Adobe RGB.
    pub fn set_output_color(&mut self, space: i32) {
        // Safety: as above.
        unsafe { libraw_set_output_color(self.handle.as_ptr(), space) }
    }

    /// Bits per sample of the developed image: 8 or 16.
    pub fn set_output_bits(&mut self, bits: i32) {
        // Safety: as above.
        unsafe { libraw_set_output_bps(self.handle.as_ptr(), bits) }
    }

    /// Whether to stretch the histogram to use the whole range.
    pub fn set_auto_brighten(&mut self, brighten: bool) {
        // Safety: as above.
        unsafe { libraw_set_no_auto_bright(self.handle.as_ptr(), i32::from(!brighten)) }
    }

    /// Highlight handling: 0 clips, 1 leaves them unclipped, 2 blends, and 3
    /// upwards rebuild.
    pub fn set_highlight_mode(&mut self, mode: i32) {
        // Safety: as above.
        unsafe { libraw_set_highlight(self.handle.as_ptr(), mode) }
    }
}

/// Size of LibRaw's white balance multiplier array.
const MULTIPLIERS: c_int = 4;

impl Drop for Processor {
    fn drop(&mut self) {
        // Safety: the handle came from `libraw_init` and is closed exactly
        // once, here.
        unsafe { libraw_close(self.handle.as_ptr()) }
    }
}

// A processor owns its LibRaw instance outright and shares nothing with any
// other, so it may be moved to whichever worker decodes the image.
unsafe impl Send for Processor {}

/// A developed image, freed when dropped.
pub struct Image {
    image: NonNull<ProcessedImage>,
}

impl Image {
    fn header(&self) -> &ProcessedImage {
        // Safety: LibRaw owns the allocation until `Drop` frees it, which
        // outlives every borrow of `self`.
        unsafe { self.image.as_ref() }
    }

    pub fn width(&self) -> u32 {
        u32::from(self.header().width)
    }

    pub fn height(&self) -> u32 {
        u32::from(self.header().height)
    }

    /// Samples per pixel; three for a developed image.
    pub fn colors(&self) -> u32 {
        u32::from(self.header().colors)
    }

    /// Bits per sample.
    pub fn bits(&self) -> u32 {
        u32::from(self.header().bits)
    }

    /// The pixels, tightly packed.
    pub fn data(&self) -> &[u8] {
        let header = self.header();

        // Safety: `data` is a flexible array member of `data_size` bytes,
        // which is how LibRaw both declares and allocates it.
        unsafe { std::slice::from_raw_parts(header.data.as_ptr(), header.data_size as usize) }
    }
}

impl Drop for Image {
    fn drop(&mut self) {
        // Safety: the pointer came from `libraw_dcraw_make_mem_image` and is
        // freed exactly once, here.
        unsafe { libraw_dcraw_clear_mem(self.image.as_ptr()) }
    }
}

unsafe impl Send for Image {}

/// Turns a LibRaw return code into a result.
fn check(code: c_int) -> Result<(), Error> {
    if code == SUCCESS {
        Ok(())
    } else {
        Err(Error::from_code(code))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_library_reports_its_version() {
        let version = version();

        assert!(version.starts_with(char::is_numeric), "got {version:?}");
    }

    #[test]
    fn an_instance_can_be_created_and_dropped() {
        let processor = Processor::new().expect("a LibRaw instance");

        drop(processor);
    }

    #[test]
    fn opening_something_that_is_not_a_raw_file_fails() {
        let mut processor = Processor::new().unwrap();
        let error = processor.open(b"this is not a raw file").unwrap_err();

        assert!(!error.message.is_empty(), "{error}");
    }

    #[test]
    fn an_empty_buffer_is_refused_rather_than_read() {
        let mut processor = Processor::new().unwrap();

        assert!(processor.open(&[]).is_err());
    }

    #[test]
    fn settings_apply_before_anything_is_opened() {
        let mut processor = Processor::new().unwrap();

        processor.set_demosaic(2);
        processor.set_output_color(1);
        processor.set_output_bits(8);
        processor.set_auto_brighten(false);
        processor.set_highlight_mode(0);
        processor.use_camera_white_balance();
    }

    #[test]
    fn developing_without_opening_anything_fails() {
        let mut processor = Processor::new().unwrap();

        assert!(processor.unpack().is_err());
    }
}
