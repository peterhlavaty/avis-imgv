//! When a change takes effect, and where a value is read.
//!
//! Twenty-six settings do not take effect until the next launch and nothing
//! anywhere says so, while the two things the interface can change today both
//! apply immediately — so the mental model the program teaches is exactly wrong
//! for all twenty-six. Recording it per row is the first half of the repair;
//! honouring it is the second.

/// When a change reaches the screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Effect {
    /// The next frame.
    Live,
    /// Once the stores are built again, which happens when the gesture ends.
    ///
    /// `stores::image_store` and `stores::thumbnail_store` are pure functions
    /// of the configuration and `ImageStore::new` takes its settings by value,
    /// so a rebuild is constructing a fresh store and re-seeding it — which is
    /// exactly how a folder is opened. But a rail on true per-frame apply would
    /// rebuild the cache sixty times a second, so these commit on
    /// `drag_stopped`, on focus loss, or on the click itself.
    Rebuild,
    /// When the folder is read again, which the viewer does for itself.
    Reopen,
    /// The next time the viewer starts, and nothing sooner.
    ///
    /// A badge means *your change has not taken effect*. Using it for a change
    /// that has is what teaches people to ignore it, so a setting about the
    /// *next launch* is not a restart: it gets a sentence and no badge. After
    /// this stage exactly one field in the whole window is a `Restart`.
    Restart,
    /// A setting about what the *next* launch does.
    ///
    /// Which mode it opens in, which folder, which panels are up. The change
    /// has taken effect — the field now says what it says — and there is
    /// nothing on screen for it to change. A badge here would be a lie, and
    /// the lie is what teaches people to ignore the badge that is not one.
    NextLaunch,
    /// Nothing to take effect: a button, or a value nobody sets.
    None,
}

impl Effect {
    /// What the row says under its control, or nothing where there is nothing
    /// to say. Under the control rather than in a tooltip, because a restart
    /// requirement is a field requirement.
    pub fn sentence(self) -> Option<&'static str> {
        match self {
            Effect::Live => None,
            Effect::Rebuild => Some("Takes effect when you let go; the caches are filled again."),
            Effect::Reopen => {
                Some("Takes effect when the folder is read again, which happens by itself.")
            }
            Effect::Restart => Some("Takes effect the next time the viewer starts."),
            Effect::NextLaunch => Some("This is about the next launch; nothing on screen changes."),
            Effect::None => None,
        }
    }

    /// Whether this row carries the restart badge.
    pub fn badged(self) -> bool {
        self == Effect::Restart
    }

    /// The badge itself, drawn beside the label.
    ///
    /// A badge is a bug report rather than a feature: it says *your change has
    /// not taken effect*. Exactly one field in the whole window carries it.
    pub const BADGE: &'static str = "↻";
}

/// Where a binding is read, which is where it can clash.
///
/// This replaces the section heading for clash detection. `clash()` compared
/// only within an editor section on the stated ground that the gallery and the
/// image view are never on screen at once — but "General" is live in every
/// mode, because `input::collect` runs unconditionally every frame. So a
/// General binding colliding with an image-view one is the collision that
/// actually bites, and the old test asserted that silence about it was correct.
/// It is not: Quit on the gallery's scroll key means the folder scrolls and the
/// program exits.
///
/// A scope states where a binding is *read*; a heading only happens to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    /// Read every frame, in every mode.
    Everywhere,
    /// Read only while a photograph is on screen.
    ImageView,
    /// Read only in the contact sheet.
    Gallery,
    /// Read only while an overlay owns the keyboard: the navigator, the tree,
    /// the destination panel, a question.
    Overlay,
    /// Not a key at all.
    None,
}

impl Scope {
    /// Whether two bindings in these scopes could ever be read on one frame.
    ///
    /// `Everywhere` overlaps with everything, including itself. Two scopes that
    /// are never both live cannot collide, which is why the gallery and the
    /// image view may share a key.
    pub fn overlaps(self, other: Scope) -> bool {
        match (self, other) {
            (Scope::None, _) | (_, Scope::None) => false,
            (Scope::Everywhere, _) | (_, Scope::Everywhere) => true,
            (a, b) => a == b,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Scope::Everywhere => "everywhere",
            Scope::ImageView => "the image view",
            Scope::Gallery => "the contact sheet",
            Scope::Overlay => "an overlay",
            Scope::None => "nowhere",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The collision that actually bites, and the one the old check was blind
    /// to: a key read in every mode against one read in the image view.
    #[test]
    fn a_general_binding_collides_with_an_image_view_one() {
        assert!(Scope::Everywhere.overlaps(Scope::ImageView));
        assert!(Scope::ImageView.overlaps(Scope::Everywhere));
        assert!(Scope::Everywhere.overlaps(Scope::Everywhere));
    }

    /// And the one the old reasoning got right: the two views are never both
    /// on screen, so they may share a key.
    #[test]
    fn the_two_views_may_share_a_key() {
        assert!(!Scope::ImageView.overlaps(Scope::Gallery));
        assert!(Scope::ImageView.overlaps(Scope::ImageView));
    }

    #[test]
    fn something_that_is_not_a_key_collides_with_nothing() {
        for scope in [
            Scope::Everywhere,
            Scope::ImageView,
            Scope::Gallery,
            Scope::Overlay,
            Scope::None,
        ] {
            assert!(!Scope::None.overlaps(scope));
            assert!(!scope.overlaps(Scope::None));
        }
    }

    /// Exactly one kind of change carries the badge, and a change about the
    /// next launch is not one of them.
    #[test]
    fn only_a_restart_is_badged() {
        assert!(Effect::Restart.badged());
        assert!(!Effect::Live.badged());
        assert!(!Effect::Rebuild.badged());
        assert!(!Effect::Reopen.badged());
        assert!(!Effect::NextLaunch.badged());
        assert!(!Effect::None.badged());
    }

    #[test]
    fn everything_that_waits_says_what_it_is_waiting_for() {
        for effect in [
            Effect::Rebuild,
            Effect::Reopen,
            Effect::Restart,
            Effect::NextLaunch,
        ] {
            assert!(effect.sentence().is_some(), "{effect:?} says nothing");
        }
        assert!(Effect::Live.sentence().is_none());
    }
}
