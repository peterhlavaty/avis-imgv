//! Turns raw tag values into the strings shown in the UI.
//!
//! Mirrors exiftool's "print conversions" for the tags a photographer actually
//! reads, and falls back to a generic rendering for everything else.

use super::labels::{self, Labels};
use super::tags::IfdKind;
use super::value::{format_f64, Value};

/// Renders `value` for `tag` the way exiftool would.
pub fn display(kind: IfdKind, tag: u16, value: &Value) -> String {
    if let Some(rendered) = display_enumerated(kind, tag, value) {
        return rendered;
    }

    match (kind, tag) {
        (IfdKind::Exif, 0x829A) => exposure_time(value),
        (IfdKind::Exif, 0x9201) => shutter_speed_from_apex(value),
        (IfdKind::Exif, 0x829D | 0x9202 | 0x9205) => aperture(kind, tag, value),
        (IfdKind::Exif, 0x920A) => with_unit(value, " mm"),
        (IfdKind::Exif, 0xA405) => with_unit(value, " mm"),
        (IfdKind::Exif, 0x9204 | 0x9203) => signed_decimal(value),
        (IfdKind::Exif, 0x9000 | 0xA000) => version(value),
        (IfdKind::Exif, 0x9101) => components_configuration(value),
        (IfdKind::Gps, 0x0002 | 0x0004) => gps_coordinate(value),
        _ => value.to_display_string(),
    }
}

/// Renders tags whose value is an index into a table of labels.
fn display_enumerated(kind: IfdKind, tag: u16, value: &Value) -> Option<String> {
    let labels: Labels = match (kind, tag) {
        (IfdKind::Root, 0x0112) => labels::ORIENTATION,
        (IfdKind::Root, 0x0103) => labels::COMPRESSION,
        (IfdKind::Root, 0x0128) => labels::RESOLUTION_UNIT,
        (IfdKind::Root, 0x0213) => labels::Y_CB_CR_POSITIONING,
        (IfdKind::Exif, 0xA001) => labels::COLOR_SPACE,
        (IfdKind::Exif, 0x8822) => labels::EXPOSURE_PROGRAM,
        (IfdKind::Exif, 0x9207) => labels::METERING_MODE,
        (IfdKind::Exif, 0x9208) => labels::LIGHT_SOURCE,
        (IfdKind::Exif, 0x9209) => return Some(flash(value)),
        (IfdKind::Exif, 0xA210) => labels::RESOLUTION_UNIT,
        (IfdKind::Exif, 0xA217) => labels::SENSING_METHOD,
        (IfdKind::Exif, 0xA300) => labels::FILE_SOURCE,
        (IfdKind::Exif, 0xA401) => labels::CUSTOM_RENDERED,
        (IfdKind::Exif, 0xA402) => labels::EXPOSURE_MODE,
        (IfdKind::Exif, 0xA403) => labels::WHITE_BALANCE,
        (IfdKind::Exif, 0xA406) => labels::SCENE_CAPTURE_TYPE,
        (IfdKind::Exif, 0xA407) => labels::GAIN_CONTROL,
        (IfdKind::Exif, 0xA408..=0xA40A) => labels::NORMAL_LOW_HIGH,
        (IfdKind::Exif, 0xA40C) => labels::SUBJECT_DISTANCE_RANGE,
        _ => return None,
    };

    let raw = value.as_u32()?;
    Some(labels::lookup(labels, raw).map_or_else(|| format!("Unknown ({raw})"), str::to_string))
}

/// `1/500`, `2`, `30` — never `0.002`.
pub fn exposure_time(value: &Value) -> String {
    match value.as_f64() {
        Some(seconds) if seconds > 0.0 && seconds < 0.25001 => {
            format!("1/{}", (1.0 / seconds).round() as i64)
        }
        Some(seconds) => format_f64((seconds * 10.0).round() / 10.0),
        None => value.to_display_string(),
    }
}

/// APEX shutter speed (`Tv`) back to a shutter time.
fn shutter_speed_from_apex(value: &Value) -> String {
    match value.as_f64() {
        Some(apex) => exposure_time(&Value::Float(vec![2f64.powf(-apex)])),
        None => value.to_display_string(),
    }
}

/// APEX aperture (`Av`) back to an f-number, or a plain f-number as is.
pub fn aperture(kind: IfdKind, tag: u16, value: &Value) -> String {
    let Some(raw) = value.as_f64() else {
        return value.to_display_string();
    };

    let f_number = if matches!((kind, tag), (IfdKind::Exif, 0x9202 | 0x9205)) {
        2f64.powf(raw / 2.0)
    } else {
        raw
    };

    format_f64((f_number * 10.0).round() / 10.0)
}

fn with_unit(value: &Value, unit: &str) -> String {
    match value.as_f64() {
        Some(v) => format!("{}{unit}", format_f64((v * 10.0).round() / 10.0)),
        None => value.to_display_string(),
    }
}

fn signed_decimal(value: &Value) -> String {
    match value.as_f64() {
        Some(v) => {
            let rounded = (v * 100.0).round() / 100.0;
            if rounded > 0.0 {
                format!("+{}", format_f64(rounded))
            } else {
                format_f64(rounded)
            }
        }
        None => value.to_display_string(),
    }
}

