//! Typed TIFF/EXIF field values.

use std::fmt::Write as _;

use super::bytes::{Cursor, Endian};

/// TIFF field type as stored in an IFD entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldType {
    Byte,
    Ascii,
    Short,
    Long,
    Rational,
    SByte,
    Undefined,
    SShort,
    SLong,
    SRational,
    Float,
    Double,
}

impl FieldType {
    pub fn from_code(code: u16) -> Option<FieldType> {
        Some(match code {
            1 => FieldType::Byte,
            2 => FieldType::Ascii,
            3 => FieldType::Short,
            4 => FieldType::Long,
            5 => FieldType::Rational,
            6 => FieldType::SByte,
            7 => FieldType::Undefined,
            8 => FieldType::SShort,
            9 => FieldType::SLong,
            10 => FieldType::SRational,
            11 => FieldType::Float,
            12 => FieldType::Double,
            _ => return None,
        })
    }

    /// Size in bytes of a single component of this type.
    pub fn size(self) -> usize {
        match self {
            FieldType::Byte | FieldType::Ascii | FieldType::SByte | FieldType::Undefined => 1,
            FieldType::Short | FieldType::SShort => 2,
            FieldType::Long | FieldType::SLong | FieldType::Float => 4,
            FieldType::Rational | FieldType::SRational | FieldType::Double => 8,
        }
    }
}

/// A decoded TIFF field value.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// `Byte`, `Short` and `Long` all widen into `u32`; callers rarely care
    /// which of the three was on disk.
    Unsigned(Vec<u32>),
    Signed(Vec<i32>),
    Rational(Vec<(u32, u32)>),
    SRational(Vec<(i32, i32)>),
    Float(Vec<f64>),
    Ascii(String),
    Undefined(Vec<u8>),
}

impl Value {
    /// Decodes `count` components of `field_type` starting at `offset`.
    pub fn decode(
        cursor: &Cursor<'_>,
        field_type: FieldType,
        count: usize,
        offset: usize,
    ) -> Option<Value> {
        let total = field_type.size().checked_mul(count)?;
        let bytes = cursor.slice(offset, total)?;
        let endian = cursor.endian();

        let value = match field_type {
            FieldType::Byte => Value::Unsigned(bytes.iter().map(|b| *b as u32).collect()),
            FieldType::SByte => Value::Signed(bytes.iter().map(|b| *b as i8 as i32).collect()),
            FieldType::Undefined => Value::Undefined(bytes.to_vec()),
            FieldType::Ascii => Value::Ascii(decode_ascii(bytes)),
            FieldType::Short => Value::Unsigned(
                bytes
                    .as_chunks::<2>()
                    .0
                    .iter()
                    .map(|c| endian.u16(*c) as u32)
                    .collect(),
            ),
            FieldType::SShort => Value::Signed(
                bytes
                    .as_chunks::<2>()
                    .0
                    .iter()
                    .map(|c| endian.u16(*c) as i16 as i32)
                    .collect(),
            ),
            FieldType::Long => Value::Unsigned(
                bytes
                    .as_chunks::<4>()
                    .0
                    .iter()
                    .map(|c| endian.u32(*c))
                    .collect(),
            ),
            FieldType::SLong => Value::Signed(
                bytes
                    .as_chunks::<4>()
                    .0
                    .iter()
                    .map(|c| endian.u32(*c) as i32)
                    .collect(),
            ),
            FieldType::Float => Value::Float(
                bytes
                    .as_chunks::<4>()
                    .0
                    .iter()
                    .map(|c| f32::from_bits(endian.u32(*c)) as f64)
                    .collect(),
            ),
            FieldType::Double => Value::Float(
                bytes
                    .as_chunks::<8>()
                    .0
                    .iter()
                    .map(|c| {
                        let (first, second) = c.split_at(4);
                        let hi = endian.u32(first.try_into().unwrap_or_default()) as u64;
                        let lo = endian.u32(second.try_into().unwrap_or_default()) as u64;
                        match endian {
                            Endian::Little => f64::from_bits((lo << 32) | hi),
                            Endian::Big => f64::from_bits((hi << 32) | lo),
                        }
                    })
                    .collect(),
            ),
            FieldType::Rational => Value::Rational(
                bytes
                    .as_chunks::<8>()
                    .0
                    .iter()
                    .map(|c| rational(endian, c))
                    .collect(),
            ),
            FieldType::SRational => Value::SRational(
                bytes
                    .as_chunks::<8>()
                    .0
                    .iter()
                    .map(|c| {
                        let (numerator, denominator) = rational(endian, c);
                        (numerator as i32, denominator as i32)
                    })
                    .collect(),
            ),
        };

        Some(value)
    }

