//! Telling the user something happened, without a dialogue to dismiss.
//!
//! A rating that could not be saved has to reach the person who pressed the
//! key. It used to reach a log line nobody sees, which is the same as not
//! reaching anybody: a whole culling session on a read-only card disappeared
//! while the stars stayed lit on screen.
//!
//! The notice is a band across the top that fades of its own accord. Nothing
//! is modal, because none of it is worth stopping a person's work for — and
//! nothing in the band is clickable either. During a cull something is in it
//! after nearly every move, copy, delete and undo, so a band that took the
//! pointer would hold a strip across the top of the photograph for 6.6 seconds
//! at a time, including the photograph's own menu. What can be acted on lives
//! in the history instead, which does not fade.

use std::collections::VecDeque;
use std::time::{Duration, Instant};

use eframe::egui::{self, Align2, Color32, RichText};

/// How long a notice stays up before it starts to fade.
const HOLD: Duration = Duration::from_secs(6);

/// How long the fade itself takes.
const FADE: Duration = Duration::from_millis(600);

/// How many lines are on screen at once; older ones stop being drawn.
const MAX_LINES: usize = 4;

/// How many are kept in the history behind the band.
///
/// A cull says something after nearly every gesture, so a hundred is an
/// afternoon's worth and still nothing to hold in memory.
const REMEMBERED: usize = 100;

/// How much a message matters.
///
/// Everything used to be the same alarm red — `Color32::from_rgb(72, 32, 32)`
/// for "Moved 12 photographs to Selects" and for "Access is denied" alike — so
/// the colour said nothing and a real failure looked like a receipt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Severity {
    /// Something was done. Most of what a cull says.
    #[default]
    Said,
    /// Something is not as expected, and the work went on.
    Warning,
    /// Something failed.
    Failure,
}

impl Severity {
    fn fill(self) -> Color32 {
        match self {
            Severity::Said => Color32::from_rgb(38, 40, 44),
            Severity::Warning => Color32::from_rgb(74, 58, 26),
            Severity::Failure => Color32::from_rgb(72, 32, 32),
        }
    }