/// `Flash` is a bit field; report the parts photographers look for.
fn flash(value: &Value) -> String {
    let Some(bits) = value.as_u32() else {
        return value.to_display_string();
    };

    if bits & 0x20 != 0 {
        return "No flash function".to_string();
    }

    let mut parts = vec![if bits & 1 != 0 {
        "Fired"
    } else {
        "Did not fire"
    }
    .to_string()];

    if bits & 1 != 0 {
        match (bits >> 1) & 0b11 {
            2 => parts.push("return not detected".to_string()),
            3 => parts.push("return detected".to_string()),
            _ => {}
        }
    }

    match (bits >> 3) & 0b11 {
        1 => parts.push("compulsory".to_string()),
        2 => parts.push("suppressed".to_string()),
        3 => parts.push("auto".to_string()),
        _ => {}
    }

    if bits & 0x40 != 0 {
        parts.push("red-eye reduction".to_string());
    }

    parts.join(", ")
}

/// EXIF stores versions as four ASCII digits: `0230` -> `2.30`.
fn version(value: &Value) -> String {
    let digits: Vec<u8> = match value {
        Value::Undefined(b) => b.clone(),
        Value::Ascii(s) => s.bytes().collect(),
        _ => return value.to_display_string(),
    };

    if digits.len() != 4 || !digits.iter().all(|b| b.is_ascii_digit()) {
        return value.to_display_string();
    }

    let major: u32 = (digits[0] - b'0') as u32 * 10 + (digits[1] - b'0') as u32;
    format!("{}.{}{}", major, digits[2] as char, digits[3] as char)
}

fn components_configuration(value: &Value) -> String {
    let Some(bytes) = value.as_bytes() else {
        return value.to_display_string();
    };

    bytes
        .iter()
        .map(|c| match c {
            0 => "-",
            1 => "Y",
            2 => "Cb",
            3 => "Cr",
            4 => "R",
            5 => "G",
            6 => "B",
            _ => "?",
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// Degrees, minutes and seconds as three rationals.
fn gps_coordinate(value: &Value) -> String {
    let Value::Rational(parts) = value else {
        return value.to_display_string();
    };

    let component = |i: usize| -> f64 {
        parts
            .get(i)
            .filter(|(_, d)| *d != 0)
            .map(|(n, d)| *n as f64 / *d as f64)
            .unwrap_or(0.0)
    };

    if parts.len() < 3 {
        return value.to_display_string();
    }

    format!(
        "{} deg {}' {}\"",
        format_f64(component(0)),
        format_f64(component(1)),
        format_f64((component(2) * 100.0).round() / 100.0)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rational(n: u32, d: u32) -> Value {
        Value::Rational(vec![(n, d)])
    }

    #[test]
    fn exposure_times_are_fractions_when_short() {
        assert_eq!(exposure_time(&rational(1, 500)), "1/500");
        assert_eq!(exposure_time(&rational(1, 4)), "1/4");
        assert_eq!(exposure_time(&rational(1, 2)), "0.5");
        assert_eq!(exposure_time(&rational(30, 1)), "30");
    }

    #[test]
    fn apex_values_convert_back() {
        // Av 5 is roughly f/5.6, Tv 8.965 is roughly 1/500.
        let av = Value::SRational(vec![(5, 1)]);
        assert_eq!(aperture(IfdKind::Exif, 0x9202, &av), "5.7");

        let tv = Value::SRational(vec![(8965784, 1000000)]);
        assert_eq!(display(IfdKind::Exif, 0x9201, &tv), "1/500");
    }

    #[test]
    fn f_number_is_used_verbatim() {
        assert_eq!(display(IfdKind::Exif, 0x829D, &rational(56, 10)), "5.6");
    }

    #[test]
    fn enumerated_tags_use_labels() {
        let six = Value::Unsigned(vec![6]);
        assert_eq!(display(IfdKind::Root, 0x0112, &six), "Rotate 90 CW");

        let unknown = Value::Unsigned(vec![99]);
        assert_eq!(display(IfdKind::Root, 0x0112, &unknown), "Unknown (99)");

        let srgb = Value::Unsigned(vec![1]);
        assert_eq!(display(IfdKind::Exif, 0xA001, &srgb), "sRGB");
    }

    #[test]
    fn focal_length_carries_its_unit() {
        assert_eq!(display(IfdKind::Exif, 0x920A, &rational(230, 10)), "23 mm");
    }

    #[test]
    fn flash_bits_are_decoded() {
        assert_eq!(
            display(IfdKind::Exif, 0x9209, &Value::Unsigned(vec![0])),
            "Did not fire"
        );
        assert_eq!(
            display(IfdKind::Exif, 0x9209, &Value::Unsigned(vec![0x09])),
            "Fired, compulsory"
        );
        assert_eq!(
            display(IfdKind::Exif, 0x9209, &Value::Unsigned(vec![0x19])),
            "Fired, auto"
        );
        assert_eq!(
            display(IfdKind::Exif, 0x9209, &Value::Unsigned(vec![0x47])),
            "Fired, return detected, red-eye reduction"
        );
        assert_eq!(
            display(IfdKind::Exif, 0x9209, &Value::Unsigned(vec![0x20])),
            "No flash function"
        );
    }

    #[test]
    fn versions_are_dotted() {
        let v = Value::Undefined(b"0230".to_vec());
        assert_eq!(display(IfdKind::Exif, 0x9000, &v), "2.30");
    }

    #[test]
    fn gps_coordinates_use_degrees_minutes_seconds() {
        let v = Value::Rational(vec![(48, 1), (8, 1), (1234, 100)]);
        assert_eq!(display(IfdKind::Gps, 0x0002, &v), "48 deg 8' 12.34\"");
    }

    #[test]
    fn exposure_compensation_keeps_its_sign() {
        let v = Value::SRational(vec![(-1, 3)]);
        assert_eq!(display(IfdKind::Exif, 0x9204, &v), "-0.33");
        let v = Value::SRational(vec![(1, 3)]);
        assert_eq!(display(IfdKind::Exif, 0x9204, &v), "+0.33");
    }
}