    /// First component as an unsigned integer, when the value holds one.
    pub fn as_u32(&self) -> Option<u32> {
        match self {
            Value::Unsigned(v) => v.first().copied(),
            Value::Signed(v) => v.first().and_then(|n| u32::try_from(*n).ok()),
            Value::Rational(v) => v.first().map(|(n, _)| *n),
            Value::Ascii(s) => s.trim().parse().ok(),
            _ => None,
        }
    }

    /// First component as a real number, when the value holds one.
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Value::Unsigned(v) => v.first().map(|n| *n as f64),
            Value::Signed(v) => v.first().map(|n| *n as f64),
            Value::Float(v) => v.first().copied(),
            Value::Rational(v) => v.first().and_then(|(n, d)| ratio(*n as f64, *d as f64)),
            Value::SRational(v) => v.first().and_then(|(n, d)| ratio(*n as f64, *d as f64)),
            Value::Ascii(s) => s.trim().parse().ok(),
            _ => None,
        }
    }

    /// The string behind an `Ascii` value, trimmed.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::Ascii(s) => Some(s.trim()),
            _ => None,
        }
    }

    /// Raw bytes behind a `Byte`/`Undefined` value.
    pub fn as_bytes(&self) -> Option<&[u8]> {
        match self {
            Value::Undefined(b) => Some(b),
            _ => None,
        }
    }

    /// Number of components held.
    pub fn len(&self) -> usize {
        match self {
            Value::Unsigned(v) => v.len(),
            Value::Signed(v) => v.len(),
            Value::Rational(v) => v.len(),
            Value::SRational(v) => v.len(),
            Value::Float(v) => v.len(),
            Value::Ascii(s) => s.len(),
            Value::Undefined(b) => b.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// All unsigned components, for tags such as `SubIFDs` or `StripOffsets`
    /// that hold a list of offsets.
    pub fn unsigned_list(&self) -> Vec<u32> {
        match self {
            Value::Unsigned(v) => v.clone(),
            Value::Signed(v) => v.iter().filter_map(|n| u32::try_from(*n).ok()).collect(),
            _ => vec![],
        }
    }

    /// Generic human readable rendering, used for tags without a dedicated
    /// formatter.
    pub fn to_display_string(&self) -> String {
        match self {
            Value::Ascii(s) => s.trim().to_string(),
            Value::Unsigned(v) => join(v.iter().map(|n| n.to_string())),
            Value::Signed(v) => join(v.iter().map(|n| n.to_string())),
            Value::Float(v) => join(v.iter().map(|n| format_f64(*n))),
            Value::Rational(v) => join(
                v.iter()
                    .map(|(n, d)| format_ratio(*n as f64, *d as f64, *n, *d)),
            ),
            Value::SRational(v) => join(
                v.iter()
                    .map(|(n, d)| format_ratio(*n as f64, *d as f64, *n, *d)),
            ),
            Value::Undefined(b) => format!("({} bytes binary data)", b.len()),
        }
    }
}

/// Splits an eight byte field into its numerator and denominator.
fn rational(endian: Endian, bytes: &[u8; 8]) -> (u32, u32) {
    let (numerator, denominator) = bytes.split_at(4);

    (
        endian.u32(numerator.try_into().unwrap_or_default()),
        endian.u32(denominator.try_into().unwrap_or_default()),
    )
}

fn ratio(numerator: f64, denominator: f64) -> Option<f64> {
    if denominator == 0.0 {
        None
    } else {
        Some(numerator / denominator)
    }
}

fn join(parts: impl Iterator<Item = String>) -> String {
    let mut out = String::new();
    for (i, part) in parts.enumerate() {
        if i > 0 {
            out.push(' ');
        }
        out.push_str(&part);
    }
    out
}

fn format_ratio<T: std::fmt::Display>(n: f64, d: f64, raw_n: T, raw_d: T) -> String {
    match ratio(n, d) {
        Some(v) => format_f64(v),
        None => format!("{raw_n}/{raw_d}"),
    }
}

/// Formats a float the way exiftool does: no trailing zeroes, no exponent for
/// the magnitudes found in image metadata.
pub fn format_f64(value: f64) -> String {
    if value == value.trunc() && value.abs() < 1e15 {
        return format!("{}", value as i64);
    }

    let mut s = format!("{value:.6}");
    while s.ends_with('0') {
        s.pop();
    }
    if s.ends_with('.') {
        s.pop();
    }
    s
}

/// Decodes a NUL terminated, possibly NUL padded ASCII field.
///
/// EXIF says ASCII and nobody writes only ASCII. Cameras write Latin-1, and
/// Adobe's software writes UTF-8 — so UTF-8 is tried first and Latin-1 is what
/// it falls back to. That order is the safe one: pure ASCII is valid UTF-8 and
/// decodes the same either way, and Latin-1 text with an accent in it is
/// almost never valid UTF-8, so it still lands in the fallback.
///
/// The other order is what this used to do, and it is why a lens name or an
/// aperture written by Adobe came out as `Ć'11` and `â€` rather than `ƒ/11`
/// and `•`: every byte of a multi-byte character became a character of its
/// own.
fn decode_ascii(bytes: &[u8]) -> String {
    let end = bytes.iter().position(|b| *b == 0).unwrap_or(bytes.len());
    let field = &bytes[..end];

    if let Ok(text) = std::str::from_utf8(field) {
        return text.trim_end().to_string();
    }

    // Latin-1: every byte is its own character, which never fails.
    let mut out = String::with_capacity(field.len());
    for byte in field {
        let _ = out.write_char(*byte as char);
    }

    out.trim_end().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_rational_and_ascii() {
        let bytes = [0x0A, 0, 0, 0, 0x02, 0, 0, 0];
        let cursor = Cursor::new(&bytes, Endian::Little);
        let value = Value::decode(&cursor, FieldType::Rational, 1, 0).unwrap();
        assert_eq!(value.as_f64(), Some(5.0));
        assert_eq!(value.to_display_string(), "5");

        let bytes = b"Canon\0\0\0";
        let cursor = Cursor::new(bytes, Endian::Little);
        let value = Value::decode(&cursor, FieldType::Ascii, 8, 0).unwrap();
        assert_eq!(value.as_str(), Some("Canon"));
    }

    /// What Adobe writes into an "ASCII" field. Reading it a byte at a time
    /// turned every multi-byte character into mojibake.
    #[test]
    fn a_utf8_field_is_read_as_utf8() {
        assert_eq!(decode_ascii("ƒ/11".as_bytes()), "ƒ/11");
        assert_eq!(
            decode_ascii("Zeiss Planar 50mm •".as_bytes()),
            "Zeiss Planar 50mm •"
        );
    }

    /// And what a camera writes is still readable, because Latin-1 with an
    /// accent in it is not valid UTF-8.
    #[test]
    fn a_latin1_field_still_reads() {
        // "Nikkor 35mm f/1.8" with a Latin-1 degree sign in it.
        let bytes = [
            b'N', b'i', b'k', b'k', b'o', b'r', b' ', 0xB0, b' ', b'3', b'5', 0,
        ];

        assert_eq!(decode_ascii(&bytes), "Nikkor ° 35");
    }

    #[test]
    fn plain_ascii_is_unchanged_either_way() {
        assert_eq!(decode_ascii(b"Canon EOS R5  "), "Canon EOS R5");
        assert_eq!(decode_ascii(b"trailing   "), "trailing");
    }

    #[test]
    fn zero_denominator_does_not_divide_by_zero() {
        let bytes = [0x0A, 0, 0, 0, 0, 0, 0, 0];
        let cursor = Cursor::new(&bytes, Endian::Little);
        let value = Value::decode(&cursor, FieldType::Rational, 1, 0).unwrap();
        assert_eq!(value.as_f64(), None);
        assert_eq!(value.to_display_string(), "10/0");
    }

    #[test]
    fn decode_rejects_truncated_data() {
        let cursor = Cursor::new(&[0, 1, 2], Endian::Big);
        assert!(Value::decode(&cursor, FieldType::Long, 4, 0).is_none());
    }

    #[test]
    fn formats_floats_without_trailing_zeroes() {
        assert_eq!(format_f64(5.0), "5");
        assert_eq!(format_f64(2.5), "2.5");
        assert_eq!(format_f64(-0.125), "-0.125");
    }
}
