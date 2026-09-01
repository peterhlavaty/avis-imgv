//! Making a thing on screen answer the second button.
//!
//! One helper, used by every surface that carries a menu, so that all of them
//! behave the same way and none of them can drift.
//!
//! Two things it does that `Response::context_menu` does not. It opens on the
//! *press* rather than on the release — Microsoft's toolbar guidance, which
//! removes the ambiguity with a drag at its source instead of tuning a
//! threshold; `Popup::context_menu` keys off `secondary_clicked()`, which is
//! reported on the release. And it draws the same small chevron on hover and
//! appends the same four words to the hover text, so that a surface either says
//! it has a menu or does not have one. NN/g's finding, applied to a second
//! affordance: because only *some* things carried tooltips, people stopped
//! expecting them and missed the ones that existed.

use eframe::egui::{self, PointerButton, Response, RichText};

/// The words every surface with a menu ends its hover text with.
///
/// *More* rather than "set it": most of these menus carry verbs as well as
/// settings, and a promise of settings on a menu of verbs is a promise broken.
pub const SAYS: &str = "Right-click for more.";

/// The chevron drawn in the corner of a surface that has a menu.
const CHEVRON: &str = "⌄";

/// Marks a response as carrying a menu, and draws the menu when it is asked for.
///
/// `hint` is the sentence under the pointer, without the trailing words: those
/// are added here so every surface says the same thing.
pub fn with_menu<R>(
    ui: &egui::Ui,
    response: &Response,
    hint: &str,
    contents: impl FnOnce(&mut egui::Ui) -> R,
) -> Option<R> {
    chevron(ui, response);

    let hint = if hint.is_empty() {
        SAYS.to_string()
    } else {
        format!("{hint} {SAYS}")
    };
    response.clone().on_hover_text(hint);

    menu(ui, response, contents)
}

/// Which surface the keyboard asked for a menu on.
///
/// `Shift + F10` is the only keyboard route to a menu — egui cannot read the
/// dedicated Menu key at all, since its key list runs F1 to F35 and has no
/// entry for it. The popup's own id is derived from the response and cannot be
/// invented, so the ask is recorded by name and each surface claims it.
static KEYBOARD_ASKED: std::sync::Mutex<Option<&'static str>> = std::sync::Mutex::new(None);

/// Records that the keyboard asked for the menu of a named surface.
pub fn ask_for_menu(surface: &'static str) {
    if let Ok(mut asked) = KEYBOARD_ASKED.lock() {
        *asked = Some(surface);
    }
}

/// Whether this surface is the one that was asked for, taking the ask.
fn claimed(surface: &str) -> bool {
    let Ok(mut asked) = KEYBOARD_ASKED.lock() else {
        return false;
    };

    if *asked == Some(surface) {
        *asked = None;
        return true;
    }

    false
}

/// The menu alone, for a surface whose hover text is drawn elsewhere.
pub fn menu<R>(
    ui: &egui::Ui,
    response: &Response,
    contents: impl FnOnce(&mut egui::Ui) -> R,
) -> Option<R> {
    named_menu(ui, response, "", contents)
}

/// The same, for a surface the keyboard can reach by name.
pub fn named_menu<R>(
    ui: &egui::Ui,
    response: &Response,
    surface: &'static str,
    contents: impl FnOnce(&mut egui::Ui) -> R,
) -> Option<R> {
    // On the press, not the release. A drag that begins on this surface and a
    // menu that opens under the pointer are then two different gestures rather
    // than the same one told apart by how long it lasted.
    let pressed = (response.hovered()
        && ui
            .ctx()
            .input(|i| i.pointer.button_pressed(PointerButton::Secondary)))
        || (!surface.is_empty() && claimed(surface));

    let open = if pressed {
        Some(egui::SetOpenCommand::Bool(true))
    } else if response.clicked() {
        // A left click closes it, or the menu would stay up over whatever the
        // click just did.
        Some(egui::SetOpenCommand::Bool(false))
    } else {
        None
    };

    // The other half of opening on the press: the *release* of that same
    // gesture is a click, and a menu whose default is "close on a click
    // outside" would shut itself the instant the button came up. So while the
    // second button is down — or has just come up — no menu here closes on a
    // click. A left click elsewhere still closes it on any later frame.
    let own_gesture = ui.ctx().input(|i| {
        i.pointer.button_down(PointerButton::Secondary)
            || i.pointer.button_released(PointerButton::Secondary)
    });

    let behaviour = if own_gesture {
        egui::PopupCloseBehavior::IgnoreClicks
    } else {
        egui::PopupCloseBehavior::CloseOnClickOutside
    };

    egui::Popup::menu(response)
        .open_memory(open)
        .close_behavior(behaviour)
        .at_pointer_fixed()
        .show(contents)
        .map(|inner| inner.inner)
}

/// Draws the chevron in the corner of a surface that carries a menu.
///
/// Uniform or none: the same six points in the weak text colour on every one of
/// them.
fn chevron(ui: &egui::Ui, response: &Response) {
    if !response.hovered() {
        return;
    }

    let rect = response.rect;
    if rect.width() < 12.0 || rect.height() < 8.0 {
        return;
    }

    ui.painter().text(
        egui::pos2(rect.right() - 3.0, rect.bottom() - 3.0),
        egui::Align2::RIGHT_BOTTOM,
        CHEVRON,
        egui::FontId::proportional(6.0),
        ui.visuals().weak_text_color(),
    );
}

/// Whether the built-in menus draw their settings rows.
///
/// A process-wide flag rather than a parameter threaded through twenty menus:
/// it is one decision the whole program agrees about, and the surfaces that
/// draw menus are scattered across a dozen files that have no configuration in
/// hand.
static SETTINGS_ROWS: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(true);

/// Sets it from the configuration.
pub fn show_settings_rows(on: bool) {
    SETTINGS_ROWS.store(on, std::sync::atomic::Ordering::Relaxed);
}

/// The last row of every menu: where the settings for this object live.
///
/// Always last, never varying, never removed — unless `menus.settings_rows` is
/// off, which leaves the verbs and takes nothing away that is not reachable
/// elsewhere. It is what makes the menu a route to the page rather than a
/// second place the same decisions are made, and it is why nothing in the
/// program is reachable *only* by right-click.
pub fn more_settings(ui: &mut egui::Ui, page: crate::config::registry::Page) -> bool {
    if !SETTINGS_ROWS.load(std::sync::atomic::Ordering::Relaxed) {
        return false;
    }

    ui.separator();

    ui.button(RichText::new(format!("More settings…  ({})", page.label())))
        .on_hover_text("Opens the settings window on that page")
        .clicked()
}

/// A row that binds a key to whatever this menu is about.
///
/// Makes the control the route to its own key, which closes the loop the
/// keyboard editor otherwise owns alone — the direct answer to the request to
/// "change keybinds by ctrl+right clicking on the menu and picking them on the
/// GUI instead of having to edit configs".
pub fn bind_a_key(ui: &mut egui::Ui, what: &str) -> bool {
    if !SETTINGS_ROWS.load(std::sync::atomic::Ordering::Relaxed) {
        return false;
    }

    ui.button(format!("Bind a key to {what}…"))
        .on_hover_text("Opens the keyboard editor with that row armed")
        .clicked()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The same four words on every surface, so a person can learn them once.
    #[test]
    fn the_hover_text_always_ends_the_same_way() {
        assert!(SAYS.ends_with("more."));
        assert!(SAYS.starts_with("Right-click"));
    }
}
