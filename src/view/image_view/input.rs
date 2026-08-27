//! Turning key presses into view commands.
//!
//! Collecting commands rather than mutating the view directly keeps the
//! keyboard map in one readable place and lets the status bar issue the same
//! commands as the keyboard.

use eframe::egui;
use eframe::epaint::Vec2;

use crate::config::{ImageViewConfig, Shortcut};
use crate::utils;

/// How much one press of the zoom keys changes the magnification.
///
/// A quarter each way: small enough to arrive at a particular framing, large
/// enough that crossing a useful range does not take twenty presses.
const ZOOM_STEP: f32 = 1.25;

/// How fast a held pan key moves the image, as a share of the panel per
/// second.
const PAN_SPEED: f32 = 1.5;

/// Something the image view can be asked to do.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Command {
    Next,
    Previous,
    /// Fit the whole image in the panel.
    Fit,
    /// Fill the panel, cropping the overflowing side.
    Fill,
    /// Keep filling the panel as the user navigates.
    ToggleFillLatch,
    FitHorizontal,
    FitVertical,
    /// Double the magnification, wrapping back to fitted.
    ZoomStep,
    /// Multiply the magnification, for the zoom keys and the wheel.
    ZoomBy(f32),
    /// Magnify to a percentage of the image's own pixels.
    ZoomToPercent(f32),
    /// Put this image where the last one was left.
    RepeatPlace,
    ToggleFrame,
    ShowMoreImages,
    ShowFewerImages,
    /// The user action at this position in the configuration.
    UserAction(usize),
}

/// Reads this frame's input and returns the commands it maps to.
///
/// Returns nothing while a text field has focus, so typing a path never
/// triggers a shortcut.
pub fn collect(ctx: &egui::Context, config: &ImageViewConfig) -> Vec<Command> {
    if utils::are_inputs_muted(ctx) {
        return Vec::new();
    }

    let bindings = [
        (&config.sc_next, Command::Next),
        (&config.sc_prev, Command::Previous),
        (&config.sc_fit, Command::Fit),
        (&config.sc_fit_maximize, Command::Fill),
        (&config.sc_latch_fit_maximize, Command::ToggleFillLatch),
        (&config.sc_fit_horizontal, Command::FitHorizontal),
        (&config.sc_fit_vertical, Command::FitVertical),
        (&config.sc_zoom, Command::ZoomStep),
        (&config.sc_zoom_in, Command::ZoomBy(ZOOM_STEP)),
        (&config.sc_zoom_out, Command::ZoomBy(1.0 / ZOOM_STEP)),
        (&config.sc_one_to_one, Command::ZoomToPercent(100.0)),
        (&config.sc_repeat_place, Command::RepeatPlace),
        (&config.sc_frame, Command::ToggleFrame),
        (&config.sc_more_images_shown, Command::ShowMoreImages),
        (&config.sc_less_images_shown, Command::ShowFewerImages),
    ];

    let mut commands: Vec<Command> = ctx.input_mut(|input| {
        bindings
            .iter()
            .filter(|(shortcut, _)| input.consume_shortcut(&shortcut.kbd_shortcut))
            .map(|(_, command)| *command)
            .collect()
    });

    commands.extend(user_actions(ctx, config));
    commands
}

fn user_actions(ctx: &egui::Context, config: &ImageViewConfig) -> Vec<Command> {
    ctx.input_mut(|input| {
        config
            .user_actions
            .iter()
            .enumerate()
            .filter(|(_, action)| input.consume_shortcut(&action.shortcut.kbd_shortcut))
            .map(|(i, _)| Command::UserAction(i))
            .collect()
    })
}

/// How far the held pan keys move the image this frame, in screen pixels.
///
/// Read as held keys rather than as shortcuts because panning is a movement,
/// not an event: the image should glide while the key is down instead of
/// stepping at whatever rate the operating system chooses to repeat it.
///
/// `panel` is the size of the space the image is drawn into, so the same key
/// covers the same share of the picture whatever the window size, and `seconds`
/// is how long the last frame took, so it covers it at the same speed whatever
/// the frame rate.
pub fn panning(ctx: &egui::Context, config: &ImageViewConfig, panel: Vec2, seconds: f32) -> Vec2 {
    if utils::are_inputs_muted(ctx) {
        return Vec2::ZERO;
    }

    let held = |shortcut: &Shortcut| {
        let key = shortcut.kbd_shortcut.logical_key;
        ctx.input(|input| input.key_down(key))
    };

    // Reading the panel as "which way the eye moves", so pressing right shows
    // what is further right. The pan itself is the opposite of that, which is
    // where the negation comes from.
    let mut direction = Vec2::ZERO;
    if held(&config.sc_pan_right) {
        direction.x -= 1.0;
    }
    if held(&config.sc_pan_left) {
        direction.x += 1.0;
    }
    if held(&config.sc_pan_down) {
        direction.y -= 1.0;
    }
    if held(&config.sc_pan_up) {
        direction.y += 1.0;
    }

    if direction == Vec2::ZERO {
        return Vec2::ZERO;
    }

    // A diagonal should not be faster than a straight line.
    let step = PAN_SPEED * seconds.clamp(0.0, 0.1);

    direction.normalized() * Vec2::new(panel.x * step, panel.y * step)
}

