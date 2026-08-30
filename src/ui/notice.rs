//! Telling the user something went wrong, without a dialogue to dismiss.
//!
//! A rating that could not be saved has to reach the person who pressed the
//! key. It used to reach a log line nobody sees, which is the same as not
//! reaching anybody: a whole culling session on a read-only card disappeared
//! while the stars stayed lit on screen.
//!
//! The notice is a band across the top that fades of its own accord. Nothing
//! is modal, because none of it is worth stopping a person's work for.

use std::time::{Duration, Instant};

use eframe::egui::{self, Align2, Color32, RichText};

/// How long a notice stays up before it starts to fade.
const HOLD: Duration = Duration::from_secs(6);

/// How long the fade itself takes.
const FADE: Duration = Duration::from_millis(600);

/// How many lines are kept; older ones are dropped as new ones arrive.
const MAX_LINES: usize = 4;

#[derive(Debug)]
struct Line {
    text: String,
    /// How many identical messages this line stands for.
    repeats: usize,
    shown: Instant,
}

/// The notices currently on screen.
#[derive(Debug, Default)]
pub struct Notices {
    lines: Vec<Line>,
}

impl Notices {
    /// Puts `text` on screen. Repeating a message counts it rather than
    /// stacking it, because a failing disk fails once per photograph.
    pub fn say(&mut self, text: impl Into<String>) {
        let text = text.into();

        if let Some(line) = self.lines.iter_mut().find(|line| line.text == text) {
            line.repeats += 1;
            line.shown = Instant::now();
            return;
        }

        self.lines.push(Line {
            text,
            repeats: 1,
            shown: Instant::now(),
        });

        while self.lines.len() > MAX_LINES {
            self.lines.remove(0);
        }
    }

    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    /// Draws whatever is still worth showing, and forgets the rest.
    ///
    /// Returns whether anything is still on screen, so the caller knows to ask
    /// for another frame while it fades.
    pub fn ui(&mut self, ctx: &egui::Context) -> bool {
        let now = Instant::now();
        self.lines
            .retain(|line| now.duration_since(line.shown) < HOLD + FADE);

        if self.lines.is_empty() {
            return false;
        }

        egui::Area::new(egui::Id::new("notices"))
            .anchor(Align2::CENTER_TOP, [0.0, 12.0])
            .interactable(false)
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                for line in &self.lines {
                    let text = if line.repeats > 1 {
                        format!("{} ({}×)", line.text, line.repeats)
                    } else {
                        line.text.clone()
                    };

                    egui::Frame::popup(ui.style())
                        .fill(Color32::from_rgb(72, 32, 32).gamma_multiply(opacity(line, now)))
                        .show(ui, |ui| {
                            ui.label(
                                RichText::new(text)
                                    .color(Color32::WHITE.gamma_multiply(opacity(line, now))),
                            );
                        });
                }
            });

        true
    }
}

/// How solid a line is drawn, from one down to nothing across the fade.
fn opacity(line: &Line, now: Instant) -> f32 {
    let age = now.duration_since(line.shown);

    if age <= HOLD {
        return 1.0;
    }

    let faded = (age - HOLD).as_secs_f32() / FADE.as_secs_f32();
    (1.0 - faded).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_repeated_message_is_counted_rather_than_stacked() {
        let mut notices = Notices::default();
        notices.say("Could not save");
        notices.say("Could not save");

        assert_eq!(notices.lines.len(), 1);
        assert_eq!(notices.lines[0].repeats, 2);
    }

    #[test]
    fn only_the_last_few_messages_are_kept() {
        let mut notices = Notices::default();
        for i in 0..MAX_LINES + 3 {
            notices.say(format!("problem {i}"));
        }

        assert_eq!(notices.lines.len(), MAX_LINES);
        assert_eq!(notices.lines[0].text, format!("problem {}", 3));
    }

    #[test]
    fn a_fresh_notice_is_fully_solid_and_an_old_one_is_gone() {
        let line = Line {
            text: "x".to_string(),
            repeats: 1,
            shown: Instant::now(),
        };

        assert_eq!(opacity(&line, line.shown), 1.0);
        assert_eq!(opacity(&line, line.shown + HOLD), 1.0);
        assert_eq!(opacity(&line, line.shown + HOLD + FADE), 0.0);
    }
}
