//! `slideshow.*`: the mode where the pictures change themselves.

use super::*;

const MOTION: &[Choice] = &[
    Choice {
        value: "still",
        label: "Hold still",
        sentence: "The whole picture, fitted to the screen, not moving.",
    },
    Choice {
        value: "zoom",
        label: "Drift inwards",
        sentence: "Fills the screen and creeps closer while it is up.",
    },
    Choice {
        value: "reveal",
        label: "Travel across",
        sentence: "Fills the screen at the picture's own shape and travels along the \
                   overflowing side, so a panorama is seen whole rather than \
                   letterboxed into a strip.",
    },
];

pub fn rows() -> Vec<Row> {
    vec![
        row!(
            Slideshow / Plain,
            "slideshow.seconds_per_image",
            "Hold each picture for",
            "How long one photograph stays up. Moving by hand restarts the clock \
             rather than skipping ahead.",
            ["slideshow", "interval", "seconds", "delay", "timer"],
            Live,
            None,
            whole!(u64, 1, 3600, " s", true, slideshow.seconds_per_image),
        ),
        row!(
            Slideshow / Plain,
            "slideshow.motion",
            "While it is up",
            "Whether the picture moves while it is on screen, and how.",
            ["ken burns", "pan", "zoom", "motion", "still"],
            Live,
            None,
            Access::Enum {
                get: |c| c.slideshow.motion.value(),
                set: |c, v| {
                    if let Some(motion) = crate::config::Motion::of(v) {
                        c.slideshow.motion = motion;
                    }
                },
                choices: MOTION,
            },
        ),
        row!(
            Slideshow / Plain,
            "slideshow.percent_zoom",
            "How much closer it creeps",
            "How far in the picture drifts over its whole turn on screen. Does nothing \
             unless the motion above is \"Drift inwards\".",
            ["ken burns", "zoom", "drift", "creep"],
            Live,
            None,
            decimal!(0.0, 200.0, " %", true, slideshow.percent_zoom),
        ),
        row!(
            Slideshow / Plain,
            "slideshow.start_with_frame_enabled",
            "Start with the white frame on",
            "Whether the border round the photograph is drawn when the slideshow \
             begins, whatever it was set to before.",
            ["border", "frame", "matte", "white"],
            Live,
            None,
            boolean!(slideshow.start_with_frame_enabled),
        ),
        row!(
            Slideshow / Plain,
            "slideshow.image_frame_background_color_override",
            "The ground behind the picture",
            "A hex colour used only during a slideshow, where a different grey — or \
             black — is often wanted. Left empty, the usual backdrop is used.",
            [
                "background",
                "backdrop",
                "grey",
                "gray",
                "black",
                "colour",
                "color"
            ],
            Live,
            None,
            Access::Colour(
                |c| c.slideshow.image_frame_background_color_override.clone(),
                |c, v| c.slideshow.image_frame_background_color_override = v,
            ),
        ),
    ]
}