/// Navigation driven by the scroll wheel, which only applies while the pointer
/// is over the image and is not being used to zoom.
pub fn scroll_navigation(ctx: &egui::Context, hovered: bool) -> Option<Command> {
    if !hovered || ctx.input(|i| i.zoom_delta()) != 1.0 {
        return None;
    }

    match ctx.input(|i| i.raw_scroll_delta.y) {
        delta if delta > 0.0 => Some(Command::Next),
        delta if delta < 0.0 => Some(Command::Previous),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use eframe::egui::{Event, Key, Modifiers, RawInput};

    fn context_with(events: Vec<Event>) -> egui::Context {
        let ctx = egui::Context::default();
        ctx.begin_pass(RawInput {
            events,
            ..Default::default()
        });
        ctx
    }

    fn key_press(key: Key) -> Event {
        Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::NONE,
        }
    }

    #[test]
    fn maps_arrow_keys_to_navigation() {
        let config = ImageViewConfig::default();
        let ctx = context_with(vec![key_press(Key::ArrowRight)]);

        assert_eq!(collect(&ctx, &config), vec![Command::Next]);
    }

    #[test]
    fn unbound_keys_produce_nothing() {
        let config = ImageViewConfig::default();
        let ctx = context_with(vec![key_press(Key::Z)]);

        assert!(collect(&ctx, &config).is_empty());
    }

    #[test]
    fn user_actions_are_reported_by_position() {
        let mut config = ImageViewConfig::default();
        config.user_actions = vec![crate::config::UserAction {
            shortcut: crate::config::Shortcut::new("z", &[]),
            exec: "true".to_string(),
            callback: None,
        }];

        let ctx = context_with(vec![key_press(Key::Z)]);
        assert_eq!(collect(&ctx, &config), vec![Command::UserAction(0)]);
    }

    #[test]
    fn scroll_navigation_needs_the_pointer_over_the_image() {
        let ctx = context_with(vec![]);
        assert_eq!(scroll_navigation(&ctx, false), None);
        assert_eq!(scroll_navigation(&ctx, true), None);
    }

    #[test]
    fn r_repeats_the_last_view() {
        let config = ImageViewConfig::default();
        let ctx = context_with(vec![key_press(Key::R)]);

        assert_eq!(collect(&ctx, &config), vec![Command::RepeatPlace]);
    }

    #[test]
    fn plus_and_minus_zoom() {
        let config = ImageViewConfig::default();

        assert_eq!(
            collect(&context_with(vec![key_press(Key::Plus)]), &config),
            vec![Command::ZoomBy(ZOOM_STEP)]
        );
        assert_eq!(
            collect(&context_with(vec![key_press(Key::Minus)]), &config),
            vec![Command::ZoomBy(1.0 / ZOOM_STEP)]
        );
    }

    /// Held, not pressed: the event only says the key went down, and `panning`
    /// asks whether it is still down.
    fn holding(key: Key) -> egui::Context {
        context_with(vec![key_press(key)])
    }

    const PANEL: Vec2 = Vec2::new(1000.0, 800.0);
    const FRAME: f32 = 1.0 / 60.0;

    #[test]
    fn nothing_held_pans_nowhere() {
        let config = ImageViewConfig::default();
        assert_eq!(
            panning(&context_with(vec![]), &config, PANEL, FRAME),
            Vec2::ZERO
        );
    }

    #[test]
    fn looking_right_moves_the_image_left() {
        let config = ImageViewConfig::default();
        let pan = panning(&holding(Key::D), &config, PANEL, FRAME);

        assert!(pan.x < 0.0, "{pan:?}");
        assert_eq!(pan.y, 0.0);
    }

    #[test]
    fn the_four_directions_are_opposites() {
        let config = ImageViewConfig::default();

        let right = panning(&holding(Key::D), &config, PANEL, FRAME);
        let left = panning(&holding(Key::A), &config, PANEL, FRAME);
        let down = panning(&holding(Key::S), &config, PANEL, FRAME);
        let up = panning(&holding(Key::W), &config, PANEL, FRAME);

        assert_eq!(right, -left);
        assert_eq!(down, -up);
        assert!(up.y > 0.0);
    }

    #[test]
    fn a_diagonal_is_no_faster_than_a_straight_line() {
        let config = ImageViewConfig::default();
        let ctx = context_with(vec![key_press(Key::D), key_press(Key::S)]);

        let diagonal = panning(&ctx, &config, PANEL, FRAME);
        let straight = panning(&holding(Key::D), &config, PANEL, FRAME);

        // Same share of the panel covered, just split between the two axes.
        assert!((diagonal.x.abs() - straight.x.abs() / 2f32.sqrt()).abs() < 0.01);
    }

    #[test]
    fn a_long_frame_does_not_send_the_image_flying() {
        let config = ImageViewConfig::default();

        let stutter = panning(&holding(Key::D), &config, PANEL, 5.0);
        let smooth = panning(&holding(Key::D), &config, PANEL, FRAME);

        assert!(stutter.x.abs() < PANEL.x);
        assert!(stutter.x.abs() > smooth.x.abs());
    }

    #[test]
    fn panning_is_silent_while_a_text_field_has_focus() {
        let config = ImageViewConfig::default();
        let ctx = holding(Key::D);
        ctx.memory_mut(|memory| memory.request_focus(egui::Id::new("a text field")));

        assert_eq!(panning(&ctx, &config, PANEL, FRAME), Vec2::ZERO);
    }
}
