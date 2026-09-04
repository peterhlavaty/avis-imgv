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
use crate::ui::destinations::Errand;
use crate::ui::empty::{Asked, Nothing, OFFERED};
use crate::ui::menus::Verb;
use crate::view::image_view::bottom_bar::BarAction;
use crate::view::image_view::{COMPARE_PANES, MAX_IMAGES_SHOWN};

use super::input::Command;
use super::panels::MenuAction;
use crate::mode::Mode;

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
    /// Puts every picked-out photograph back.
    ///
    /// Says so, because with the strip shut and the contact sheet somewhere
    /// else there may be nothing on screen that was drawing the set — and a
    /// key that silently changes what every other key means is the worst kind
    /// of silence. A comparison built from the set is ended by the same rule
    /// that follows it, on the next frame.
    pub(super) fn pick_none_out(&mut self) {
        let held = self.grid_view.selected_count();
        if held == 0 {
            return;
        }

        self.grid_view.clear_selection();
        self.notices.say(match held {
            1 => "Put the picked-out photograph back.".to_string(),
            held => format!("Put all {held} picked-out photographs back."),
        });
    }

    /// Pins the picked-out photographs side by side, or this one and its
    /// neighbours when nothing is picked out.
    ///
    /// The same rule the menus already use to decide what a verb is about:
    /// somebody who has picked out four and asks to compare means the four.
    /// Nothing picked out means the frame in hand, which is what the key and
    /// the photograph's own menu have always meant.
    fn compare_marked(&mut self) {
        let picked: Vec<usize> = self.grid_view.selection().iter().collect();

        if picked.len() < 2 {
            self.image_view.start_comparing(COMPARE_PANES);
            self.set_mode(Mode::Image);
            return;
        }

        // What the comparison was built from, so the rule that keeps it on the
        // set does not read the first frame as a change.
        self.compared_from = self.grid_view.selection().clone();

        let taken = self.image_view.compare_these(&picked);
        if taken == 0 {
            self.notices
                .say("Nothing to compare: the folder is not showing those photographs.");
            return;
        }

        if taken < picked.len() {
            self.notices.say(format!(
                "Comparing {taken} of the {} picked out; the panel holds {MAX_IMAGES_SHOWN}.",
                picked.len()
            ));
        }

        self.set_mode(Mode::Image);
    }

    pub(super) fn run_verb(&mut self, verb: Verb, path: PathBuf) {
        // The five turns differ only in what they compose with what is already
        // there, so they are one line rather than five.
        if let Some(extra) = verb.turn() {
            self.turn_by(extra);
            return;
        }

        match verb {
            Verb::Bin => self.delete_open_image(false),
            Verb::PutBack => self.put_back(),
            Verb::DeleteForGood => self.delete_open_image(true),
            Verb::CopyPath => self.copy_paths(),
            Verb::CopyPicture => {
                let crop = self.marked_crop(&path);
                self.copy_picture(&path, crop)
            }
            Verb::ShowInFolder => {
                if !reveal::in_file_manager(&path) {
                    self.notices
                        .say("Could not open the file manager. The log says why.");
                }
            }
            Verb::PickNone => self.pick_none_out(),
            Verb::MoveTo => self.send_somewhere(Errand::Move),
            Verb::CopyTo => self.send_somewhere(Errand::Copy),
            // The photograph's own menu answers this one where it stands,
            // because there it is about the frame under the pointer and its
            // neighbours. From a cell or a thumbnail it is about the set, and
            // the set is not something either view can pin.
            Verb::Compare => self.compare_marked(),
            // The view answers for these; they never reach here. The two
            // marks go the same way, through `BarAction::FlagOne`, because
            // only the view knows which pane the button came down on.
            Verb::Open
            | Verb::Fit
            | Verb::ActualPixels
            | Verb::Fill
            | Verb::Keep
            | Verb::Reject => {}
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

    /// The marking, when the verb is about the very photograph it was drawn
    /// on.
    ///
    /// A cell in the contact sheet is a different photograph and a different
    /// menu, and a rectangle somebody drew on the one they were viewing says
    /// nothing at all about it.
    fn marked_crop(&self, path: &Path) -> Option<egui::Rect> {
        if self.mode == super::Mode::Grid {
            return None;
        }

        if self.image_view.active_path().as_deref() != Some(path) {
            return None;
        }

        self.image_view.marked_area()
    }

    /// Starts a full size decode whose pixels go on the clipboard.
    ///
    /// `crop` is the part of the photograph wanted, in its own coordinates,
    /// nought to one — the whole of it when nothing is marked out. It is
    /// applied after the turn rather than before, because the marking was
    /// drawn on the photograph as it is shown and the pixels are stored
    /// however the camera wrote them.
    fn copy_picture(&mut self, path: &Path, crop: Option<egui::Rect>) {
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
                    .map(|image| upright(&image, crop));

                let _ = sender.send(result);
            });

        match spawned {
            Ok(_) => {
                self.copying.outstanding += 1;
                self.notices.say(match crop {
                    Some(_) => "Copying the marked area…",
                    None => "Copying the picture…",
                });
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
                    let [across, down] = image.size;
                    ctx.copy_image(image);
                    // The size, because a crop is a thing whose size somebody
                    // wants to know and the whole photograph has one too.
                    self.notices
                        .say(format!("Copied {across} × {down} pixels."));
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
            BarAction::FlagOne(index, flag) => self.flag_one(index, flag),
            BarAction::SetAdvancing(on) => {
                self.advancing = on;
                self.settings.tags.advance_after_marking = on;
                self.tag_config.advance_after_marking = on;
                self.save_settings();
            }
            // Written to the file rather than only to the view, on the same
            // rule as every other value a key or a word in the bar nudges: a
            // preference that dies with the session is one somebody sets twice
            // a day.
            BarAction::SetOpening(opening) => {
                self.settings.image_view.opening = opening;
                self.image_view.set_config(self.settings.image_view.clone());
                self.save_settings();
            }
            BarAction::SetKeeping(keeping) => {
                self.settings.image_view.keep_zoom = keeping.zoom;
                self.settings.image_view.keep_pan = keeping.pan;
                self.image_view.set_config(self.settings.image_view.clone());
                self.save_settings();
            }
            BarAction::Settings(path) => self.open_settings_at(path),
            BarAction::ToggleStack => self.apply_command(Command::ToggleStack),
            BarAction::ShowEverything => self.apply_command(Command::SuspendFilter),
            BarAction::ShowMessages => self.open_card(crate::app::cards::Card::Messages),
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

/// Turns a decoded image the right way up, cuts `crop` out of it if there is
/// one, and hands back what egui wants.
///
/// The GPU does the turn by sampling the texture in a different order rather
/// than by copying ninety megabytes, so the pixels held have never been
/// turned. The clipboard has no such trick, and `Orientation::applied` is the
/// one place that turn is written.
///
/// The crop is measured against the photograph as it was shown, which is what
/// makes turning it first the only order that works: a marking drawn on the
/// top left of a portrait frame is not the top left of the pixels a camera
/// held sideways wrote.
fn upright(image: &decoder::DecodedImage, crop: Option<egui::Rect>) -> egui::ColorImage {
    let surface = &image.surface;

    let Some(raw) =
        image::RgbaImage::from_raw(surface.width, surface.height, surface.pixels.to_vec())
    else {
        return egui::ColorImage::filled([1, 1], egui::Color32::TRANSPARENT);
    };

    let turned = image.orientation.applied(&raw);
    let (width, height) = (turned.width(), turned.height());

    let turned =
        match crop.and_then(|crop| crate::view::image_view::area::in_pixels(crop, width, height)) {
            Some((left, top, across, down)) => {
                image::imageops::crop_imm(turned.as_ref(), left, top, across, down).to_image()
            }
            None => turned.into_owned(),
        };

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
        if self.menu_visible || self.deck.holds(crate::app::cards::Card::CheatSheet) {
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
    /// Opens the window holding every key bound to one command.
    ///
    /// The whole list used to open with a row armed somewhere inside it, which
    /// asked a person who right-clicked one thing to find it again among
    /// ninety — and the armed row was off screen whenever the list happened to
    /// be scrolled elsewhere. The eleven **Bind a key to…** rows, the settings
    /// window's **Change this key…** and the cheat sheet all land here, and
    /// **All keys…** in that window is the way on to the list.
    pub(super) fn arm_key(&mut self, path: &'static str) {
        self.keys.arm(path);
        self.open_card(crate::app::cards::Card::OneKey);
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
        use crate::config::kinds::FlagRule;
        use crate::metadata::xmp::Label;
        use crate::view::narrow::LabelRule;

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
            // Whatever is drawn over the photograph owns the surface, and the
            // keyboard has to be able to reach it or the marking's two verbs
            // would be on the second button alone.
            _ if self.image_view.has_marked_area() => "marked area",
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