    /// What the history calls it.
    pub fn label(self) -> &'static str {
        match self {
            Severity::Said => "",
            Severity::Warning => "Warning",
            Severity::Failure => "Failed",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Line {
    pub text: String,
    pub severity: Severity,
    /// How many identical messages this line stands for.
    pub repeats: usize,
    pub shown: Instant,
}

/// The notices currently on screen, and what has been said lately.
#[derive(Debug, Default)]
pub struct Notices {
    lines: Vec<Line>,
    /// Everything said this session, newest last.
    history: VecDeque<Line>,
    /// How many have arrived since the history was last looked at.
    unseen: usize,
}

impl Notices {
    /// Puts `text` on screen. Repeating a message counts it rather than
    /// stacking it, because a failing disk fails once per photograph.
    pub fn say(&mut self, text: impl Into<String>) {
        self.at(Severity::Said, text);
    }

    /// Something is not as expected, and the work went on.
    pub fn warn(&mut self, text: impl Into<String>) {
        self.at(Severity::Warning, text);
    }

    /// Something failed.
    pub fn fail(&mut self, text: impl Into<String>) {
        self.at(Severity::Failure, text);
    }

    fn at(&mut self, severity: Severity, text: impl Into<String>) {
        let text = text.into();

        if let Some(line) = self.lines.iter_mut().find(|line| line.text == text) {
            line.repeats += 1;
            line.shown = Instant::now();
            line.severity = severity;

            if let Some(kept) = self.history.back_mut() {
                if kept.text == text {
                    kept.repeats += 1;
                    kept.shown = line.shown;
                    self.unseen += 1;
                    return;
                }
            }
        } else {
            self.lines.push(Line {
                text: text.clone(),
                severity,
                repeats: 1,
                shown: Instant::now(),
            });

            while self.lines.len() > MAX_LINES {
                self.lines.remove(0);
            }
        }

        self.history.push_back(Line {
            text,
            severity,
            repeats: 1,
            shown: Instant::now(),
        });
        self.unseen += 1;

        while self.history.len() > REMEMBERED {
            self.history.pop_front();
        }
    }

    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    /// Everything said this session, newest first.
    pub fn history(&self) -> impl Iterator<Item = &Line> {
        self.history.iter().rev()
    }

    /// How many have arrived since the history was last looked at.
    pub fn unseen(&self) -> usize {
        self.unseen
    }

    /// Marks the history as read, for when it has been opened.
    pub fn mark_seen(&mut self) {
        self.unseen = 0;
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
            // Deliberately. See the note at the top of this file.
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
                        .fill(line.severity.fill().gamma_multiply(opacity(line, now)))
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

/// Draws the history: everything said lately, whether or not it was seen.
///
/// The band holds four lines for 6.6 seconds and drops the rest without a word,
/// so a failure that arrived during a burst of moves was gone before anybody
/// read it and could not be recovered.
/// What a row of the history was clicked to do.
///
/// The routes live here rather than on the band, which stays untouchable: a
/// strip across the top of the photograph that takes the pointer for 6.6
/// seconds at a time is a worse defect than one that cannot be clicked.
/// Nothing a person has six seconds to click; everything a person can go back
/// to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Asked {
    /// Anything that ends in the log offers the log.
    OpenLog,
    /// A startup clash offers the row that clashed.
    Keys,
}

/// Everything the viewer has said lately, whether or not it was seen.
pub fn contents(ui: &mut egui::Ui, notices: &mut Notices) -> Option<Asked> {
    let mut asked = None;

    // Read while it is being read: the badge in the bar is about what has been
    // said since anybody last looked, and this is looking.
    notices.mark_seen();

    if notices.history.is_empty() {
        ui.weak("Nothing has been said yet.");
        return None;
    }

    ui.horizontal(|ui| {
        ui.label(format!("{} messages", notices.history.len()));
        if ui.button("Copy them all").clicked() {
            let text = notices
                .history()
                .map(|line| line.text.clone())
                .collect::<Vec<_>>()
                .join("\n");
            ui.ctx().copy_text(text);
        }
    });

    ui.separator();

    egui::ScrollArea::vertical().show(ui, |ui| {
        for line in notices.history() {
            ui.horizontal_wrapped(|ui| {
                let mark = line.severity.label();
                if !mark.is_empty() {
                    ui.label(
                        RichText::new(mark)
                            .color(line.severity.fill().to_opaque().gamma_multiply(2.4))
                            .strong(),
                    );
                }

                let text = if line.repeats > 1 {
                    format!("{} ({}×)", line.text, line.repeats)
                } else {
                    line.text.clone()
                };

                ui.label(text);

                // A failure that ends in the log offers the log; a
                // clash offers the keys. A message about something that
                // went wrong and no way to reach what went wrong is
                // the shape of dead end this whole stage is about.
                if line.severity != Severity::Said && ui.small_button("Open the log").clicked() {
                    asked = Some(Asked::OpenLog);
                }

                if line.text.contains("are both on") && ui.small_button("Fix it").clicked() {
                    asked = Some(Asked::Keys);
                }
            });
        }
    });

    asked
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
    fn only_the_last_few_messages_are_on_screen() {
        let mut notices = Notices::default();
        for i in 0..MAX_LINES + 3 {
            notices.say(format!("problem {i}"));
        }

        assert_eq!(notices.lines.len(), MAX_LINES);
        assert_eq!(notices.lines[0].text, format!("problem {}", 3));
    }

    /// The band drops what it cannot draw; the history does not, which is the
    /// whole reason it exists.
    #[test]
    fn what_the_band_drops_is_still_in_the_history() {
        let mut notices = Notices::default();
        for i in 0..MAX_LINES + 3 {
            notices.say(format!("problem {i}"));
        }

        assert_eq!(notices.history.len(), MAX_LINES + 3);
        assert_eq!(notices.history().next().unwrap().text, "problem 6");
    }

    #[test]
    fn the_history_stops_growing() {
        let mut notices = Notices::default();
        for i in 0..REMEMBERED + 40 {
            notices.say(format!("problem {i}"));
        }

        assert_eq!(notices.history.len(), REMEMBERED);
    }

    /// A failure and a receipt are not the same colour any more.
    #[test]
    fn severity_reaches_the_line() {
        let mut notices = Notices::default();
        notices.say("Moved 12 photographs to Selects");
        notices.fail("Access is denied");

        assert_eq!(notices.lines[0].severity, Severity::Said);
        assert_eq!(notices.lines[1].severity, Severity::Failure);
        assert_ne!(Severity::Said.fill(), Severity::Failure.fill());
    }

    #[test]
    fn the_count_of_what_has_not_been_read_is_kept() {
        let mut notices = Notices::default();
        notices.say("one");
        notices.warn("two");

        assert_eq!(notices.unseen(), 2);
        notices.mark_seen();
        assert_eq!(notices.unseen(), 0);
    }

    #[test]
    fn a_fresh_notice_is_fully_solid_and_an_old_one_is_gone() {
        let line = Line {
            text: "x".to_string(),
            severity: Severity::Said,
            repeats: 1,
            shown: Instant::now(),
        };

        assert_eq!(opacity(&line, line.shown), 1.0);
        assert_eq!(opacity(&line, line.shown + HOLD), 1.0);
        assert_eq!(opacity(&line, line.shown + HOLD + FADE), 0.0);
    }
}
