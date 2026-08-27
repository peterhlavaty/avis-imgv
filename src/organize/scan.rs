//! Reading what a whole folder says about itself.
//!
//! Sorting and filtering need the metadata of every file, not just the ones on
//! screen, so opening one of the folder modes starts a sweep: the front of
//! each file for its EXIF, and its sidecar for the rating and keywords.
//!
//! It costs a couple of milliseconds a file across every core, and runs on a
//! thread of its own so the interface stays live while it happens. Results
//! arrive in batches, and the list is usable — sortable, filterable, even
//! renameable — from the first frame, filling in as they land.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc::{channel, Receiver};
use std::sync::Arc;

use rayon::prelude::*;

use crate::annotations::sidecar;
use crate::formats::Format;
use crate::metadata::dates::{self, DateField};
use crate::metadata::xmp::Xmp;
use crate::metadata::Metadata;

use super::similarity::{self, Fingerprint};
use super::Entry;

use image::RgbaImage;

/// How many files are read before the batch is sent.
///
/// Small enough that a folder starts filling in immediately, large enough that
/// a thousand files do not mean a thousand channel sends and repaints.
const BATCH: usize = 32;

/// What the sweep found out about one file.
pub struct Read {
    pub path: PathBuf,
    pub size: u64,
    pub metadata: Metadata,
    pub annotations: Xmp,
    pub dates: Vec<DateField>,
    pub fingerprint: Option<Fingerprint>,
    pub thumbnail: Option<Arc<RgbaImage>>,
}

/// A folder being read, in the background.
pub struct Scan {
    results: Receiver<Vec<Read>>,
    done: Arc<AtomicUsize>,
    cancelled: Arc<AtomicBool>,
    total: usize,
}

impl Scan {
    /// Starts reading `paths`. Dropping the scan stops it.
    pub fn start(paths: Vec<PathBuf>) -> Scan {
        let (sender, results) = channel();
        let total = paths.len();

        let done = Arc::new(AtomicUsize::new(0));
        let cancelled = Arc::new(AtomicBool::new(false));

        let progress = Arc::clone(&done);
        let stop = Arc::clone(&cancelled);

        // Spawned rather than run on the pool directly: `par_chunks` blocks
        // until the whole folder is read, and the interface has to keep
        // drawing while that happens.
        let spawned = std::thread::Builder::new()
            .name("avis-scan".to_string())
            .spawn(move || {
                paths.par_chunks(BATCH).for_each(|chunk| {
                    if stop.load(Ordering::Relaxed) {
                        return;
                    }

                    let batch: Vec<Read> = chunk.iter().filter_map(read).collect();

                    progress.fetch_add(chunk.len(), Ordering::Relaxed);

                    // A closed channel means the mode was left; nothing to do.
                    let _ = sender.send(batch);
                });
            });

        if let Err(e) = &spawned {
            tracing::error!("Could not start the folder scan: {e}");
        }

        Scan {
            results,
            done,
            cancelled,
            total,
        }
    }

    /// Moves whatever has been read into `entries`.
    ///
    /// Returns whether anything arrived, so the caller can redraw only when
    /// there is something new to draw.
    pub fn collect_into(&mut self, entries: &mut [Entry]) -> bool {
        let mut arrived = false;

        while let Ok(batch) = self.results.try_recv() {
            for read in batch {
                let Some(entry) = entries.iter_mut().find(|entry| entry.path == read.path) else {
                    continue;
                };

                entry.size = read.size;
                entry.metadata = Some(read.metadata);
                entry.annotations = read.annotations;
                entry.dates = read.dates;
                entry.fingerprint = read.fingerprint;
                entry.thumbnail = read.thumbnail;
            }

            arrived = true;
        }

        arrived
    }

    /// How many files have been read, and how many there are.
    pub fn progress(&self) -> (usize, usize) {
        (
            self.done.load(Ordering::Relaxed).min(self.total),
            self.total,
        )
    }

    pub fn is_finished(&self) -> bool {
        let (done, total) = self.progress();
        done >= total
    }
}

impl Drop for Scan {
    fn drop(&mut self) {
        self.cancelled.store(true, Ordering::Relaxed);
    }
}

