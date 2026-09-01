//! Carrying out the verbs the context menus offer.
//!
//! The views answer for what they draw — a fit, a fill, a magnification — and
//! everything that needs the folder, the journal, a raw and JPEG pair or a
//! decoder comes here.

use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, Sender};

use eframe::egui;

use crate::actions::reveal;
use crate::decoder::{self, DecodeOptions};
use crate::ui::empty::{Asked, Nothing, OFFERED};
use crate::ui::menus::Verb;
use crate::view::image_view::bottom_bar::BarAction;

use super::input::Command;
use super::panels::MenuAction;

use super::App;

/// A full size decode asked for by "Copy the picture", on its way back.
///
/// The clipboard wants the image's own pixels, not the reduction the store is
/// holding, and decoding a sixty megapixel raw takes long enough that doing it
/// on the frame the menu was clicked would stop the window. So it is done on a
/// thread of its own and picked up on whichever frame it lands.
pub struct Copying {
    pub sender: Sender<Result<egui::ColorImage, String>>,
    pub receiver: Receiver<Result<egui::ColorImage, String>>,
    /// How many are still out, so the notice can say when nothing came back.
    pub outstanding: usize,
}

impl Default for Copying {
    fn default() -> Self {
        let (sender, receiver) = channel();

        Copying {
            sender,
            receiver,
            outstanding: 0,
        }
    }
}

impl App {
    /// Does whatever a menu asked for that the view could not.
    pub(super) fn run_verb(&mut self, verb: Verb, path: PathBuf) {
        // The five turns differ only in what they compose with what is already
        // there, so they are one line rather than five.
        if let Some(extra) = verb.turn() {
            self.turn_by(extra);
            return;
        }

        match verb {
            Verb::Bin => self.delete_open_image(false),
            Verb::CopyPath => self.copy_paths(),
            Verb::CopyPicture => self.copy_picture(&path),
            Verb::ShowInFolder => {
                if !reveal::in_file_manager(&path) {
                    self.notices
                        .say("Could not open the file manager. The log says why.");
                }
            }
            // The view answers for these; they never reach here.
            Verb::Open | Verb::Fit | Verb::ActualPixels | Verb::Fill | Verb::Compare => {}
            // Answered above, before the match.
            Verb::TurnRight
            | Verb::TurnLeft
            | Verb::TurnHalf
            | Verb::MirrorHorizontally
            | Verb::MirrorVertically => {}
        }
    }

    /// Puts the paths of everything the command is about on the clipboard.
    ///
    /// One per line, which is what every file manager and every shell expects
    /// to be handed.
    fn copy_paths(&mut self) {
        let paths = self.marked_paths();
        if paths.is_empty() {
            return;
        }

        let text = paths
            .iter()
            .map(|path| path.display().to_string())
            .collect::<Vec<_>>()
            .join("\n");

        self.pending_clipboard = Some(text);
        self.notices.say(match paths.len() {
            1 => "Copied the path.".to_string(),
            n => format!("Copied {n} paths."),
        });
    }

    /// Starts a full size decode whose pixels go on the clipboard.
    fn copy_picture(&mut self, path: &Path) {
        let sender = self.copying.sender.clone();
        let profile: std::sync::Arc<str> =
            std::sync::Arc::from(self.config.output_icc_profile.as_str());
        let path = path.to_path_buf();
        let raw = super::stores::raw_options(&self.settings.raw);

        let spawned = std::thread::Builder::new()
            .name("avis-copy-image".to_string())
            .spawn(move || {
                let options = DecodeOptions::new(profile).with_raw(raw);
                let result = decoder::load(&path, &options)
                    .map_err(|e| format!("{e}"))
                    .map(|image| upright(&image));

                let _ = sender.send(result);
            });

        match spawned {
            Ok(_) => {
                self.copying.outstanding += 1;
                self.notices.say("Copying the picture…");
            }
            Err(e) => {
                tracing::error!("Could not start the copy: {e}");
                self.notices.say("Could not copy the picture.");
            }
        }
    }

    /// Runs whatever was raised during the frame with no context to hand.
    pub(super) fn handle_pending_commands(&mut self, ctx: &egui::Context) {
        for command in std::mem::take(&mut self.pending_commands) {
            self.apply(command, ctx);
        }
    }

