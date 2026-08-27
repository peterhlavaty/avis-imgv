use std::env;
use std::sync::Arc;

use avis_imgv::app::App;
use eframe::egui_wgpu::{WgpuConfiguration, WgpuSetup, WgpuSetupCreateNew};
use eframe::{wgpu, NativeOptions};
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

const DEVICE_LABEL: &str = "avis-imgv-device";

const HELP: &str = "\
Usage: avis-imgv [OPTIONS] [PATH]

  PATH          An image to open, or a directory to open the first image of.
                Defaults to the working directory.

Options:
  --slideshow   Start in slideshow mode. Useful as a photo frame.
  --fullscreen  Start fullscreen.
  --benchmark   Walk the folder as fast as it will go, report how many images
                a second that was, and quit.
  --help        Show this message.";

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.iter().any(|arg| arg == "--help") {
        println!("{HELP}");
        return;
    }

    init_tracing();
    tracing::info!("Starting avis-imgv with args: {}", args.join(" "));

    match avis_imgv::decoder::raw::version() {
        Some(version) => tracing::info!("Raw development available, LibRaw {version}"),
        None => tracing::info!("Built without LibRaw; raw files show their embedded preview"),
    }

    let slideshow = args.iter().any(|arg| arg == "--slideshow");
    let fullscreen = args.iter().any(|arg| arg == "--fullscreen");
    let benchmark = args.iter().any(|arg| arg == "--benchmark");

    if let Err(e) = eframe::run_native(
        "Avis Image Viewer",
        native_options(),
        Box::new(move |cc| Ok(Box::new(App::new(cc, slideshow, fullscreen, benchmark)))),
    ) {
        tracing::error!("{e}");
    }
}

fn init_tracing() {
    let level = if cfg!(debug_assertions) {
        Level::DEBUG
    } else {
        Level::INFO
    };

    let subscriber = FmtSubscriber::builder().with_max_level(level).finish();

    if tracing::subscriber::set_global_default(subscriber).is_err() {
        eprintln!("Failure installing the log subscriber; continuing without logs");
    }
}

/// Asks wgpu for everything the adapter offers.
///
/// Low powered hardware caps texture sizes well below the 8192 egui assumes —
/// a Raspberry Pi 5 stops at 4096 — and the decoder needs the real number to
/// know how far to downscale.
fn native_options() -> NativeOptions {
    let device_descriptor = Arc::new(|adapter: &wgpu::Adapter| {
        let limits = adapter.limits();
        tracing::info!("Max 2D texture size: {}", limits.max_texture_dimension_2d);

        wgpu::DeviceDescriptor {
            label: Some(DEVICE_LABEL),
            required_limits: limits,
            ..wgpu::DeviceDescriptor::default()
        }
    });

    NativeOptions {
        wgpu_options: WgpuConfiguration {
            wgpu_setup: WgpuSetup::CreateNew(WgpuSetupCreateNew {
                device_descriptor,
                ..Default::default()
            }),
            ..Default::default()
        },
        ..Default::default()
    }
}
