//! What the viewer remembers between one run and the next.
//!
//! Not settings — those are the configuration, and the user writes them. This
//! is where they were: the size and place of the window, the folder that was
//! open, the photograph that was on screen, and for every folder visited
//! lately, which photograph was being looked at in it.
//!
//! The last of those is the one that earns its keep. Culling a shoot is not
//! one sitting: somebody works through four hundred frames, goes to make
//! coffee, opens something else, comes back — and a viewer that starts them at
//! the first frame again has thrown away the only piece of state that took any
//! effort to build.
//!
//! Written on the way out and read on the way in, as JSON beside the
//! configuration. A session file that cannot be read is not an error worth
//! reporting: it means starting where a first run starts, which is a perfectly
//! good place to start.

use std::collections::VecDeque;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::{APPLICATION, ORGANIZATION, QUALIFIER};

/// How many folders' positions are kept.
///
/// Enough to cover the shoots somebody is moving between, small enough that
/// the file stays a file rather than a history.
const REMEMBERED_FOLDERS: usize = 64;

/// Where the window was.
#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq)]
pub struct Geometry {
    pub width: f32,
    pub height: f32,
    /// Absent when the platform does not say, which is normal on Wayland.
    pub x: Option<f32>,
    pub y: Option<f32>,
    pub maximised: bool,
}

impl Geometry {
    /// Whether this is worth restoring.
    ///
    /// A window of no size is what a minimised one reports on some platforms,
    /// and restoring it would open the viewer as a sliver nobody can find.
    pub fn is_usable(&self) -> bool {
        self.width >= 320.0 && self.height >= 240.0
    }
}

/// Where the viewer was when it was last closed.
#[derive(Deserialize, Serialize, Debug, Default, Clone)]
#[serde(default)]
pub struct Session {
    pub window: Option<Geometry>,
    /// The folder that was open.
    pub folder: Option<PathBuf>,
    /// Which photograph was being looked at, per folder, most recent first.
    ///
    /// A list rather than a map so the order is the recency, which is what
    /// decides who is forgotten first.
    pub positions: VecDeque<(PathBuf, PathBuf)>,
}

