//! Develops a raw file and reports what came out.
//!
//! `cargo run --features libraw --example develop_raw -- <path>...`

use std::path::PathBuf;
use std::time::Instant;

use avis_imgv::decoder::raw::{self, Demosaic, Options};

fn main() {
    match raw::version() {
        Some(version) => println!("LibRaw {version}"),
        None => {
            println!("built without LibRaw; run with --features libraw");
            return;
        }
    }

    for argument in std::env::args().skip(1) {
        let path = PathBuf::from(&argument);
        let Ok(bytes) = std::fs::read(&path) else {
            println!("{argument}: could not be read");
            continue;
        };

        for demosaic in [Demosaic::Fast, Demosaic::Balanced, Demosaic::Best] {
            let options = Options {
                develop: true,
                demosaic,
                ..Default::default()
            };

            let started = Instant::now();
            match raw::develop(&bytes, &options) {
                Ok(image) => println!(
                    "{argument}: {demosaic:?} -> {}x{} in {:?}",
                    image.width(),
                    image.height(),
                    started.elapsed()
                ),
                Err(e) => println!("{argument}: {demosaic:?} -> {e}"),
            }
        }
    }
}
