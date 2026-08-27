//! Prints everything the in-process metadata reader finds in an image.
//!
//! Useful when comparing against exiftool while adding tags.
//!
//! `cargo run --example dump_metadata -- <path>...`

use std::path::PathBuf;

use avis_imgv::formats::Format;
use avis_imgv::metadata::Metadata;

fn main() {
    for arg in std::env::args().skip(1) {
        let path = PathBuf::from(&arg);
        let bytes = match std::fs::read(&path) {
            Ok(bytes) => bytes,
            Err(e) => {
                println!("{arg}: {e}");
                continue;
            }
        };

        let started = std::time::Instant::now();
        let (mut metadata, preview) = Metadata::parse(&bytes, Format::from_path(&path));
        let elapsed = started.elapsed();
        metadata.add_file_tags(&path, bytes.len());

        println!("=== {arg} ({:?} in {elapsed:?})", Format::from_path(&path));
        println!("  orientation: {:?}", metadata.orientation);
        println!("  icc bytes:   {:?}", metadata.icc.as_ref().map(Vec::len));
        println!("  preview:     {:?}", preview.map(<[u8]>::len));

        if let Some(preview) = preview {
            match image::load_from_memory(preview) {
                Ok(image) => println!("  preview size: {}x{}", image.width(), image.height()),
                Err(e) => println!("  preview failed to decode: {e}"),
            }
        }
        for (tag, value) in &metadata.tags {
            println!("  {tag}: {value}");
        }
    }
}