    /// Picks up whatever the copy threads finished with.
    pub(super) fn handle_copying(&mut self, ctx: &egui::Context) {
        while let Ok(result) = self.copying.receiver.try_recv() {
            self.copying.outstanding = self.copying.outstanding.saturating_sub(1);

            match result {
                Ok(image) => {
                    ctx.copy_image(image);
                    self.notices.say("Copied the picture.");
                }
                Err(e) => {
                    tracing::error!("Could not decode for the clipboard: {e}");
                    self.notices.say(format!("Could not copy the picture: {e}"));
                }
            }
        }

        if self.copying.outstanding > 0 {
            ctx.request_repaint_after(std::time::Duration::from_millis(100));
        }

        if let Some(text) = self.pending_clipboard.take() {
            ctx.copy_text(text);
        }
    }
}

impl App {
    /// Does what one of the status bar's own words was clicked to do.
    pub(super) fn run_bar_action(&mut self, action: BarAction) {
        match action {
            BarAction::ToggleFlatten => self.apply_command(Command::ToggleFlatten),
            BarAction::ToggleWatching => self.apply_command(Command::ToggleWatcher),
            // The two that write a setting rather than only a mode. They save
            // the configuration directly for now; when the registry exists
            // they go through their rows, so that a menu row and a settings
            // page row are one declaration rendered twice.
            BarAction::SetAdvancing(on) => {
                self.advancing = on;
                self.settings.tags.advance_after_marking = on;
                self.tag_config.advance_after_marking = on;
                self.save_settings();
            }
            BarAction::Settings(path) => self.open_settings_at(path),
            BarAction::ToggleStack => self.apply_command(Command::ToggleStack),
            BarAction::ShowEverything => self.apply_command(Command::SuspendFilter),
            BarAction::ShowMessages => self.messages_visible = true,
            // One verb, offered wherever a mark is drawn. It closes every dead
            // end of its kind at once: a true statement on screen that cannot
            // be acted on.
            BarAction::ShowOnlyFlag(flag) => self.show_only(Narrow::Flag(flag)),
            BarAction::ShowOnlyLabel(label) => self.show_only(Narrow::Label(label)),
            BarAction::ShowOnlyStars(stars) => self.show_only(Narrow::Stars(stars)),
            BarAction::BindKey(path) => self.arm_key(path),
            BarAction::Mode(mode) => self.set_mode(mode),
            BarAction::SetPairing(prefer) => {
                if self.settings.raw.pair_with_jpeg == prefer {
                    return;
                }

                self.settings.raw.pair_with_jpeg = prefer;
                self.save_settings();

                // Pairing decides what the collection *is*, so the folder is
                // read again rather than waiting for a restart.
                self.reopen_folder();
                self.notices
                    .say(format!("{} — the folder was read again.", prefer.label()));
            }
        }
    }
}

/// Turns a decoded image the right way up and hands back what egui wants.
///
/// The GPU does this by sampling the texture in a different order rather than
/// by copying ninety megabytes, so the pixels held have never been turned. The
/// clipboard has no such trick, and `Orientation::applied` is the one place
/// that turn is written.
fn upright(image: &decoder::DecodedImage) -> egui::ColorImage {
    let surface = &image.surface;

    let Some(raw) =
        image::RgbaImage::from_raw(surface.width, surface.height, surface.pixels.to_vec())
    else {
        return egui::ColorImage::filled([1, 1], egui::Color32::TRANSPARENT);
    };

    let turned = image.orientation.applied(&raw);

    egui::ColorImage::from_rgba_unmultiplied(
        [turned.width() as usize, turned.height() as usize],
        turned.as_raw(),
    )
}

impl App {
    /// What to draw when there is nothing to draw.
    ///
    /// The two states are different screens: a folder with nothing in it wants
    /// a way to open another one, and a folder emptied by the rules wants the
    /// rules named and a way to set them aside.
    pub(super) fn nothing_to_show(&self) -> Nothing {
        Nothing {
            filtered: !self.paths.is_empty(),
            rules: self.narrowing.rules.sentences(),
            recent: self
                .session
                .recent_folders(OFFERED)
                .map(Path::to_path_buf)
                .collect(),
            // Named on a first run only, and thereafter left to the Help menu:
            // a line that never goes away stops being read.
            say_the_keys: self.first_session,
        }
    }