impl Session {
    /// Where the session file lives.
    pub fn path() -> Option<PathBuf> {
        directories::ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION)
            .map(|dirs| dirs.config_dir().join("session.json"))
    }

    /// Reads it, or hands back an empty one.
    pub fn load() -> Session {
        let Some(path) = Session::path() else {
            return Session::default();
        };

        let Ok(text) = std::fs::read_to_string(&path) else {
            return Session::default();
        };

        // The same byte order mark tolerance the configuration has: a file
        // somebody has opened in a Windows editor should still be read.
        match serde_json::from_str(text.trim_start_matches('\u{feff}')) {
            Ok(session) => session,
            Err(e) => {
                tracing::warn!("The session could not be read, starting fresh: {e}");
                Session::default()
            }
        }
    }

    /// Writes it, quietly.
    ///
    /// A session that cannot be saved costs the next run its starting place
    /// and nothing else, so it is logged rather than reported.
    pub fn save(&self) {
        let Some(path) = Session::path() else {
            return;
        };

        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        match serde_json::to_string_pretty(self) {
            Ok(json) => {
                if let Err(e) = std::fs::write(&path, json) {
                    tracing::warn!("Could not write the session: {e}");
                }
            }
            Err(e) => tracing::warn!("Could not build the session: {e}"),
        }
    }

    /// Which photograph was last being looked at in `folder`.
    pub fn position_in(&self, folder: &Path) -> Option<&Path> {
        self.positions
            .iter()
            .find(|(remembered, _)| remembered == folder)
            .map(|(_, image)| image.as_path())
    }

    /// Records where the viewer is, moving that folder to the front.
    pub fn remember(&mut self, folder: &Path, image: Option<&Path>) {
        self.folder = Some(folder.to_path_buf());

        let Some(image) = image else {
            return;
        };

        self.positions
            .retain(|(remembered, _)| remembered != folder);
        self.positions
            .push_front((folder.to_path_buf(), image.to_path_buf()));

        while self.positions.len() > REMEMBERED_FOLDERS {
            self.positions.pop_back();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn folder(name: &str) -> PathBuf {
        PathBuf::from("/photos").join(name)
    }

    #[test]
    fn a_fresh_session_remembers_nothing() {
        let session = Session::default();

        assert!(session.window.is_none());
        assert!(session.folder.is_none());
        assert!(session.position_in(&folder("trip")).is_none());
    }

    /// The one that earns its keep: coming back to a shoot lands where the
    /// culling stopped rather than at the first frame.
    #[test]
    fn a_folder_is_reopened_where_it_was_left() {
        let mut session = Session::default();
        let trip = folder("trip");

        session.remember(&trip, Some(&trip.join("IMG_204.jpg")));

        assert_eq!(
            session.position_in(&trip),
            Some(trip.join("IMG_204.jpg").as_path())
        );
        assert_eq!(session.folder, Some(trip));
    }

    #[test]
    fn coming_back_to_a_folder_replaces_where_it_was() {
        let mut session = Session::default();
        let trip = folder("trip");

        session.remember(&trip, Some(&trip.join("a.jpg")));
        session.remember(&trip, Some(&trip.join("b.jpg")));

        assert_eq!(session.positions.len(), 1);
        assert_eq!(
            session.position_in(&trip),
            Some(trip.join("b.jpg").as_path())
        );
    }

    #[test]
    fn several_folders_are_remembered_apart() {
        let mut session = Session::default();

        for name in ["one", "two", "three"] {
            let at = folder(name);
            session.remember(&at, Some(&at.join("x.jpg")));
        }

        for name in ["one", "two", "three"] {
            assert!(session.position_in(&folder(name)).is_some(), "{name}");
        }
    }

    /// The file is a file, not a history: the folders nobody has visited
    /// lately are the ones that go.
    #[test]
    fn only_so_many_folders_are_kept() {
        let mut session = Session::default();

        for index in 0..REMEMBERED_FOLDERS + 20 {
            let at = folder(&index.to_string());
            session.remember(&at, Some(&at.join("x.jpg")));
        }

        assert_eq!(session.positions.len(), REMEMBERED_FOLDERS);
        assert!(
            session.position_in(&folder("0")).is_none(),
            "the oldest stayed"
        );
        assert!(
            session
                .position_in(&folder(&(REMEMBERED_FOLDERS + 19).to_string()))
                .is_some(),
            "the newest went"
        );
    }

    /// Opening a folder with nothing in it records the folder without
    /// pretending a photograph was being looked at.
    #[test]
    fn a_folder_with_no_photograph_records_only_the_folder() {
        let mut session = Session::default();
        session.remember(&folder("empty"), None);

        assert_eq!(session.folder, Some(folder("empty")));
        assert!(session.positions.is_empty());
    }

    #[test]
    fn a_window_of_no_size_is_not_restored() {
        let usable = Geometry {
            width: 1280.0,
            height: 800.0,
            x: Some(10.0),
            y: Some(20.0),
            maximised: false,
        };

        assert!(usable.is_usable());
        assert!(!Geometry {
            width: 0.0,
            height: 0.0,
            ..usable
        }
        .is_usable());
    }

    #[test]
    fn a_session_survives_being_written_and_read() {
        let mut session = Session {
            window: Some(Geometry {
                width: 1280.0,
                height: 800.0,
                x: Some(10.0),
                y: Some(20.0),
                maximised: true,
            }),
            ..Session::default()
        };

        session.remember(&folder("trip"), Some(&folder("trip").join("a.jpg")));

        let json = serde_json::to_string(&session).unwrap();
        let read: Session = serde_json::from_str(&json).unwrap();

        assert_eq!(read.window, session.window);
        assert_eq!(read.folder, session.folder);
        assert_eq!(read.positions, session.positions);
    }

    /// A file from an older build, or one somebody has edited badly, costs the
    /// starting place and nothing else.
    #[test]
    fn a_session_that_cannot_be_read_is_an_empty_one() {
        assert!(serde_json::from_str::<Session>("not json").is_err());

        // Missing fields are filled in rather than refused.
        let partial: Session = serde_json::from_str("{}").unwrap();
        assert!(partial.folder.is_none());
    }
}
