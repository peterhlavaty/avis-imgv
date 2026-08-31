use std::env;
use std::sync::Arc;

use avis_imgv::app::App;
use eframe::egui_wgpu::{WgpuConfiguration, WgpuSetup, WgpuSetupCreateNew};
use eframe::{wgpu, NativeOptions};

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

    avis_imgv::logging::start();
    tracing::info!("Starting avis-imgv with args: {}", args.join(" "));

    if let Some(log) = avis_imgv::logging::path() {
        tracing::info!("Logging to {}", log.display());
    }

    match avis_imgv::decoder::raw::version() {
        Some(version) => tracing::info!("Raw development available, LibRaw {version}"),
        None => tracing::info!("Built without LibRaw; raw files show their embedded preview"),
    }

    let slideshow = args.iter().any(|arg| arg == "--slideshow");
    let fullscreen = args.iter().any(|arg| arg == "--fullscreen");
    let benchmark = args.iter().any(|arg| arg == "--benchmark");

    // Read here rather than in the app: the window's size and place have to be
    // decided before the window is made.
    let session = avis_imgv::session::Session::load();

    if let Err(e) = eframe::run_native(
        "Avis Image Viewer",
        native_options(session.window),
        Box::new(move |cc| Ok(Box::new(App::new(cc, slideshow, fullscreen, benchmark)))),
    ) {
        tracing::error!("{e}");
    }
}

/// Asks wgpu for everything the adapter offers.
///
/// Low powered hardware caps texture sizes well below the 8192 egui assumes —
/// a Raspberry Pi 5 stops at 4096 — and the decoder needs the real number to
/// know how far to downscale.
fn native_options(window: Option<avis_imgv::session::Geometry>) -> NativeOptions {
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
        viewport: viewport(window),
        ..Default::default()
    }
}

/// The window as it was left, where that is worth restoring.
///
/// A window of no size is what a minimised one reports on some platforms, and
/// opening as a sliver nobody can find is worse than opening at the default.
/// The position is only restored where the platform gave one — Wayland does
/// not — and eframe puts an unplaced window where the compositor wants it.
fn viewport(window: Option<avis_imgv::session::Geometry>) -> eframe::egui::ViewportBuilder {
    let builder = eframe::egui::ViewportBuilder::default();

    let Some(window) = window.filter(avis_imgv::session::Geometry::is_usable) else {
        return builder;
    };

    let builder = builder
        .with_inner_size([window.width, window.height])
        .with_maximized(window.maximised);

    match (window.x, window.y) {
        (Some(x), Some(y)) => builder.with_position([x, y]),
        _ => builder,
    }
}
