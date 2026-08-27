//! Text helpers built on top of metadata: user facing name formatting and
//! pairing raw files with their JPEG siblings.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use itertools::Itertools;
use regex::Regex;

use crate::formats::{extension_of, RAW_EXTENSIONS};

/// `$( literal #Tag# literal )` — a fragment that is kept only when the tag
/// resolves.
fn tag_expression() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    // The pattern is a compile time constant, so it cannot fail to compile.
    RE.get_or_init(|| Regex::new(r"(\$\(([^()]*#([\w \s]*)#[^()]*)\))").expect("valid regex"))
}

/// Expands `$(...#Tag#...)` fragments against `metadata`.
///
/// Fragments whose tag is missing disappear entirely, which lets one format
/// string serve images with wildly different metadata.
pub fn format_string_with_metadata(input: &str, metadata: &BTreeMap<String, String>) -> String {
    let mut output = String::from(input);

    for captures in tag_expression().captures_iter(input) {
        // Group 1 is the whole `$(...)`, 2 its body, 3 the tag name.
        let (Some(expression), Some(body), Some(tag)) =
            (captures.get(1), captures.get(2), captures.get(3))
        else {
            continue;
        };

        let replacement = match metadata.get(tag.as_str()) {
            Some(value) => body.as_str().replace(&format!("#{}#", tag.as_str()), value),
            None => String::new(),
        };

        output = output.replace(expression.as_str(), &replacement);
    }

    output
}

/// Collapses raw+JPEG pairs that share a file stem into a single entry,
/// preferring the JPEG because it is the one we can display.
pub fn group_raw_jpg_paths(paths: &[PathBuf]) -> Vec<PathBuf> {
    paths
        .iter()
        .map(|path| (path.clone(), file_stem(path)))
        .sorted()
        .chunk_by(|(_, stem)| stem.clone())
        .into_iter()
        .filter_map(|(_, group)| {
            let group: Vec<PathBuf> = group.map(|(path, _)| path).collect();

            group
                .iter()
                .find(|path| !RAW_EXTENSIONS.contains(&extension_of(path).as_str()))
                .or_else(|| group.first())
                .cloned()
        })
        .collect()
}

fn file_stem(path: &Path) -> String {
    path.file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn metadata(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    #[test]
    fn drops_fragments_whose_tag_is_missing() {
        let input = "$(#File Name#)$( • ƒ#Aperture#)$( • #Shutter Speed#)$( • #ISO# ISO)";
        let tags = metadata(&[
            ("File Name", "test.jpg"),
            ("Aperture", "5.0"),
            ("ISO", "500"),
        ]);

        assert_eq!(
            format_string_with_metadata(input, &tags),
            "test.jpg • ƒ5.0 • 500 ISO"
        );
    }

    #[test]
    fn a_format_without_tags_is_returned_as_is() {
        assert_eq!(
            format_string_with_metadata("plain title", &metadata(&[])),
            "plain title"
        );
    }

    #[test]
    fn prefers_the_jpeg_of_a_raw_pair() {
        let grouped = group_raw_jpg_paths(&[
            PathBuf::from("photo1.RAF"),
            PathBuf::from("photo1.JPG"),
            PathBuf::from("photo2.JPG"),
            PathBuf::from("photo3.RAF"),
        ]);

        assert_eq!(
            grouped,
            vec![
                PathBuf::from("photo1.JPG"),
                PathBuf::from("photo2.JPG"),
                PathBuf::from("photo3.RAF"),
            ]
        );
    }

    #[test]
    fn unsorted_input_still_groups() {
        let grouped = group_raw_jpg_paths(&[
            PathBuf::from("photo3.RAF"),
            PathBuf::from("photo1.JPG"),
            PathBuf::from("photo2.RAF"),
            PathBuf::from("photo1.RAF"),
        ]);

        assert_eq!(
            grouped,
            vec![
                PathBuf::from("photo1.JPG"),
                PathBuf::from("photo2.RAF"),
                PathBuf::from("photo3.RAF"),
            ]
        );
    }

    #[test]
    fn handles_edge_cases() {
        assert_eq!(group_raw_jpg_paths(&[]), Vec::<PathBuf>::new());
        assert_eq!(
            group_raw_jpg_paths(&[PathBuf::from("only.RAF"), PathBuf::from("only.RAF")]),
            vec![PathBuf::from("only.RAF")]
        );
    }
}
