//! Choosing where a photograph goes.
//!
//! One key opens it and one key answers it, because this is the middle of a
//! cull: the slots are numbered, the digits pick them, and the folder that was
//! picked last time is what `Enter` takes. Pressing the same key twice in a row
//! skips the panel entirely and repeats the last answer, which is the motion
//! FastStone taught everyone.

use std::path::PathBuf;

use eframe::egui::{self, RichText};

/// Whether the photographs are being moved or copied.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Errand {
    Move,
    Copy,
}

impl Errand {
    pub fn verb(self) -> &'static str {
        match self {
            Errand::Move => "Move",
            Errand::Copy => "Copy",
        }
    }
}

/// One place on the panel.
#[derive(Debug, Clone)]
pub struct Slot {
    pub label: String,
    pub path: PathBuf,
}

/// What the panel is being asked about.
#[derive(Debug, Clone)]
pub struct Asking {
    pub errand: Errand,
    /// What is going, so the panel can say how much.
    pub count: usize,
    pub slots: Vec<Slot>,
    /// Where the last one went, offered again on `Enter`.
    pub last: Option<Slot>,
}

/// What the user picked.
#[derive(Debug, Clone)]
pub enum Answer {
    Send(Slot),
    /// Pick a folder that is not on the panel.
    Browse,
    Cancel,
}

/// Draws the panel and reports what was picked, if anything.
pub fn ui(ctx: &egui::Context, asking: &Asking) -> Option<Answer> {
    let mut answer = None;

    egui::Window::new(format!("{} to…", asking.errand.verb()))
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.label(
                RichText::new(match asking.count {
                    1 => "This photograph".to_string(),
                    count => format!("{count} photographs"),
                })
                .weak(),
            );
            ui.add_space(8.0);

            for (index, slot) in asking.slots.iter().enumerate().take(9) {
                let digit = index + 1;
                let response = ui
                    .button(format!("{digit}   {}", slot.label))
                    .on_hover_text(slot.path.display().to_string());

                if response.clicked() {
                    answer = Some(Answer::Send(slot.clone()));
                }
            }

            if let Some(last) = &asking.last {
                ui.add_space(6.0);
                if ui
                    .button(format!("↵   Again: {}", last.label))
                    .on_hover_text(last.path.display().to_string())
                    .clicked()
                {
                    answer = Some(Answer::Send(last.clone()));
                }
            }

            ui.add_space(6.0);
            if ui.button("Choose a folder…").clicked() {
                answer = Some(Answer::Browse);
            }

            ui.add_space(4.0);
            ui.label(RichText::new("Escape leaves them where they are").weak());
        });

    answer.or_else(|| keyboard(ctx, asking))
}

/// The keys, which are the point of the panel.
///
/// Consumed rather than read: answering hands the keyboard back, and the views
/// draw afterwards, so a key merely looked at would go on to mean whatever it
/// means the rest of the time.
fn keyboard(ctx: &egui::Context, asking: &Asking) -> Option<Answer> {
    if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape)) {
        return Some(Answer::Cancel);
    }

    if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Enter)) {
        return asking.last.clone().map(Answer::Send);
    }

    let digits = [
        egui::Key::Num1,
        egui::Key::Num2,
        egui::Key::Num3,
        egui::Key::Num4,
        egui::Key::Num5,
        egui::Key::Num6,
        egui::Key::Num7,
        egui::Key::Num8,
        egui::Key::Num9,
    ];

    for (index, key) in digits.iter().enumerate() {
        // Any modifiers, because on a Slovak or German layout the digits are
        // the shifted characters of the top row.
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, *key)) {
            return asking.slots.get(index).cloned().map(Answer::Send);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_verb_is_the_one_that_was_asked_for() {
        assert_eq!(Errand::Move.verb(), "Move");
        assert_eq!(Errand::Copy.verb(), "Copy");
    }
}
