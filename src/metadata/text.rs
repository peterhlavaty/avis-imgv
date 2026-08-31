//! Pairing raw files with their JPEG siblings.
//!
//! The name formatting that used to live here is now one grammar shared by the
//! status bar, the bulk rename and the captions; see
//! [`crate::metadata::template`].

use std::path::{Path, PathBuf};

use itertools::Itertools;

use crate::formats::{extension_of, RAW_EXTENSIONS};

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
