//! Which of the things the viewer does is on screen.
//!
//! Two of them are ways of looking at a folder and two are ways of working on
//! one, but they are the same kind of choice from the user's side: what the
//! window is for right now.

/// What the main area of the window is showing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Mode {
    /// One photograph, filling the window.
    #[default]
    Image,
    /// The contact sheet.
    Grid,
    /// Renaming the whole folder at once.
    Rename,
    /// Correcting a camera clock across the whole folder.
    TimeShift,
    /// Finding the brackets, stacks and bursts and tidying them away.
    Group,
    /// Unattended playback, filling the screen.
    Slideshow,
}

impl Mode {
    /// Every mode, in the order the menu lists them and the order the key
    /// cycles through them.
    pub const ALL: &'static [Mode] = &[
        Mode::Image,
        Mode::Grid,
        Mode::Rename,
        Mode::TimeShift,
        Mode::Group,
        Mode::Slideshow,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Mode::Image => "Image",
            Mode::Grid => "Gallery",
            Mode::Rename => "Bulk rename",
            Mode::TimeShift => "Shift capture time",
            Mode::Group => "Group shots",
            Mode::Slideshow => "Slideshow",
        }
    }

    /// Whether this mode takes over the whole screen.
    pub fn is_fullscreen(self) -> bool {
        self == Mode::Slideshow
    }

    /// The next mode round, for the key that cycles them.
    pub fn next(self) -> Mode {
        let at = Mode::ALL.iter().position(|mode| *mode == self).unwrap_or(0);

        Mode::ALL[(at + 1) % Mode::ALL.len()]
    }

    /// Whether this mode works on the folder rather than looking at it.
    ///
    /// These are the ones that need the whole folder's metadata read, and the
    /// ones where the image caches can be left alone.
    pub fn is_folder_job(self) -> bool {
        matches!(self, Mode::Rename | Mode::TimeShift | Mode::Group)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_viewer_opens_on_an_image() {
        assert_eq!(Mode::default(), Mode::Image);
    }

    #[test]
    fn cycling_reaches_every_mode_and_comes_back() {
        let mut mode = Mode::default();
        let mut seen = vec![mode];

        for _ in 1..Mode::ALL.len() {
            mode = mode.next();
            seen.push(mode);
        }

        assert_eq!(seen, Mode::ALL);
        assert_eq!(mode.next(), Mode::default(), "and round again");
    }

    #[test]
    fn every_mode_is_in_the_list_that_the_menu_shows() {
        for mode in Mode::ALL {
            assert!(!mode.label().is_empty());
        }

        assert_eq!(Mode::ALL.len(), 6);
    }

    #[test]
    fn the_two_that_change_files_are_the_folder_jobs() {
        assert!(!Mode::Image.is_folder_job());
        assert!(!Mode::Grid.is_folder_job());
        assert!(Mode::Rename.is_folder_job());
        assert!(Mode::TimeShift.is_folder_job());
        assert!(Mode::Group.is_folder_job());
        assert!(!Mode::Slideshow.is_folder_job());
    }

    #[test]
    fn only_the_slideshow_takes_the_whole_screen() {
        let fullscreen: Vec<Mode> = Mode::ALL
            .iter()
            .copied()
            .filter(|mode| mode.is_fullscreen())
            .collect();

        assert_eq!(fullscreen, vec![Mode::Slideshow]);
    }
}