    /// Does what the screen with nothing on it was clicked to do.
    pub(super) fn run_asked(&mut self, asked: Asked) {
        match asked {
            Asked::OpenFolder => self.handle_menu(MenuAction::OpenFolder),
            Asked::OpenFiles => self.handle_menu(MenuAction::OpenFiles),
            Asked::Open(folder) => {
                let landing = self.session.position_in(&folder).map(Path::to_path_buf);
                self.open_directory(&folder, landing.as_deref());
            }
            // Already a command, and already the bar's own wording.
            Asked::ShowEverything => self.apply_command(Command::SuspendFilter),
        }
    }
}

impl App {
    /// Draws the first-run hint until one of the keys it names is pressed.
    pub(super) fn show_first_run_hint(&mut self, ctx: &egui::Context) {
        if !self.hint_visible {
            return;
        }

        // Dismissed by doing the thing rather than by a close button: the
        // hint's whole job is to get those two keys pressed once.
        if self.menu_visible || self.cheat_sheet_visible {
            self.hint_visible = false;
            return;
        }

        let menu_key = crate::ui::keys::describe(&self.config.sc_menu);
        crate::app::panels::first_run_hint(ctx, &menu_key);
    }
}

impl App {
    /// Opens the keyboard editor with the row that binds `path` armed.
    ///
    /// The other half of the reverse trip: a menu on the thing itself is the
    /// route to its key, which closes the loop the keyboard editor otherwise
    /// owns alone.
    pub(super) fn arm_key(&mut self, path: &'static str) {
        self.keys.arm(path);
        self.keys_visible = true;
    }
}

/// What a "show only these" was asked about.
///
/// A payload rather than a whole `Rules`, because `Command` derives `Copy` and
/// is taken by value, and three of the seven fields of `Rules` are `String`s.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Narrow {
    Flag(crate::metadata::xmp::Flag),
    Label(crate::metadata::xmp::Label),
    Stars(u8),
}

impl App {
    /// Narrows the folder to the photographs carrying one mark.
    ///
    /// The filter bar is raised with it, so the change is visible and
    /// reversible: a folder that silently lost nine tenths of itself is a
    /// worse answer than no answer.
    pub(super) fn show_only(&mut self, narrow: Narrow) {
        use crate::metadata::xmp::Label;
        use crate::view::narrow::{FlagRule, LabelRule};

        match narrow {
            Narrow::Flag(flag) => {
                self.narrowing.rules.flag = match flag {
                    crate::metadata::xmp::Flag::Picked => FlagRule::Picked,
                    crate::metadata::xmp::Flag::Rejected => FlagRule::Rejected,
                    crate::metadata::xmp::Flag::Unflagged => FlagRule::Unflagged,
                };
            }
            Narrow::Label(label) => {
                self.narrowing.rules.label = Label::CHOICES
                    .iter()
                    .position(|candidate| *candidate == label)
                    .map(LabelRule::One)
                    .unwrap_or(LabelRule::Any);
            }
            Narrow::Stars(stars) => {
                self.narrowing.rules.min_stars = stars;
                self.narrowing.rules.max_stars = crate::metadata::xmp::MAX_RATING as u8;
            }
        }

        self.narrowing.suspended = false;
        self.filter_visible = true;
        self.apply_narrowing();
    }
}

impl App {
    /// Opens the menu of whichever surface last had the keyboard.
    ///
    /// egui has no `Key::ContextMenu` — its key list runs F1 to F35 and has no
    /// entry for the dedicated Menu key — so `Shift + F10` is the only keyboard
    /// route there is. Each surface opens its own popup anchored to its own
    /// rect rather than at the pointer, which is where a keyboard user's
    /// attention already is.
    pub(super) fn open_context_for_focus(&mut self, _ctx: &egui::Context) {
        // The photograph or the cell, whichever is on screen: those are the two
        // surfaces a keyboard user is on, and the two whose menus carry verbs
        // rather than settings alone.
        crate::ui::surface::ask_for_menu(match self.mode {
            super::Mode::Grid => "cell",
            _ => "photograph",
        });
    }
}

impl App {
    /// Narrows the folder to the photographs carrying one keyword.
    ///
    /// A `String` and so not a `Copy` payload, which is why it is a method
    /// rather than another arm of `Narrow`: `Command` derives `Copy` and is
    /// taken by value everywhere it is read.
    pub(super) fn show_only_keyword(&mut self, keyword: &str) {
        self.narrowing.rules.keyword = keyword.to_string();
        self.narrowing.suspended = false;
        self.filter_visible = true;
        self.apply_narrowing();
    }
}
