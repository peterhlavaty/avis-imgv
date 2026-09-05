//! Moving through the folder.
//!
//! Every way in and out of a picture goes through `select`, which is where the
//! zoom and position of the one being left are put away and the one arrived at
//! is put back where it was.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use eframe::epaint::Vec2;

use crate::actions::Callback;
use crate::cache::{StoreConfig, StoreStats};
use crate::config::{ImageViewConfig, Motion, SlideshowConfig};
use crate::metadata::Metadata;

use super::opening::Opens;
use super::slideshow::Slideshow;
use super::viewports::place_of;
use super::viewports::{Keep, Place, Viewports};
use super::{zoom, ImageView};
use crate::collection::visible::Visible;
use crate::config::kinds::Opening;

impl ImageView {
    /// Opens a new collection, optionally starting on a specific image.
    pub fn set_images(&mut self, paths: Vec<PathBuf>, selected: Option<&Path>) {
        let selected = selected
            .and_then(|path| paths.iter().position(|candidate| candidate == path))
            .unwrap_or(0);

        self.visible = Visible::everything(paths.len());
        self.store.set_paths(paths);
        self.viewports.clear();
        self.select(selected);
    }

    /// Narrows or reorders what is walked through, without disturbing the
    /// caches.
    ///
    /// The store still holds every photograph, so a filter applied in the
    /// middle of a cull costs a vector rather than a folder's worth of
    /// decoding. The cursor stays on the photograph it was on when that
    /// photograph is still shown, and lands on its nearest neighbour when it
    /// is not — which is what rejecting a frame with the rejects hidden has to
    /// do.
    pub fn set_visible(&mut self, visible: Visible) {
        let staying = self.store.path(self.cursor).map(Path::to_path_buf);
        self.visible = visible;

        let wanted = staying
            .and_then(|path| self.store.index_of(&path))
            .and_then(|index| self.visible.nearest(index))
            .and_then(|position| self.visible.at(position));

        if let Some(index) = wanted {
            self.select(index);
        }
    }

    /// How many photographs are on show, and where in them the cursor is.
    pub fn position(&self) -> (usize, usize) {
        let shown = self.visible.len();
        let at = self.visible.position_of(self.cursor).unwrap_or(0);

        (at, shown)
    }

    /// Moves to a position in what is on show, rather than in the store.
    pub fn select_position(&mut self, position: usize) {
        if let Some(index) = self.visible.at(position) {
            self.select(index);
        }
    }

    /// Starts or stops the slideshow.
    ///
    /// Starting it fills the screen and hides the status bar; stopping it puts
    /// the picture back where the user had it, because a drifting zoom is not
    /// somewhere anybody chose to be.
    pub fn set_slideshow(&mut self, running: bool, config: &SlideshowConfig) {
        if running == self.slideshow.is_some() {
            return;
        }

        self.slideshow_config = config.clone();
        self.slideshow = running.then(|| Slideshow::new(config));
        // The photograph on screen opens again under whichever of the two
        // answers now applies.
        self.viewport.opened = false;
        self.frame.enabled = running && config.start_with_frame_enabled;

        if !running {
            zoom::fit(&mut self.viewport);
            self.viewport.pan = Vec2::ZERO;
        }
    }

    /// Takes a changed slideshow configuration, running or not.
    ///
    /// Everything applies while the window is open, and the duration and the
    /// motion are now chosen from the slideshow's own menu *while it is
    /// running* — a change that waited for the next start would read as the
    /// menu having done nothing. The clock is not restarted: the picture on
    /// screen keeps the time it has already had.
    ///
    /// A changed motion re-opens the photograph, because the motion is what
    /// decides whether it is fitted or filled and the latch would otherwise
    /// hold it at the answer the old one gave.
    pub fn set_slideshow_config(&mut self, config: &SlideshowConfig) {
        if self.slideshow_config == *config {
            return;
        }

        let moves = self.slideshow_config.motion != config.motion;
        self.slideshow_config = config.clone();

        if let Some(slideshow) = &mut self.slideshow {
            slideshow.set_config(config);

            if moves {
                self.viewport.opened = false;
            }
        }
    }

