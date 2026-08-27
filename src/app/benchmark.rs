//! Measuring how fast the viewer actually goes.
//!
//! Started with `--benchmark`, the viewer advances one image every frame for
//! as long as it takes to walk the folder, then reports what it managed and
//! quits. The number that matters is images a second: it folds in decoding,
//! the wait for the next image to be ready, the upload, and drawing.

use std::time::{Duration, Instant};

/// Stop after this long even if the folder is longer, so a benchmark on ten
/// thousand images still finishes.
const TIME_LIMIT: Duration = Duration::from_secs(20);

/// Frames at the start that are not counted, while the first images are still
/// being decoded and the window is still settling.
const WARMUP_FRAMES: usize = 30;

/// A run in progress.
pub struct Benchmark {
    /// How many images to walk through before reporting.
    target: usize,
    started: Option<Instant>,
    frames: usize,
    advances: usize,
    frame_times: Vec<Duration>,
}

/// What a finished run measured.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Report {
    pub images: usize,
    pub elapsed: Duration,
    pub slowest_frame: Duration,
    pub median_frame: Duration,
}

impl Report {
    /// The headline: images a second, sustained.
    pub fn images_per_second(&self) -> f64 {
        if self.elapsed.is_zero() {
            return 0.0;
        }

        self.images as f64 / self.elapsed.as_secs_f64()
    }

    pub fn log(&self) {
        tracing::info!(
            "Benchmark: {} images in {:.2}s — {:.1} images/s, median frame {:.2}ms, slowest {:.2}ms",
            self.images,
            self.elapsed.as_secs_f64(),
            self.images_per_second(),
            self.median_frame.as_secs_f64() * 1000.0,
            self.slowest_frame.as_secs_f64() * 1000.0,
        );
    }
}

impl Benchmark {
    /// A run that walks `target` images.
    pub fn new(target: usize) -> Benchmark {
        Benchmark {
            target,
            started: None,
            frames: 0,
            advances: 0,
            frame_times: Vec::new(),
        }
    }

    /// Records a frame, and says whether the viewer should move on.
    ///
    /// `moved` is whether the last request to advance actually took, which it
    /// does not while the next image is still being decoded — that wait is
    /// exactly what is being measured.
    pub fn frame(&mut self, frame_time: Duration, moved: bool) -> bool {
        self.frames += 1;

        // The first frames go on opening the folder rather than on browsing
        // it, and counting them would flatter nothing.
        if self.frames <= WARMUP_FRAMES {
            return true;
        }

        let started = *self.started.get_or_insert_with(Instant::now);
        self.frame_times.push(frame_time);

        if moved {
            self.advances += 1;
        }

        self.advances < self.target && started.elapsed() < TIME_LIMIT
    }

    /// What the run measured, once it has stopped.
    pub fn report(&self) -> Report {
        let elapsed = self.started.map(|at| at.elapsed()).unwrap_or_default();

        let mut times = self.frame_times.clone();
        times.sort_unstable();

        Report {
            images: self.advances,
            elapsed,
            slowest_frame: times.last().copied().unwrap_or_default(),
            median_frame: times.get(times.len() / 2).copied().unwrap_or_default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frames(benchmark: &mut Benchmark, count: usize, each: Duration) {
        for _ in 0..count {
            benchmark.frame(each, true);
        }
    }

    #[test]
    fn the_opening_frames_are_not_counted() {
        let mut benchmark = Benchmark::new(1000);
        frames(&mut benchmark, WARMUP_FRAMES, Duration::from_millis(100));

        assert_eq!(benchmark.report().images, 0);
    }

    #[test]
    fn it_stops_once_it_has_walked_the_target() {
        let mut benchmark = Benchmark::new(3);
        frames(&mut benchmark, WARMUP_FRAMES, Duration::from_millis(1));

        assert!(benchmark.frame(Duration::from_millis(1), true));
        assert!(benchmark.frame(Duration::from_millis(1), true));
        assert!(!benchmark.frame(Duration::from_millis(1), true));
        assert_eq!(benchmark.report().images, 3);
    }

    #[test]
    fn frames_that_did_not_advance_still_count_against_the_rate() {
        let mut benchmark = Benchmark::new(2);
        frames(&mut benchmark, WARMUP_FRAMES, Duration::from_millis(1));

        // Waiting for the next image to decode is part of what is measured.
        benchmark.frame(Duration::from_millis(50), false);
        benchmark.frame(Duration::from_millis(1), true);

        let report = benchmark.report();
        assert_eq!(report.images, 1);
        assert_eq!(report.slowest_frame, Duration::from_millis(50));
    }

    #[test]
    fn the_median_ignores_a_single_slow_frame() {
        let mut benchmark = Benchmark::new(100);
        frames(&mut benchmark, WARMUP_FRAMES, Duration::from_millis(1));
        frames(&mut benchmark, 20, Duration::from_millis(10));
        benchmark.frame(Duration::from_millis(500), true);

        let report = benchmark.report();
        assert_eq!(report.median_frame, Duration::from_millis(10));
        assert_eq!(report.slowest_frame, Duration::from_millis(500));
    }

    #[test]
    fn a_run_that_never_started_reports_nothing_rather_than_dividing_by_zero() {
        let report = Benchmark::new(10).report();

        assert_eq!(report.images, 0);
        assert_eq!(report.images_per_second(), 0.0);
    }

    #[test]
    fn the_rate_is_images_over_the_time_they_took() {
        let report = Report {
            images: 60,
            elapsed: Duration::from_secs(2),
            slowest_frame: Duration::ZERO,
            median_frame: Duration::ZERO,
        };

        assert_eq!(report.images_per_second(), 30.0);
    }
}
