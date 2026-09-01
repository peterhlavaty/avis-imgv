//! Drawing a rail and its handle.
//!
//! The toolkit's own drawing, copied rather than adapted: the same rail height
//! from the same spacing, the same corner radius, the same trailing fill and
//! the same handle in whichever of the two shapes the style asks for. Only the
//! interaction is different, and a page of settings that drew two kinds of
//! slider would say otherwise.

use eframe::egui::{
    self, epaint, style::HandleShape, NumExt as _, Rangef, Rect, Response, Ui, Vec2,
};

/// How large the handle is drawn, which is also how far in from each end of the
/// rect its centre can go.
pub fn handle_radius(rect: Rect) -> f32 {
    rect.height() / 2.5
}

/// The stretch of the rect the handle's centre can be in.
///
/// A rect taller than it is wide would shrink to nothing and leave the ends the
/// wrong way round, which is a panic the moment anything is clamped to it. The
/// rails here are all far wider than they are tall; the guard is for the one
/// that is given its height by a layout rather than by a number.
pub fn rail(rect: Rect, shape: HandleShape) -> Rangef {
    let radius = handle_radius(rect);
    let radius = match shape {
        HandleShape::Circle => radius,
        HandleShape::Rect { aspect_ratio } => radius * aspect_ratio,
    };

    let shrunk = rect.x_range().shrink(radius);

    if shrunk.min > shrunk.max {
        Rangef::point(rect.center().x)
    } else {
        shrunk
    }
}

/// The rail, the fill behind the handle, and the handle at `at`.
pub fn draw(ui: &Ui, response: &Response, at: f32, shape: HandleShape) {
    let rect = response.rect;
    if !ui.is_rect_visible(rect) {
        return;
    }

    let visuals = ui.style().interact(response);
    let widgets = &ui.visuals().widgets;
    let radius = (ui.style().spacing.slider_rail_height / 2.0).at_least(0.0);

    let rail_rect = Rect::from_min_max(
        egui::pos2(rect.left(), rect.center().y - radius),
        egui::pos2(rect.right(), rect.center().y + radius),
    );
    let corner_radius = widgets.inactive.corner_radius;

    ui.painter()
        .rect_filled(rail_rect, corner_radius, widgets.inactive.bg_fill);

    let centre = egui::pos2(at, rail_rect.center().y);

    if ui.visuals().slider_trailing_fill {
        let mut trailing = rail_rect;
        trailing.max.x = centre.x + f32::from(corner_radius.nw);
        ui.painter()
            .rect_filled(trailing, corner_radius, ui.visuals().selection.bg_fill);
    }

    let handle = handle_radius(rect);

    match shape {
        HandleShape::Circle => {
            ui.painter().add(epaint::CircleShape {
                center: centre,
                radius: handle + visuals.expansion,
                fill: visuals.bg_fill,
                stroke: visuals.fg_stroke,
            });
        }
        HandleShape::Rect { aspect_ratio } => {
            let half = Vec2::new(handle * aspect_ratio, handle) + Vec2::splat(visuals.expansion);
            ui.painter().rect(
                Rect::from_center_size(centre, 2.0 * half),
                visuals.corner_radius,
                visuals.bg_fill,
                visuals.fg_stroke,
                epaint::StrokeKind::Inside,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The handle's centre stops half a handle short of each end, or half of it
    /// would be drawn outside the control.
    #[test]
    fn the_rail_is_the_rect_less_a_handle_at_each_end() {
        let rect = Rect::from_min_size(egui::pos2(10.0, 0.0), Vec2::new(100.0, 20.0));
        let rail = rail(rect, HandleShape::Circle);

        assert_eq!(handle_radius(rect), 8.0);
        assert_eq!(rail.min, 18.0);
        assert_eq!(rail.max, 102.0);
    }

    /// A rect the handle does not fit across leaves one point rather than a
    /// range with its ends the wrong way round.
    #[test]
    fn a_rect_too_narrow_for_a_handle_is_a_point() {
        let rect = Rect::from_min_size(egui::pos2(0.0, 0.0), Vec2::new(10.0, 100.0));
        let rail = rail(rect, HandleShape::Circle);

        assert!(rail.min <= rail.max, "{rail:?} would panic when clamped to");
        assert_eq!(rail.clamp(1000.0), rect.center().x);
    }
}
