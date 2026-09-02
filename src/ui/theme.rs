//! The viewer's own dark theme.

use eframe::egui;

use egui::{style, Color32, Theme, ThemePreference, Visuals};

// Only the bundled font needs these, and a build without it should not be
// warned about imports it was never going to use.
#[cfg(feature = "custom_font")]
use epaint::{
    text::{FontData, FontDefinitions},
    FontFamily,
};

/// Installs the theme and, with the `custom_font` feature, the bundled font.
///
/// It used to be dark whatever anybody wanted, above a reason that is sound and
/// too wide: a light surround does shift how the photograph in front of it
/// reads, but what surrounds the photograph is the *backdrop*, which is its own
/// field and which this does not touch. A theme setting that lied would be
/// worse than none, so this one is live: the function is called from one place
/// and nothing else in the program holds a `Visuals` of its own.
pub fn apply_theme(ctx: &egui::Context, light: bool) {
    #[cfg(feature = "custom_font")]
    apply_fonts(ctx);

    if light {
        apply_light(ctx);
        return;
    }

    ctx.set_theme(ThemePreference::Dark);
    let previous_theme = Visuals::dark();

    let accent = Color32::from_rgb(220, 220, 220);
    let bg = Color32::from_rgb(48, 48, 48);
    let wbg = Color32::from_rgb(200, 200, 200);
    let extreme_bg = Color32::from_rgb(70, 70, 70);
    let light_bg = Color32::from_rgb(150, 150, 150);
    let font = Color32::from_rgb(185, 185, 185);

    ctx.set_visuals_of(
        Theme::Dark,
        egui::Visuals {
            override_text_color: Some(font),
            window_fill: bg,
            panel_fill: bg,
            button_frame: true,
            extreme_bg_color: extreme_bg,
            widgets: style::Widgets {
                noninteractive: create_widget_visuals(
                    previous_theme.widgets.noninteractive,
                    wbg,
                    accent,
                ),
                inactive: create_widget_visuals(
                    previous_theme.widgets.inactive,
                    extreme_bg,
                    accent,
                ),
                hovered: create_widget_visuals(previous_theme.widgets.hovered, light_bg, bg),
                active: create_widget_visuals(previous_theme.widgets.active, wbg, light_bg),
                open: create_widget_visuals(previous_theme.widgets.open, wbg, bg),
            },
            ..previous_theme
        },
    );
}

fn create_widget_visuals(
    previous: style::WidgetVisuals,
    bg_fill: egui::Color32,
    stroke: egui::Color32,
) -> style::WidgetVisuals {
    style::WidgetVisuals {
        bg_fill,
        bg_stroke: egui::Stroke {
            color: stroke,
            ..previous.bg_stroke
        },
        ..previous
    }
}

#[cfg(feature = "custom_font")]
pub fn apply_fonts(ctx: &egui::Context) {
    tracing::info!("Applying custom fonts");

    let mut fonts = FontDefinitions::default();

    fonts.font_data.insert(
        "custom_font".to_owned(),
        std::sync::Arc::new(FontData::from_static(include_bytes!(
            "../../resources/Atkinson_Hyperlegible_Next/AtkinsonHyperlegibleNext-Regular.ttf"
        ))),
    );

    fonts.font_data.insert(
        "custom_font_italic".to_owned(),
        std::sync::Arc::new(FontData::from_static(include_bytes!(
            "../../resources/Atkinson_Hyperlegible_Next/AtkinsonHyperlegibleNext-Italic.ttf"
        ))),
    );

    let mut_fonts = fonts.families.get_mut(&FontFamily::Proportional).unwrap();

    mut_fonts.insert(0, "custom_font".to_owned());
    mut_fonts.insert(1, "custom_font_italic".to_owned());

    ctx.set_fonts(fonts);
}

