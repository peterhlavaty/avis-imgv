//! `raw.*`: how a camera raw file becomes a picture.

use super::*;

const PAIRING: &[Choice] = &[
    Choice {
        value: "off",
        label: "Show both",
        sentence: "The raw and the JPEG are two photographs, rated separately.",
    },
    Choice {
        value: "jpeg",
        label: "Show the JPEG",
        sentence: "One photograph, browsed as the JPEG, which decodes in a fraction of \
                   the time. Marks and moves reach both files.",
    },
    Choice {
        value: "raw",
        label: "Show the raw",
        sentence: "One photograph, browsed as the raw, which is the file that will be \
                   developed. Marks and moves reach both files.",
    },
];

const SOURCE: &[Choice] = &[
    Choice {
        value: "preview",
        label: "The camera's preview",
        sentence: "The JPEG the camera embedded: what it showed you on its own screen, \
                   and almost free to decode. A DNG written by Camera Raw carries a 256 \
                   pixel preview and nothing else, which is why a DNG can look tiny.",
    },
    Choice {
        value: "develop",
        label: "Develop the sensor data",
        sentence: "The real file, demosaiced. Slower by an order of magnitude, and the \
                   only way to judge sharpness or blown highlights.",
    },
];

const QUALITY: &[Choice] = &[
    Choice {
        value: "fast",
        label: "Fast",
        sentence: "Bilinear. Quick enough to browse with, soft at the pixel level.",
    },
    Choice {
        value: "balanced",
        label: "Balanced",
        sentence: "What most people should leave it at.",
    },
    Choice {
        value: "best",
        label: "Best",
        sentence: "Slowest, and worth it only when judging fine detail.",
    },
];

/// LibRaw takes an `i32` and this used to be handed to it with no validation
/// at all. Four named choices, with the passes in the fourth.
const HIGHLIGHTS: &[Choice] = &[
    Choice {
        value: "0",
        label: "Clip",
        sentence: "Blown highlights go to white. What the camera itself does.",
    },
    Choice {
        value: "1",
        label: "Leave unclipped",
        sentence: "Keeps what is above white, which usually looks magenta.",
    },
    Choice {
        value: "2",
        label: "Blend",
        sentence: "Mixes the two, which is the honest middle.",
    },
    Choice {
        value: "3",
        label: "Rebuild",
        sentence: "Reconstructs the highlight from the channels that did not clip. \
                   Slowest, and sometimes invents colour.",
    },
];

pub fn rows() -> Vec<Row> {
    vec![
        row!(
            RawFiles / Plain,
            "raw.pair_with_jpeg",
            "Raw and JPEG shot together",
            "A camera set to raw+JPEG writes two files of one frame. Browsing both \
             means rating the shoot twice and letting the two copies disagree about \
             what was decided.",
            [
                "raw+jpeg",
                "pair",
                "duplicate",
                "two files",
                "cr2",
                "cr3",
                "nef",
                "arw",
                "dng"
            ],
            Reopen,
            None,
            Access::Enum {
                get: |c| c.raw.pair_with_jpeg.value(),
                set: |c, v| {
                    if let Some(prefer) = crate::organize::pairs::Prefer::of(v) {
                        c.raw.pair_with_jpeg = prefer;
                    }
                },
                choices: PAIRING,
            },
        ),
        row!(
            RawFiles / Developing,
            "raw.source",
            "What a raw file shows",
            "The most consequential setting a raw shooter has. The camera's preview is \
             almost free and is what most browsing wants; developing the sensor data is \
             the only way to judge what is actually in the file.",
            [
                "raw",
                "develop",
                "preview",
                "why is my raw small",
                "libraw",
                "demosaic"
            ],
            Rebuild,
            None,
            Access::Enum {
                get: |c| c.raw.source.value(),
                set: |c, v| {
                    if let Some(source) = crate::config::RawSource::of(v) {
                        c.raw.source = source;
                    }
                },
                choices: SOURCE,
            },
        ),
        row!(
            RawFiles / Developing,
            "raw.quality",
            "How much work to spend developing",
            "Which demosaicing algorithm is used when the sensor data is developed. \
             Does nothing while the camera's preview is being shown.",
            ["demosaic", "quality", "slow", "sharp"],
            Rebuild,
            None,
            Access::Enum {
                get: |c| c.raw.quality.value(),
                set: |c, v| {
                    if let Some(quality) = crate::config::RawQuality::of(v) {
                        c.raw.quality = quality;
                    }
                },
                choices: QUALITY,
            },
        ),
        row!(
            RawFiles / Developing,
            "raw.camera_white_balance",
            "Use the camera's white balance",
            "Without it colours come out noticeably wrong, so it is on unless you have \
             a reason. Does nothing while the camera's preview is being shown.",
            ["white balance", "wb", "colour", "color", "temperature"],
            Rebuild,
            None,
            boolean!(raw.camera_white_balance),
        ),
        row!(
            RawFiles / Developing,
            "raw.auto_brighten",
            "Stretch the tones to fill the range",
            "Lifts a dark raw so it reads like the camera's own preview. Off is the \
             honest rendering and a shock beside a JPEG.",
            ["brightness", "auto", "exposure", "dark"],
            Rebuild,
            None,
            boolean!(raw.auto_brighten),
        ),
        row!(
            RawFiles / Developing,
            "raw.highlight_mode",
            "What happens to blown highlights",
            "Handed straight to LibRaw with no validation until now, so a number \
             outside the four it understands was simply passed through.",
            ["highlight", "blown", "clipping", "recovery", "magenta"],
            Rebuild,
            None,
            Access::Enum {
                get: |c| match c.raw.highlight_mode {
                    0 => "0",
                    1 => "1",
                    2 => "2",
                    _ => "3",
                },
                set: |c, v| {
                    if let Ok(mode) = v.parse::<u8>() {
                        c.raw.highlight_mode = mode;
                    }
                },
                choices: HIGHLIGHTS,
            },
        ),
    ]
}
