//! Finding the images to open, from the command line or from a directory.

use std::path::{Path, PathBuf};
use std::{env, fs};

use crate::formats;
use crate::STARTER_STATE_ARGS;

/// Reads the command line and returns the collection to open, plus the image
/// to land on when one was named directly.
pub fn paths_from_args() -> (Vec<PathBuf>, Option<PathBuf>) {
    let args: Vec<String> = env::args()
        .skip(1)
        .filter(|arg| !STARTER_STATE_ARGS.contains(&arg.as_str()))
        .collect();

    tracing::info!("Opening {args:?}");

    match args.len() {
        0 => (crawl_current_dir(), None),
        1 => from_single_arg(&args[0]),
        // Several paths: treat them as the collection itself.
        _ => (
            args.iter()
                .map(PathBuf::from)
                .filter(|path| formats::is_supported(path))
                .collect(),
            None,
        ),
    }
}

fn crawl_current_dir() -> Vec<PathBuf> {
    match env::current_dir() {
        Ok(dir) => crawl(&dir, false),
        Err(e) => {
            tracing::error!("Failure reading the working directory -> {e}");
            Vec::new()
        }
    }
}

/// A single argument is either a directory to open or an image to land on.
fn from_single_arg(arg: &str) -> (Vec<PathBuf>, Option<PathBuf>) {
    let path = absolute(PathBuf::from(arg));

    if path.is_dir() {
        return (crawl(&path, false), None);
    }

    match path.parent() {
        Some(parent) => (crawl(parent, false), Some(path.clone())),
        None => (vec![path], None),
    }
}

/// Resolves a relative path against the working directory.
fn absolute(path: PathBuf) -> PathBuf {
    if path.is_absolute() {
        return path;
    }

    match env::current_dir() {
        Ok(dir) => dir.join(path),
        Err(_) => path,
    }
}

/// Collects every image in `path`, descending into sub-directories when
/// `flatten` is set.
pub fn crawl(path: &Path, flatten: bool) -> Vec<PathBuf> {
    let mut images = Vec::new();
    let mut directories = vec![path.to_path_buf()];

    while let Some(directory) = directories.pop() {
        let entries = match fs::read_dir(&directory) {
            Ok(entries) => entries,
            Err(e) => {
                tracing::error!("Failure reading {} -> {e}", directory.display());
                continue;
            }
        };

        for entry in entries.flatten() {
            let path = entry.path();

            if path.is_dir() {
                if flatten {
                    directories.push(path);
                }
            } else if formats::is_supported(&path) {
                images.push(path);
            }
        }
    }

    tracing::info!("Found {} images in {}", images.len(), path.display());
    images
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Builds a small directory tree in a unique temporary directory.
    fn fixture(name: &str) -> PathBuf {
        let root = env::temp_dir().join(format!("avis-crawler-{name}"));
        let _ = fs::remove_dir_all(&root);

        fs::create_dir_all(root.join("nested")).unwrap();
        for file in ["a.jpg", "b.PNG", "notes.txt"] {
            fs::write(root.join(file), b"x").unwrap();
        }
        fs::write(root.join("nested/c.jpg"), b"x").unwrap();

        root
    }

    #[test]
    fn finds_images_and_skips_everything_else() {
        let root = fixture("flat");
        let mut found = crawl(&root, false);
        found.sort();

        assert_eq!(found, vec![root.join("a.jpg"), root.join("b.PNG")]);
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn descends_when_flattened() {
        let root = fixture("deep");
        let mut found = crawl(&root, true);
        found.sort();

        assert_eq!(
            found,
            vec![
                root.join("a.jpg"),
                root.join("b.PNG"),
                root.join("nested/c.jpg")
            ]
        );
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn a_missing_directory_yields_nothing() {
        assert!(crawl(Path::new("/definitely/not/here"), false).is_empty());
    }

    #[test]
    fn naming_an_image_opens_its_folder_on_it() {
        let root = fixture("single");
        let image = root.join("a.jpg");

        let (paths, selected) = from_single_arg(&image.to_string_lossy());

        assert_eq!(paths.len(), 2);
        assert_eq!(selected, Some(image));
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn naming_a_directory_opens_all_of_it() {
        let root = fixture("dir");
        let (paths, selected) = from_single_arg(&root.to_string_lossy());

        assert_eq!(paths.len(), 2);
        assert_eq!(selected, None);
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn relative_paths_resolve_against_the_working_directory() {
        let resolved = absolute(PathBuf::from("photo.jpg"));
        assert!(resolved.is_absolute());
    }
}
