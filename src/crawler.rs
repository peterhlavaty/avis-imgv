//! Finding the images to open, from the command line or from a directory.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use std::{env, fs};

use crate::formats;
use crate::STARTER_STATE_ARGS;

/// What the command line asked for.
#[derive(Debug, Default)]
pub struct Opening {
    /// The collection to open.
    pub images: Vec<PathBuf>,
    /// The one to land on, when an image was named directly.
    pub selected: Option<PathBuf>,
    /// The directory that was read.
    ///
    /// Carried rather than derived from the first photograph, because a
    /// directory with no photographs in it has no first photograph — and the
    /// viewer then had nothing to say it was in, so it said the home
    /// directory, and asking to flatten an empty folder crawled everything the
    /// user owns.
    pub folder: Option<PathBuf>,
    /// Whether a path was actually named on the command line.
    ///
    /// Not the same as having found a folder: with no arguments at all the
    /// working directory is read, and the working directory of a viewer
    /// started from a desktop icon is nobody's choice. What decides whether
    /// the last session is restored is whether somebody typed a path, and this
    /// is the only place that knows.
    pub named: bool,
}

/// Reads the command line and returns the collection to open, plus the image
/// to land on when one was named directly.
pub fn paths_from_args() -> Opening {
    let args: Vec<String> = env::args()
        .skip(1)
        .filter(|arg| !STARTER_STATE_ARGS.contains(&arg.as_str()))
        .collect();

    tracing::info!("Opening {args:?}");

    match args.len() {
        0 => crawl_current_dir(),
        1 => Opening {
            named: true,
            ..from_single_arg(&args[0])
        },
        // Several paths: treat them as the collection itself, and let the
        // folder be worked out from them.
        _ => Opening {
            images: args
                .iter()
                .map(PathBuf::from)
                .filter(|path| formats::is_supported(path))
                .collect(),
            named: true,
            ..Opening::default()
        },
    }
}

fn crawl_current_dir() -> Opening {
    match env::current_dir() {
        Ok(dir) => Opening {
            images: crawl(&dir, false),
            selected: None,
            folder: Some(dir),
            named: false,
        },
        Err(e) => {
            tracing::error!("Failure reading the working directory -> {e}");
            Opening::default()
        }
    }
}

