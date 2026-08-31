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
    sort(&mut images);
    images
}

/// Puts a collection in the order a person reads it.
///
/// `Vec::sort` compares bytes, which gives `IMG_10` before `IMG_9` and puts
/// every capital before every lower case letter. The folder modes have sorted
/// naturally since they were written; the views people actually browse in did
/// not, so the two disagreed about what order a folder was in and the README
/// described the wrong one.
pub fn sort(images: &mut [PathBuf]) {
    images.sort_by(order);
}

/// Where `image` belongs in an already sorted collection.
///
/// So a photograph appearing in a watched folder can be put in its place
/// rather than being appended and the whole thing sorted again — which, for
/// the collection, means the difference between one insertion and reading the
/// folder afresh.
pub fn position_for(images: &[PathBuf], image: &Path) -> usize {
    images.partition_point(|existing| order(existing, &image.to_path_buf()).is_lt())
}

/// Folder first, then name, both naturally.
fn order(a: &PathBuf, b: &PathBuf) -> std::cmp::Ordering {
    let (Some(left), Some(right)) = (a.parent(), b.parent()) else {
        return a.cmp(b);
    };

    // The folder first, so a flattened tree stays grouped by directory
    // rather than interleaving two folders' frame numbers.
    crate::organize::sort::natural(&left.to_string_lossy(), &right.to_string_lossy()).then_with(
        || {
            crate::organize::sort::natural(
                &a.file_name().unwrap_or_default().to_string_lossy(),
                &b.file_name().unwrap_or_default().to_string_lossy(),
            )
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A photograph appearing in a watched folder goes where sorting it would
    /// have put it, without the folder being sorted again.
    #[test]
    fn a_new_photograph_lands_where_it_belongs() {
        let mut images: Vec<PathBuf> = ["/p/IMG_1.jpg", "/p/IMG_2.jpg", "/p/IMG_10.jpg"]
            .iter()
            .map(PathBuf::from)
            .collect();

        for (arriving, expected) in [
            ("/p/IMG_0.jpg", 0),
            ("/p/IMG_3.jpg", 2),
            ("/p/IMG_11.jpg", 3),
        ] {
            assert_eq!(
                position_for(&images, &PathBuf::from(arriving)),
                expected,
                "{arriving}"
            );
        }

        // And inserting there is the same as appending and sorting.
        let arriving = PathBuf::from("/p/IMG_3.jpg");
        let at = position_for(&images, &arriving);
        images.insert(at, arriving);

        let mut sorted = images.clone();
        sort(&mut sorted);
        assert_eq!(images, sorted);
    }

    /// The folder comes first, so a frame arriving in one subfolder does not
    /// land among another's.
    #[test]
    fn a_new_photograph_lands_in_its_own_folder() {
        let images: Vec<PathBuf> = ["/p/a/1.jpg", "/p/a/2.jpg", "/p/b/1.jpg"]
            .iter()
            .map(PathBuf::from)
            .collect();

        assert_eq!(position_for(&images, &PathBuf::from("/p/a/3.jpg")), 2);
        assert_eq!(position_for(&images, &PathBuf::from("/p/b/0.jpg")), 2);
        assert_eq!(position_for(&images, &PathBuf::from("/p/c/1.jpg")), 3);
    }

    #[test]
    fn the_first_photograph_of_an_empty_collection_goes_first() {
        assert_eq!(position_for(&[], &PathBuf::from("/p/a.jpg")), 0);
    }

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

    /// `Vec::sort` compares bytes, which is the wrong order for a folder off a
    /// camera: `IMG_10` came before `IMG_9`, and the folder modes disagreed
    /// with the views people browse in.
    #[test]
    fn a_folder_is_ordered_the_way_a_person_reads_it() {
        let mut paths: Vec<PathBuf> = ["IMG_10.jpg", "IMG_100.jpg", "IMG_9.jpg", "IMG_2.jpg"]
            .iter()
            .map(|name| PathBuf::from("/photos").join(name))
            .collect();

        sort(&mut paths);

        let names: Vec<String> = paths
            .iter()
            .map(|path| path.file_name().unwrap().to_string_lossy().into_owned())
            .collect();

        assert_eq!(
            names,
            ["IMG_2.jpg", "IMG_9.jpg", "IMG_10.jpg", "IMG_100.jpg"]
        );
    }

    /// A flattened tree stays grouped by folder rather than interleaving two
    /// folders' frame numbers.
    #[test]
    fn the_folder_comes_before_the_name() {
        let mut paths: Vec<PathBuf> = [
            "/photos/b/IMG_1.jpg",
            "/photos/a/IMG_2.jpg",
            "/photos/a/IMG_1.jpg",
        ]
        .iter()
        .map(PathBuf::from)
        .collect();

        sort(&mut paths);

        assert_eq!(paths[0], PathBuf::from("/photos/a/IMG_1.jpg"));
        assert_eq!(paths[1], PathBuf::from("/photos/a/IMG_2.jpg"));
        assert_eq!(paths[2], PathBuf::from("/photos/b/IMG_1.jpg"));
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