/// Everything one file has to say, without decoding a pixel of it.
///
/// Only the front of the file is read, which is where every container we know
/// puts its metadata. A file that hides its EXIF further in shows no dates
/// here, and the time shift will simply have nothing to offer for it.
fn read(path: &PathBuf) -> Option<Read> {
    let size = std::fs::metadata(path).map(|file| file.len()).unwrap_or(0);
    let head = crate::decoder::preview::head(path)?;
    let format = Format::from_path(path);

    let parsed = crate::metadata::Metadata::parse(&head, format);
    let thumbnail = parsed.thumbnail.and_then(decode_thumbnail).map(Arc::new);
    let fingerprint = thumbnail.as_deref().and_then(similarity::fingerprint);

    let mut metadata = parsed.metadata;
    metadata.add_file_tags(path, size as usize);

    // The sidecar wins over what the file carries, the same way it does when
    // an image is opened.
    let annotations = sidecar::read(path).unwrap_or_else(|| std::mem::take(&mut metadata.xmp));

    Some(Read {
        path: path.clone(),
        size,
        metadata,
        annotations,
        dates: dates::fields(&head, format),
        fingerprint,
        thumbnail,
    })
}

/// The thumbnail the camera wrote, decoded.
///
/// A hundred and sixty by a hundred and twenty pixels decode in well under a
/// millisecond, which is what makes it affordable to do for a whole folder —
/// and it earns its keep twice, once as the summary the grouping compares
/// frames by and once as the picture the group panel shows.
fn decode_thumbnail(bytes: &[u8]) -> Option<RgbaImage> {
    crate::decoder::codec::decode(bytes, Some(Format::Jpeg)).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decoder::test_support::encode;
    use image::ImageFormat;
    use std::time::{Duration, Instant};

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("avis-scan-{name}"));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        dir
    }

    /// Runs a scan to completion, or gives up after ten seconds.
    fn finish(mut scan: Scan, entries: &mut [Entry]) {
        let started = Instant::now();

        while !scan.is_finished() && started.elapsed() < Duration::from_secs(10) {
            scan.collect_into(entries);
            std::thread::yield_now();
        }

        // Whatever the last batch was is still in the channel.
        std::thread::sleep(Duration::from_millis(50));
        scan.collect_into(entries);
    }

    #[test]
    fn a_folder_is_read_into_its_entries() {
        let dir = temp_dir("read");
        let mut paths = Vec::new();

        for index in 0..5 {
            let path = dir.join(format!("photo{index}.jpg"));
            std::fs::write(&path, encode(32, 24, [1, 2, 3, 255], ImageFormat::Jpeg)).unwrap();
            paths.push(path);
        }

        let mut entries = super::super::entries(&paths);
        finish(Scan::start(paths), &mut entries);

        assert!(entries.iter().all(Entry::is_scanned), "every file was read");
        assert!(entries.iter().all(|entry| entry.size > 0));
        assert_eq!(
            entries[0].tag("File Name"),
            Some("photo0.jpg"),
            "the file tags are filled in"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_file_that_cannot_be_read_leaves_its_entry_alone() {
        let dir = temp_dir("missing");
        let paths = vec![dir.join("not-there.jpg")];

        let mut entries = super::super::entries(&paths);
        finish(Scan::start(paths), &mut entries);

        assert!(!entries[0].is_scanned());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn an_empty_folder_finishes_at_once() {
        let scan = Scan::start(Vec::new());

        assert!(scan.is_finished());
        assert_eq!(scan.progress(), (0, 0));
    }

    #[test]
    fn progress_never_reports_more_than_there_is() {
        let dir = temp_dir("progress");
        let path = dir.join("photo.jpg");
        std::fs::write(&path, encode(16, 16, [0, 0, 0, 255], ImageFormat::Jpeg)).unwrap();

        let paths = vec![path];
        let mut entries = super::super::entries(&paths);
        let scan = Scan::start(paths);
        finish(scan, &mut entries);

        let scan = Scan::start(vec![dir.join("photo.jpg")]);
        let (done, total) = scan.progress();
        assert!(done <= total);

        let _ = std::fs::remove_dir_all(&dir);
    }
}