/// A single argument is either a directory to open or an image to land on.
fn from_single_arg(arg: &str) -> Opening {
    let path = absolute(PathBuf::from(arg));

    if path.is_dir() {
        return Opening {
            images: crawl(&path, false),
            selected: None,
            folder: Some(path),
            named: false,
        };
    }

    match path.parent() {
        Some(parent) => Opening {
            images: crawl(parent, false),
            selected: Some(path.clone()),
            folder: Some(parent.to_path_buf()),
            named: false,
        },
        None => Opening {
            images: vec![path],
            ..Opening::default()
        },
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

/// A crawl in progress, walked a directory at a time.
///
/// The crawl used to be three synchronous lines, and on a deep tree or a share
/// over the network every one of the nine ways into it stopped the window
/// repainting with nothing on screen to say why — the one state that draws
/// nothing because the program is not drawing at all. A `ViewportCommand` is
/// applied after `update` returns, so even setting the title first reaches the
/// screen only once the crawl has finished: there is no interim measure short
/// of this.
///
/// Chunked rather than moved to a worker. Every caller has a follow-on that
/// assumes the collection is in hand when the walk ends — landing on a path,
/// restoring a position, re-focusing after a folder job — and eight of the
/// nine hand that follow-on *in* rather than reading what is returned, so
/// carrying it across frames is a field on this and not nine rewrites.
pub struct Walk {
    root: PathBuf,
    flatten: bool,
    /// What is left to look in.
    directories: Vec<PathBuf>,
    /// Every directory actually opened, by the name the filesystem settles on.
    seen: HashSet<PathBuf>,
    images: Vec<PathBuf>,
}

impl Walk {
    pub fn new(path: &Path, flatten: bool) -> Walk {
        Walk {
            root: path.to_path_buf(),
            flatten,
            directories: vec![path.to_path_buf()],
            seen: HashSet::new(),
            images: Vec::new(),
        }
    }

    /// Looks in as many more as `budget` pays for.
    ///
    /// Returns whether there is anything left to look in. A budget rather than
    /// a count, and for the same reason the uploads have one: the cost is the
    /// directory, not the number of them. A hundred entries on a local disk
    /// and one on a share over a network are the same number and a
    /// thousandfold difference in what it costs to read them, so a count would
    /// either crawl slowly on the fast case or stutter on the slow one.
    ///
    /// Always at least one, so a single directory that takes longer than the
    /// whole budget still makes progress rather than being looked at for ever.
    pub fn step(&mut self, budget: Duration) -> bool {
        let started = Instant::now();

        loop {
            let Some(directory) = self.directories.pop() else {
                return false;
            };

            self.look_in(&directory);

            if started.elapsed() >= budget {
                return !self.directories.is_empty();
            }
        }
    }

    /// Walks the rest of it, however long that takes.
    pub fn run(mut self) -> Vec<PathBuf> {
        while self.step(Duration::from_secs(3600)) {}
        self.finish()
    }

    /// How many photographs have been found so far.
    pub fn found(&self) -> usize {
        self.images.len()
    }

    /// The collection, in order.
    pub fn finish(mut self) -> Vec<PathBuf> {
        tracing::info!(
            "Found {} images in {}",
            self.images.len(),
            self.root.display()
        );

        sort(&mut self.images);
        self.images
    }

    fn look_in(&mut self, directory: &Path) {
        // Testing whether something is a directory follows links, so a
        // symbolic link or a Windows junction pointing at one of its own
        // ancestors used to send a flattened crawl round for ever —
        // collecting the same photographs again at a longer path each time
        // until the memory ran out. Somebody's `Pictures/latest -> .` is all
        // it takes.
        if !self
            .seen
            .insert(fs::canonicalize(directory).unwrap_or_else(|_| directory.to_path_buf()))
        {
            tracing::debug!(
                "Already crawled {}, not going round again",
                directory.display()
            );
            return;
        }

        let entries = match fs::read_dir(directory) {
            Ok(entries) => entries,
            Err(e) => {
                tracing::error!("Failure reading {} -> {e}", directory.display());
                return;
            }
        };

        for entry in entries.flatten() {
            let path = entry.path();

            if is_hidden(&path) {
                continue;
            }

            if path.is_dir() {
                if self.flatten {
                    self.directories.push(path);
                }
            } else if formats::is_supported(&path) {
                self.images.push(path);
            }
        }
    }
}

/// Collects every image in `path`, descending into sub-directories when
/// `flatten` is set.
///
/// The whole walk at once, for the callers that have nothing else to do while
/// it happens: a test, and the folder scan. What the window does is
/// [`Walk::step`].
pub fn crawl(path: &Path, flatten: bool) -> Vec<PathBuf> {
    Walk::new(path, flatten).run()
}

/// Whether an entry is one the filesystem means to keep out of the way.
///
/// The leading dot, which is the convention everywhere and is what the caches
/// other programs leave behind are named: `.thumbnails`, `.DS_Store`, and the
/// `._IMG_1234.JPG` resource forks macOS writes beside photographs on
/// non-native volumes — those last are named like the photograph, are not
/// photographs, and used to be opened as a black frame between every pair of
/// real ones on a card that had been through a Mac.
///
/// A directory named on the command line is crawled whatever it is called;
/// this is about what turns up inside one.
fn is_hidden(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.starts_with('.'))
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

    /// The caches and resource forks other programs leave beside photographs
    /// are not photographs.
    #[test]
    fn hidden_entries_are_left_alone() {
        for name in [".DS_Store", "._IMG_1234.JPG", ".thumbnails"] {
            assert!(is_hidden(&PathBuf::from("/photos").join(name)), "{name}");
        }

        for name in ["IMG_1234.JPG", "a.jpg", "holiday 1.jpg"] {
            assert!(!is_hidden(&PathBuf::from("/photos").join(name)), "{name}");
        }
    }

    /// A folder pointing at one of its own ancestors used to send a flattened
    /// crawl round for ever. Only reachable where a link can be made without
    /// privileges, so the test makes one and skips itself where it cannot.
    #[cfg(unix)]
    #[test]
    fn a_folder_pointing_at_itself_is_crawled_once() {
        let root = env::temp_dir().join("avis-crawler-cycle");
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("inner")).unwrap();
        fs::write(root.join("inner/a.jpg"), b"x").unwrap();

        if std::os::unix::fs::symlink(&root, root.join("inner/loop")).is_err() {
            return;
        }

        // Would not return at all before the guard.
        let found = crawl(&root, true);

        assert_eq!(found.len(), 1, "{found:?}");
        let _ = fs::remove_dir_all(&root);
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

        let opening = from_single_arg(&image.to_string_lossy());

        assert_eq!(opening.images.len(), 2);
        assert_eq!(opening.selected, Some(image));
        assert_eq!(opening.folder, Some(root.clone()));
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn naming_a_directory_opens_all_of_it() {
        let root = fixture("dir");
        let opening = from_single_arg(&root.to_string_lossy());

        assert_eq!(opening.images.len(), 2);
        assert_eq!(opening.selected, None);
        assert_eq!(opening.folder, Some(root.clone()));
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn relative_paths_resolve_against_the_working_directory() {
        let resolved = absolute(PathBuf::from("photo.jpg"));
        assert!(resolved.is_absolute());
    }
}
