//! Frame timings, for the overlay and for the benchmark.

use std::time::{Duration, Instant};

use eframe::egui;

/// How many recent frames the shown average covers. A second or so at a
/// healthy frame rate, which is long enough to read and short enough to react.
const RECENT_FRAMES: usize = 60;

/// What the readout says for itself, for the menu every panel carries.
///
/// No key row: the readout is on `F10` and nothing else, which is the one
/// binding in the program that is not a setting, so there is nothing for the
/// keyboard editor to arm. What its figures are about is the memory and the
/// threads, so that is the page its menu ends on.
pub const CHROME: crate::ui::panel::Chrome<'static> = crate::ui::panel::Chrome {
    subject: crate::ui::surface::Subject::the("The performance readout"),
    hide: Some(crate::app::input::Command::ToggleMetrics),
    key: None,
    page: crate::config::registry::Page::SpeedAndMemory,
    setting: "cache.ram_budget_mb",
};

pub struct PerfMetrics {
    frame_started: Instant,
    last: Duration,
    longest: Duration,
    /// The most recent frames, oldest first.
    recent: Vec<Duration>,
}

impl Default for PerfMetrics {
    fn default() -> PerfMetrics {
        Self::new()
    }
}

impl PerfMetrics {
    pub fn new() -> PerfMetrics {
        PerfMetrics {
            frame_started: Instant::now(),
            last: Duration::ZERO,
            longest: Duration::ZERO,
            recent: Vec::with_capacity(RECENT_FRAMES),
        }
    }

    pub fn new_frame(&mut self) {
        self.frame_started = Instant::now();
    }

    pub fn end_frame(&mut self) {
        self.last = self.frame_started.elapsed();
        self.longest = self.longest.max(self.last);

        if self.recent.len() == RECENT_FRAMES {
            self.recent.remove(0);
        }
        self.recent.push(self.last);
    }

    /// How long the last completed frame took.
    pub fn last_frame(&self) -> Duration {
        self.last
    }

    /// Mean of the recent frames, which is what a frame rate is read from.
    pub fn recent_mean(&self) -> Duration {
        if self.recent.is_empty() {
            return Duration::ZERO;
        }

        self.recent.iter().sum::<Duration>() / self.recent.len() as u32
    }

    /// Frames a second, from the recent mean.
    pub fn frames_per_second(&self) -> f64 {
        let mean = self.recent_mean().as_secs_f64();

        if mean > 0.0 {
            1.0 / mean
        } else {
            0.0
        }
    }

    pub fn display_metrics(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.monospace(format!("{:.0} fps", self.frames_per_second()))
                .on_hover_text(
                    "Frames a second, from the mean of the last sixty. Sixty is the \
                     usual ceiling: a monitor does not show more.",
                );
            ui.monospace("•");

            ui.monospace(format!("frame {:.2}ms", millis(self.last)))
                .on_hover_text("How long the last frame took to draw");
            ui.monospace("•");

            ui.monospace(format!("recent {:.2}ms", millis(self.recent_mean())))
                .on_hover_text(
                    "The mean of the last sixty frames, which is what the rate is read from",
                );
            ui.monospace("•");

            ui.monospace(format!("worst {:.2}ms", millis(self.longest)))
                .on_hover_text(
                    "The slowest frame since the viewer started. A single slow frame is a \
                     stutter somebody saw.",
                );
        });
    }
}

fn millis(duration: Duration) -> f64 {
    duration.as_secs_f64() * 1000.0
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Records `count` frames of a known length, without waiting for them.
    fn record(metrics: &mut PerfMetrics, count: usize, each: Duration) {
        for _ in 0..count {
            metrics.recent.push(each);
            metrics.last = each;
            metrics.longest = metrics.longest.max(each);

            if metrics.recent.len() > RECENT_FRAMES {
                metrics.recent.remove(0);
            }
        }
    }

    #[test]
    fn a_fresh_meter_reports_nothing_rather_than_dividing_by_zero() {
        let metrics = PerfMetrics::new();

        assert_eq!(metrics.recent_mean(), Duration::ZERO);
        assert_eq!(metrics.frames_per_second(), 0.0);
    }

    #[test]
    fn the_frame_rate_comes_from_the_recent_mean() {
        let mut metrics = PerfMetrics::new();
        record(&mut metrics, 10, Duration::from_millis(20));

        assert_eq!(metrics.recent_mean(), Duration::from_millis(20));
        assert!((metrics.frames_per_second() - 50.0).abs() < 0.01);
    }

    #[test]
    fn the_window_only_remembers_the_recent_frames() {
        let mut metrics = PerfMetrics::new();
        record(&mut metrics, RECENT_FRAMES, Duration::from_millis(100));
        record(&mut metrics, RECENT_FRAMES, Duration::from_millis(10));

        assert_eq!(metrics.recent_mean(), Duration::from_millis(10));
        // The worst frame is remembered for the whole session, though.
        assert_eq!(metrics.longest, Duration::from_millis(100));
    }

    #[test]
    fn timing_a_frame_records_it() {
        let mut metrics = PerfMetrics::new();
        metrics.new_frame();
        metrics.end_frame();

        assert!(metrics.last_frame() < Duration::from_millis(100));
        assert_eq!(metrics.recent.len(), 1);
    }
}