    /// Takes a changed configuration, for when the keyboard map is edited.
    pub fn set_config(&mut self, config: ImageViewConfig) {
        self.frame.relative_size = config.frame_size_relative_to_image;

        // The same clamp the constructor applies, for the same reason: the
        // keys widen and narrow the comparison for the session, and a change
        // made in the settings window is a change made now.
        self.images_shown = config
            .nr_images_shown
            .clamp(1, crate::view::image_view::MAX_IMAGES_SHOWN);

        self.config = config;
    }

    /// What the pointer does, which every one of these takes effect on the
    /// next frame drawn: they are read where they are used and held nowhere.
    pub fn set_mouse(&mut self, mouse: crate::config::MouseConfig) {
        self.mouse = mouse;
    }

    /// Asks this view for something a gesture was bound to.
    ///
    /// Carried out on the next frame rather than now, because the caller is
    /// the application and the view wants the same context the keys are read
    /// with.
    pub fn queue(&mut self, command: crate::view::image_view::input::Command) {
        self.queued.push(command);
    }

    /// The grey behind the photograph, as the configuration spells it.
    pub fn set_backdrop(&mut self, hex: &str) {
        self.backdrop = hex.to_string();
    }

    /// Which mask is painted over the photograph, if any.
    ///
    /// For the two figures under the histogram, which offer to put the
    /// clipping mask on and have to say so in the words for the state they
    /// find: a row offering what is already there is a row that does nothing.
    pub fn marking(&self) -> crate::decoder::overlays::Overlay {
        self.marking
    }

    /// Paints the clipping mask, or takes it off if it is already on.
    ///
    /// What "Blown 3.4 %" means, done where it is read. Focus peaking is a
    /// different mask and is replaced rather than kept: the two cannot both be
    /// painted, and the figure that was clicked is about the clipping.
    pub fn mark_clipping(&mut self) {
        use crate::decoder::overlays::Overlay;

        self.marking = if self.marking == Overlay::Clipping {
            Overlay::Off
        } else {
            Overlay::Clipping
        };
    }

    pub fn selected_index(&self) -> usize {
        self.cursor
    }

    /// The store position the keys are about, if any is.
    ///
    /// The same as `selected_index` except while a comparison has had its
    /// focused photograph taken out of the picked-out set, when nothing on
    /// screen is the one being marked and the honest answer is none. Every
    /// accessor below is this one and a lookup, so "no current photograph"
    /// reaches the status bar, the keyword panel, the side panel and every
    /// command through the `None` they all already handle.
    pub fn focused(&self) -> Option<usize> {
        // Nothing being current is only reachable inside a comparison, and the
        // rule is here rather than in the flag so that it cannot be got wrong
        // by forgetting to clear one: with a single photograph on screen it is
        // the one being looked at, and there is nothing else it could be.
        (!self.focus_off || self.comparing.is_none()).then_some(self.cursor)
    }

    /// The photograph the cursor sits on, whether or not it is current.
    ///
    /// For the two readers that want a place rather than a subject: the
    /// history, which names the move it recorded, and the session, which
    /// remembers where the folder was left.
    pub fn path_at_cursor(&self) -> Option<PathBuf> {
        self.store.path(self.cursor).map(Path::to_path_buf)
    }

    pub fn active_path(&self) -> Option<PathBuf> {
        self.store.path(self.focused()?).map(Path::to_path_buf)
    }

    /// The tones of the photograph on screen, for the side panel.
    pub fn active_histogram(&self) -> Option<&crate::decoder::histogram::Histogram> {
        self.store.histogram(self.focused()?)
    }

    pub fn active_metadata(&self) -> Option<&Metadata> {
        self.store.metadata(self.focused()?)
    }

    /// Metadata read from the whole of the active file, once it is decoded.
    pub fn active_decoded_metadata(&self) -> Option<&Metadata> {
        self.store.decoded_metadata(self.focused()?)
    }

    /// Leaves no photograph current, keeping where the last one was.
    ///
    /// What taking the focused frame out of the picked-out set comes to: the
    /// frame goes from the panel and nothing takes its place, because the
    /// person said which one they no longer wanted rather than which one they
    /// wanted next. The arrow keys carry on from where it was, which is why
    /// the cursor is left where it is rather than moved.
    pub fn release_focus(&mut self) {
        self.focus_off = true;
    }