/// The light palette.
///
/// Built the same way as the dark one, from egui's own light visuals with the
/// same five greys inverted, so the two are the same design rather than two
/// designs. The ground behind the photograph is not among them.
#[allow(clippy::needless_pass_by_value)]
fn apply_light(ctx: &egui::Context) {
    ctx.set_theme(ThemePreference::Light);
    let previous_theme = Visuals::light();

    let accent = Color32::from_rgb(40, 40, 40);
    let bg = Color32::from_rgb(238, 238, 238);
    let wbg = Color32::from_rgb(70, 70, 70);
    let extreme_bg = Color32::from_rgb(252, 252, 252);
    let light_bg = Color32::from_rgb(120, 120, 120);
    let font = Color32::from_rgb(40, 40, 40);

    ctx.set_visuals_of(
        Theme::Light,
        egui::Visuals {
            override_text_color: Some(font),
            window_fill: bg,
            panel_fill: bg,
            button_frame: true,
            extreme_bg_color: extreme_bg,
            widgets: style::Widgets {
                noninteractive: create_widget_visuals(
                    previous_theme.widgets.noninteractive,
                    wbg,
                    accent,
                ),
                inactive: create_widget_visuals(
                    previous_theme.widgets.inactive,
                    extreme_bg,
                    accent,
                ),
                hovered: create_widget_visuals(previous_theme.widgets.hovered, light_bg, bg),
                active: create_widget_visuals(previous_theme.widgets.active, wbg, light_bg),
                open: create_widget_visuals(previous_theme.widgets.open, wbg, bg),
            },
            ..previous_theme
        },
    );
}

/// A colour the configuration spells out, or `fallback` when it does not.
///
/// Falls back to the setting's own default rather than to something arbitrary,
/// because a colour nobody can read the value of is a colour nobody can fix.
pub fn colour(hex: &str, fallback: Color32) -> Color32 {
    Color32::from_hex(hex).unwrap_or(fallback)
}

/// The grey behind the photograph, from what the file holds.
pub fn backdrop(hex: &str) -> Color32 {
    colour(hex, Color32::from_rgb(119, 119, 119))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_backdrop_reads_from_its_hex() {
        assert_eq!(backdrop("#000000"), Color32::BLACK);
        assert_eq!(backdrop("#777777"), Color32::from_rgb(119, 119, 119));
    }

    /// A colour the file cannot answer for falls back to the default rather
    /// than to black, which would look like a bug.
    #[test]
    fn nonsense_falls_back_to_the_default_grey() {
        assert_eq!(backdrop("not a colour"), Color32::from_rgb(119, 119, 119));
        assert_eq!(backdrop(""), Color32::from_rgb(119, 119, 119));
    }
}
#[cfg(test)]
mod glyphs {
    use epaint::text::{FontDefinitions, FontFamily, FontId, Fonts};
    use epaint::AlphaFromCoverage;

    /// Every glyph the interface draws, in one place, so that the font chain
    /// can be asked whether it has them.
    ///
    /// `◐` and `❏` were once drawn in the contact sheet's corners and neither
    /// was in a font the proportional family loads — `◐` is in Hack, which is
    /// the *monospace* family — so both drew an empty box. Nothing catches
    /// that but looking, and looking is what a test is for.
    fn every_glyph() -> Vec<&'static str> {
        use crate::organize::group::Kind;
        use crate::view::stacks::glyph;

        let mut glyphs = vec![
            "★",
            "■",
            "✉",
            crate::view::image_view::bottom_bar::KEEPING_ZOOM,
            crate::view::image_view::bottom_bar::KEEPING_PAN,
        ];

        glyphs.extend(
            [Kind::Hdr, Kind::FocusStack, Kind::Timelapse, Kind::Series]
                .into_iter()
                .map(glyph),
        );

        glyphs
    }

    #[test]
    fn every_glyph_the_interface_draws_is_in_a_font_it_loads() {
        let mut fonts = Fonts::new(
            4096,
            AlphaFromCoverage::default(),
            FontDefinitions::default(),
        );
        let font = FontId::new(14.0, FontFamily::Proportional);

        for glyph in every_glyph() {
            assert!(
                fonts.has_glyphs(&font, glyph),
                "{glyph} is in none of the fonts the proportional family loads, \
                 so it would draw as an empty box"
            );
        }
    }
}
