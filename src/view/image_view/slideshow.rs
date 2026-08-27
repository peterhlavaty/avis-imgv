//! Unattended playback: hold each image for a while, drifting slowly into it.

use std::time::{Duration, Instant};

use crate::config::{Motion, SlideshowConfig};

/// Repaint cadence while zooming. Twenty steps a second is smooth enough to
/// read as motion and cheap enough to leave a photo frame idle.
const ZOOM_STEP: Duration = Duration::from_millis(50);

/// What the view should do this frame.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Step {
    /// Move to the next image.
    pub advance: bool,
    /// Multiplier on top of the zoom that makes the image fill the panel.
    pub zoom_scale: f32,
    /// How far through this picture's turn we are, from nought to one. What
    /// the travelling motion moves along.
    pub progress: f32,
    /// How long the view may sleep before it needs to draw again.
    pub repaint_after: Duration,
}

/// Drives the slideshow clock.
pub struct Slideshow {
    seconds_per_image: f64,
    zoom_fraction: f64,
    motion: Motion,
    shown_at: Instant,
}

impl Slideshow {
    pub fn new(config: &SlideshowConfig) -> Slideshow {
        Slideshow {
            seconds_per_image: config.seconds_per_image.max(1) as f64,
            zoom_fraction: config.percent_zoom as f64 / 100.0,
            motion: config.motion,
            shown_at: Instant::now(),
        }
    }

    /// What this slideshow does with a picture while it is up.
    pub fn motion(&self) -> Motion {
        self.motion
    }

    /// Restarts the clock, called whenever a new image comes up.
    pub fn restart(&mut self) {
        self.shown_at = Instant::now();
    }

    /// Advances the clock and reports what the view should do.
    pub fn tick(&mut self) -> Step {
        let elapsed = self.shown_at.elapsed().as_secs_f64();
        let progress = (elapsed / self.seconds_per_image).clamp(0.0, 1.0);
        let advance = elapsed >= self.seconds_per_image;

        if advance {
            self.restart();
        }

        Step {
            advance,
            // Grows from 1 to 1 + zoom_fraction across the image's turn.
            zoom_scale: (1.0 + progress * self.zoom_fraction) as f32,
            progress: progress as f32,
            repaint_after: if self.moves() {
                ZOOM_STEP
            } else {
                Duration::from_secs_f64((self.seconds_per_image - elapsed).max(0.0))
            },
        }
    }

    /// Whether anything is animated, which is what decides how often the view
    /// has to be woken up.
    pub fn moves(&self) -> bool {
        match self.motion {
            Motion::Still => false,
            Motion::Zoom => self.zoom_fraction != 0.0,
            Motion::Reveal => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config(seconds: u64, percent: f32) -> SlideshowConfig {
        moving(seconds, percent, Motion::Zoom)
    }

    fn moving(seconds: u64, percent: f32, motion: Motion) -> SlideshowConfig {
        SlideshowConfig {
            seconds_per_image: seconds,
            percent_zoom: percent,
            motion,
            start_with_frame_enabled: false,
            image_frame_background_color_override: None,
        }
    }

    #[test]
    fn holds_an_image_until_its_time_is_up() {
        let mut slideshow = Slideshow::new(&config(60, 25.0));
        let step = slideshow.tick();

        assert!(!step.advance);
        assert!(step.zoom_scale >= 1.0 && step.zoom_scale < 1.01);
    }

    #[test]
    fn advances_once_the_interval_passes() {
        let mut slideshow = Slideshow::new(&config(1, 0.0));
        slideshow.shown_at = Instant::now() - Duration::from_secs(2);

        assert!(slideshow.tick().advance);
        // The clock restarts, so the next tick holds again.
        assert!(!slideshow.tick().advance);
    }

    #[test]
    fn zoom_reaches_the_configured_amount_by_the_end() {
        let mut slideshow = Slideshow::new(&config(10, 25.0));
        slideshow.shown_at = Instant::now() - Duration::from_secs(10);

        assert!((slideshow.tick().zoom_scale - 1.25).abs() < 0.01);
    }

    #[test]
    fn zoom_never_overshoots() {
        let mut slideshow = Slideshow::new(&config(1, 50.0));
        slideshow.shown_at = Instant::now() - Duration::from_secs(30);

        assert!(slideshow.tick().zoom_scale <= 1.5);
    }

    #[test]
    fn a_still_slideshow_sleeps_until_the_next_image() {
        let mut slideshow = Slideshow::new(&moving(15, 0.0, Motion::Still));
        let step = slideshow.tick();

        assert!(!slideshow.moves());
        assert!(step.repaint_after > Duration::from_secs(14));
    }

    #[test]
    fn a_travelling_slideshow_is_woken_up_often() {
        let mut slideshow = Slideshow::new(&moving(15, 0.0, Motion::Reveal));

        assert!(slideshow.moves(), "there is no zoom, but it still moves");
        assert!(slideshow.tick().repaint_after <= ZOOM_STEP);
    }

    #[test]
    fn progress_runs_from_nought_to_one_across_a_picture() {
        let mut slideshow = Slideshow::new(&moving(10, 0.0, Motion::Reveal));
        assert!(slideshow.tick().progress < 0.01);

        slideshow.shown_at = Instant::now() - Duration::from_secs(5);
        let halfway = slideshow.tick().progress;
        assert!((halfway - 0.5).abs() < 0.05, "{halfway}");
    }

    #[test]
    fn progress_never_runs_past_the_end() {
        let mut slideshow = Slideshow::new(&moving(1, 0.0, Motion::Reveal));
        slideshow.shown_at = Instant::now() - Duration::from_secs(30);

        assert!(slideshow.tick().progress <= 1.0);
    }

    #[test]
    fn the_motion_travels_with_the_slideshow() {
        let slideshow = Slideshow::new(&moving(5, 0.0, Motion::Reveal));
        assert_eq!(slideshow.motion(), Motion::Reveal);
    }

    #[test]
    fn a_zero_second_interval_does_not_divide_by_zero() {
        let mut slideshow = Slideshow::new(&config(0, 25.0));
        let step = slideshow.tick();

        assert!(step.zoom_scale.is_finite());
    }
}