    /// Whether no photograph is current.
    pub fn nothing_is_current(&self) -> bool {
        self.focus_off
    }

    pub fn stats(&self) -> StoreStats {
        self.store.stats()
    }

    pub fn take_callback(&mut self) -> Option<Callback> {
        self.callback.take()
    }

    /// Moves to `index`, which is where the caches centre themselves.
    ///
    /// The image being left keeps its zoom and pan, and the one arrived at
    /// gets whatever it was left at, so a folder can be walked without losing
    /// the place found in each picture.
    pub fn select(&mut self, index: usize) {
        let previous = self.cursor;
        let was_off = self.focus_off;

        self.cursor = if self.store.is_empty() {
            0
        } else {
            index.min(self.store.len() - 1)
        };

        // Going to a photograph is what makes one current again: every route
        // to here — a key, a thumbnail, a click on a pane — is somebody
        // pointing at one, and the only way to have none is to say so.
        self.focus_off = false;

        self.store.set_cursor(self.cursor);

        if self.cursor == previous && !was_off {
            return;
        }

        // A rectangle drawn over one photograph means nothing over the next
        // one, and a marking left behind is a marking somebody copies by
        // accident.
        self.area.clear();

        // Cloned rather than borrowed: the viewports outlive the borrow of
        // the store that produced the path.
        if let Some(path) = self.store.path(previous).map(Path::to_path_buf) {
            self.viewports.save(&path, &self.viewport);
        }

        // Remembered even when it was not worth an entry of its own: repeating
        // a plain fitted view onto the next picture is still a thing to ask
        // for, and it is what undoes an accidental repeat.
        self.previous_place = place_of(&self.viewport);

        self.viewport.reset_for_new_image();

        let arrived = self.active_path();
        let keeping = self.keeping();
        self.viewports.arrive(
            arrived.as_deref(),
            &mut self.viewport,
            keeping,
            self.previous_place,
        );

        if let Some(slideshow) = &mut self.slideshow {
            slideshow.restart();
        }
    }

    /// Where the open photograph is zoomed and panned to.
    ///
    /// The part of the viewport that belongs to the picture rather than to the
    /// view: the latches saying what a *new* image should do are a preference
    /// and are not something to put back.
    pub fn place(&self) -> Place {
        place_of(&self.viewport)
    }

    /// How large the photograph is being drawn, as a percentage of its own
    /// pixels — the number the bottom bar shows.
    ///
    /// Nought before the first frame has drawn it, there being no geometry to
    /// ask yet.
    pub fn zoom_percent(&self) -> f32 {
        self.metrics.percentage_zoom
    }

    pub fn set_place(&mut self, place: Place) {
        Viewports::put(&mut self.viewport, place);
    }

    pub fn select_path(&mut self, path: &Path) {
        if let Some(index) = self.store.index_of(path) {
            self.select(index);
        }
    }

    /// Takes a photograph that has appeared into the collection, at `index`.
    ///
    /// The cursor follows the photograph it was on rather than the position it
    /// was at: a frame landing earlier in the folder must not move the one
    /// being looked at out from under the viewer. What is on show is left to
    /// the caller, which knows the filter and the order.
    pub fn insert(&mut self, index: usize, path: PathBuf) {
        self.store.insert(index, path);

        if index <= self.cursor {
            self.cursor += 1;
        }
    }

    /// Takes a photograph out of the collection.
    ///
    /// Which photograph is left on screen depends on which one went. Losing
    /// the one being looked at keeps the *position*, so what appears is the
    /// one that is now next — the single most complained about detail of
    /// culling in Lightroom, and the reason this is written out rather than
    /// left to a rule. Losing one from below it keeps the *photograph*, by
    /// following it down: a file deleted elsewhere while this viewer has the
    /// folder open must not step the viewer forward a frame.
    pub fn pop(&mut self, path: &Path) {
        let Some(index) = self.store.index_of(path) else {
            return;
        };

        let looking_at = self.cursor;

        self.store.remove(index);
        self.visible.remove_shifting(index);
        self.viewports.forget(path);

        let wanted = if index < looking_at {
            looking_at - 1
        } else {
            looking_at
        };

        let landing = self
            .visible
            .nearest(wanted.min(self.store.len().saturating_sub(1)))
            .and_then(|position| self.visible.at(position))
            .unwrap_or(0);

        self.select(landing);
    }

