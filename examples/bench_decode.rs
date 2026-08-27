//! Times each stage of the decode pipeline, so an optimisation can be aimed at
//! the stage that actually costs something.
//!
//! `cargo run --release --example bench_decode -- <path>...`
//!
//! The last column decodes the same file through the `image` crate instead, so
//! the JPEG fast path can be seen to be worth having.

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use avis_imgv::decoder::{codec, color, resize};
use avis_imgv::formats::Format;
use avis_imgv::metadata::Metadata;

/// Runs each file this many times and keeps the best, which is the number that
/// says what the machine can do rather than what the scheduler did.
const RUNS: usize = 5;

fn main() {
    let paths: Vec<PathBuf> = std::env::args().skip(1).map(PathBuf::from).collect();

    if paths.is_empty() {
        println!("usage: bench_decode <path>...");
        return;
    }

    println!(
        "{:<24} {:>8} {:>8} {:>9} {:>8} {:>8} {:>9} {:>12}",
        "file", "read", "meta", "decode", "resize", "colour", "total", "via image"
    );

    for path in &paths {
        match best_of(path) {
            Some(timing) => timing.print(path),
            None => println!("{}: could not be decoded", path.display()),
        }
    }
}

#[derive(Default, Clone, Copy)]
struct Timing {
    read: Duration,
    metadata: Duration,
    decode: Duration,
    resize: Duration,
    colour: Duration,
    /// The same decode through the `image` crate, for comparison.
    through_image: Duration,
    width: u32,
    height: u32,
}

impl Timing {
    fn total(&self) -> Duration {
        self.read + self.metadata + self.decode + self.resize + self.colour
    }

    fn print(&self, path: &Path) {
        let name = path
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_default();

        println!(
            "{:<24} {:>8} {:>8} {:>9} {:>8} {:>8} {:>9} {:>12}   {}x{}",
            truncate(&name, 24),
            millis(self.read),
            millis(self.metadata),
            millis(self.decode),
            millis(self.resize),
            millis(self.colour),
            millis(self.total()),
            millis(self.through_image),
            self.width,
            self.height,
        );
    }
}

/// Milliseconds with enough decimals to see a tenth of one.
fn millis(duration: Duration) -> String {
    format!("{:.2}ms", duration.as_secs_f64() * 1000.0)
}

fn truncate(text: &str, width: usize) -> String {
    if text.chars().count() <= width {
        return text.to_string();
    }

    text.chars().take(width - 1).chain(['…']).collect()
}

fn best_of(path: &Path) -> Option<Timing> {
    (0..RUNS)
        .filter_map(|_| run(path))
        .min_by_key(Timing::total)
}

/// One pass of exactly what `decoder::decode` does, stage by stage.
fn run(path: &Path) -> Option<Timing> {
    let mut timing = Timing::default();

    let started = Instant::now();
    let bytes = std::fs::read(path).ok()?;
    timing.read = started.elapsed();

    let format = Format::from_path(path);

    let started = Instant::now();
    let (metadata, preview) = Metadata::parse(&bytes, format);
    timing.metadata = started.elapsed();

    let (source, source_format) = match preview {
        Some(preview) => (preview, Some(Format::Jpeg)),
        None => (bytes.as_slice(), format),
    };

    let started = Instant::now();
    let mut image = codec::decode(source, source_format).ok()?;
    timing.decode = started.elapsed();

    let started = Instant::now();
    image = resize::to_max_edge(image, None);
    timing.resize = started.elapsed();

    let started = Instant::now();
    color::convert(&mut image, &metadata, "srgb");
    timing.colour = started.elapsed();

    // What the same work costs without the fast path: the `image` crate hands
    // back RGB, which then has to be widened in a second pass.
    let started = Instant::now();
    let through_image = image::load_from_memory(source).ok()?.into_rgba8();
    timing.through_image = started.elapsed();

    timing.width = through_image.width();
    timing.height = through_image.height();

    Some(timing)
}
