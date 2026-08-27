//! ICC profiles: the ones we ship, and reading the name out of an embedded one.

pub const SRGB_V4: &[u8] = include_bytes!("../../icc/sRGB-elle-V4-g22.icc");
pub const DISPLAY_P3: &[u8] = include_bytes!("../../icc/Display P3.icc");
pub const CLAY_RGB: &[u8] = include_bytes!("../../icc/ClayRGB-elle-V2-g22.icc");

/// Profiles bundled with the viewer, keyed by a substring of their name.
///
/// The match is a `contains` so that vendor variants such as `RT_sRGB` resolve
/// to the closest profile we have.
pub const BUILT_IN: &[(&str, &[u8])] = &[
    ("adobe rgb", CLAY_RGB),
    ("display p3", DISPLAY_P3),
    ("srgb", SRGB_V4),
];

const HEADER_SIZE: usize = 128;
const TAG_ENTRY_SIZE: usize = 12;
const DESCRIPTION_TAG: &[u8; 4] = b"desc";

/// The built-in profile whose name `description` mentions.
pub fn built_in(description: &str) -> Option<&'static [u8]> {
    let description = description.to_lowercase();
    BUILT_IN
        .iter()
        .find(|(name, _)| description.contains(name))
        .map(|(_, icc)| *icc)
}

/// Reads the human readable name out of an ICC profile, matching what
/// exiftool reports as `Profile Description`.
pub fn description(profile: &[u8]) -> Option<String> {
    let payload = tag_payload(profile, DESCRIPTION_TAG)?;

    match payload.get(..4)? {
        b"desc" => text_description(payload),
        b"mluc" => multi_localized(payload),
        _ => None,
    }
}

/// Locates a tag's payload in the profile's tag table.
fn tag_payload<'a>(profile: &'a [u8], wanted: &[u8; 4]) -> Option<&'a [u8]> {
    let be_u32 = |at: usize| -> Option<usize> {
        let b = profile.get(at..at + 4)?;
        Some(u32::from_be_bytes([b[0], b[1], b[2], b[3]]) as usize)
    };

    let count = be_u32(HEADER_SIZE)?;
    if count > 1024 {
        return None;
    }

    for i in 0..count {
        let entry = HEADER_SIZE + 4 + i * TAG_ENTRY_SIZE;
        if profile.get(entry..entry + 4)? != wanted {
            continue;
        }

        let offset = be_u32(entry + 4)?;
        let size = be_u32(entry + 8)?;
        return profile.get(offset..offset.checked_add(size)?);
    }

    None
}

/// ICC v2 `textDescriptionType`: an ASCII string with its own length.
fn text_description(payload: &[u8]) -> Option<String> {
    let b = payload.get(8..12)?;
    let len = u32::from_be_bytes([b[0], b[1], b[2], b[3]]) as usize;
    let text = payload.get(12..12 + len)?;

    Some(trim_nul(text))
}

/// ICC v4 `multiLocalizedUnicodeType`: UTF-16BE records, one per locale.
fn multi_localized(payload: &[u8]) -> Option<String> {
    let be_u32 = |at: usize| -> Option<usize> {
        let b = payload.get(at..at + 4)?;
        Some(u32::from_be_bytes([b[0], b[1], b[2], b[3]]) as usize)
    };

    if be_u32(8)? == 0 {
        return None;
    }

    // The first record is good enough: profiles in the wild are either
    // English-only or list English first.
    let length = be_u32(20)?;
    let offset = be_u32(24)?;
    let text = payload.get(offset..offset.checked_add(length)?)?;

    let utf16: Vec<u16> = text
        .as_chunks::<2>()
        .0
        .iter()
        .map(|c| u16::from_be_bytes([c[0], c[1]]))
        .collect();

    Some(trim_nul(&String::from_utf16_lossy(&utf16).into_bytes()))
}

fn trim_nul(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes)
        .trim_end_matches('\0')
        .trim()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Builds a profile carrying a single tag.
    fn profile_with(tag: &[u8; 4], payload: &[u8]) -> Vec<u8> {
        let mut out = vec![0u8; HEADER_SIZE];
        out.extend_from_slice(&1u32.to_be_bytes());

        let offset = HEADER_SIZE + 4 + TAG_ENTRY_SIZE;
        out.extend_from_slice(tag);
        out.extend_from_slice(&(offset as u32).to_be_bytes());
        out.extend_from_slice(&(payload.len() as u32).to_be_bytes());
        out.extend_from_slice(payload);
        out
    }

    #[test]
    fn reads_v2_descriptions() {
        let mut payload = b"desc".to_vec();
        payload.extend_from_slice(&[0; 4]);
        payload.extend_from_slice(&8u32.to_be_bytes());
        payload.extend_from_slice(b"sRGB\0\0\0\0");

        let profile = profile_with(DESCRIPTION_TAG, &payload);
        assert_eq!(description(&profile).as_deref(), Some("sRGB"));
    }

    #[test]
    fn reads_v4_descriptions() {
        let text: Vec<u8> = "Display P3"
            .encode_utf16()
            .flat_map(u16::to_be_bytes)
            .collect();

        let mut payload = b"mluc".to_vec();
        payload.extend_from_slice(&[0; 4]);
        payload.extend_from_slice(&1u32.to_be_bytes()); // record count
        payload.extend_from_slice(&12u32.to_be_bytes()); // record size
        payload.extend_from_slice(b"enUS");
        payload.extend_from_slice(&(text.len() as u32).to_be_bytes());
        payload.extend_from_slice(&28u32.to_be_bytes()); // offset within payload
        payload.extend_from_slice(&text);

        let profile = profile_with(DESCRIPTION_TAG, &payload);
        assert_eq!(description(&profile).as_deref(), Some("Display P3"));
    }

    #[test]
    fn truncated_profiles_do_not_panic() {
        let mut payload = b"desc".to_vec();
        payload.extend_from_slice(&[0; 4]);
        payload.extend_from_slice(&8u32.to_be_bytes());
        payload.extend_from_slice(b"sRGB\0\0\0\0");
        let profile = profile_with(DESCRIPTION_TAG, &payload);

        for len in 0..profile.len() {
            let _ = description(&profile[..len]);
        }
    }

    #[test]
    fn matches_built_in_profiles_loosely() {
        assert_eq!(built_in("RT_sRGB").map(<[u8]>::len), Some(SRGB_V4.len()));
        assert_eq!(
            built_in("Adobe RGB (1998)").map(<[u8]>::len),
            Some(CLAY_RGB.len())
        );
        assert!(built_in("ProPhoto RGB").is_none());
    }

    #[test]
    fn shipped_profiles_are_readable() {
        for (name, bytes) in BUILT_IN {
            let described = description(bytes);
            assert!(
                described.as_deref().is_some_and(|d| !d.is_empty()),
                "the {name} profile should carry a description, got {described:?}"
            );
        }
    }

    #[test]
    fn the_srgb_profile_names_itself() {
        let described = description(SRGB_V4).unwrap_or_default().to_lowercase();
        assert!(
            described.contains("srgb"),
            "sRGB describes itself as {described:?}"
        );
    }
}