    pub fn reload(&mut self, path: &Path) {
        if let Some(index) = self.store.index_of(path) {
            self.store.reload(index);
        }
    }

    /// Turns a photograph on the card by any orientation, without decoding it
    /// again. Undo takes the difference between the two orientations.
    pub fn turn_by(&mut self, path: &Path, extra: crate::metadata::Orientation) {
        if let Some(index) = self.store.index_of(path) {
            self.store.turn_by(index, extra);
        }
    }

    /// Builds this view's store again on new settings, keeping the folder.
    ///
    /// Returns whether it had to: the settings window commits on every gesture
    /// and the store is only rebuilt when something it was built from has
    /// actually moved.
    pub fn rebuild_store(&mut self, config: StoreConfig, output_profile: Arc<str>) -> bool {
        if self.store.config() == config {
            return false;
        }

        self.store.rebuild(config, output_profile);
        true
    }

    /// Which corner the photograph's own details are drawn in, as the key
    /// cycling it has left them.
    pub fn overlay_corner(&self) -> crate::config::kinds::Corner {
        self.config.overlay_corner
    }

    /// What a photograph opens at, as the settings say and the key has left
    /// them.
    ///
    /// The settings themselves rather than what the canvas will do: this is
    /// the half of the mirror that writes the file, and a slideshow's answer
    /// is not a preference anybody expressed.
    pub fn opens(&self) -> Opens {
        Opens {
            at: self.config.opening,
            percent: self.config.opening_percent,
        }
    }

    /// What the photograph on screen actually opens at.
    ///
    /// A slideshow answers for itself. Filling the screen is what all but one
    /// of its motions start from, and `Motion::Still` is the one that says
    /// *the whole picture, fitted to the screen* — which the latch the
    /// constructor used to set quietly contradicted, so a still slideshow
    /// started by the `--slideshow` switch cropped every photograph in it.
    pub(super) fn opens_at(&self) -> Opens {
        let at = match self.slideshow.as_ref().map(Slideshow::motion) {
            None => return self.opens(),
            Some(Motion::Still) => Opening::Fit,
            Some(_) => Opening::Fill,
        };

        Opens { at, ..self.opens() }
    }

    /// Whether the magnification and the corner carry from one photograph to
    /// the next, as the two toggles in the status bar have left them.
    pub fn keeping(&self) -> Keep {
        Keep {
            zoom: self.config.keep_zoom,
            pan: self.config.keep_pan,
        }
    }

    /// How many photographs are drawn side by side, as the keys have left it.
    pub fn images_shown(&self) -> usize {
        self.images_shown
    }

    pub fn next_image(&mut self) {
        if self.should_wait() {
            return;
        }

        self.step(|visible, at| visible.next(at));
    }

    pub fn previous_image(&mut self) {
        self.step(|visible, at| visible.previous(at));
    }

    /// Moves to the first or the last photograph on show.
    pub fn jump_to_end(&mut self, last: bool) {
        let position = if last {
            self.visible.len().saturating_sub(1)
        } else {
            0
        };

        self.select_position(position);
    }

    /// Moves by a run of photographs at once, for `Page Up` and `Page Down`.
    pub fn page(&mut self, forward: bool, by: usize) {
        let (at, shown) = self.position();
        if shown == 0 {
            return;
        }

        let wanted = if forward {
            (at + by).min(shown - 1)
        } else {
            at.saturating_sub(by)
        };

        self.select_position(wanted);
    }

    /// One step through what is on show.
    fn step(&mut self, next: impl Fn(&Visible, usize) -> Option<usize>) {
        let (at, _) = self.position();

        if let Some(position) = next(&self.visible, at) {
            self.select_position(position);
        }
    }
}
