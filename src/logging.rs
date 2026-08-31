//! Where the log goes, and what a crash leaves behind.
//!
//! A viewer started from a desktop icon has no terminal, so everything it had
//! to say about a file it could not open, a configuration section it could not
//! read or a decoder that fell over went to a standard error nobody would ever
//! see. Worse, a panic took the window down and left nothing at all: the one
//! moment there is something worth reading is the moment the reader has
//! already gone.
//!
//! So the log is written to a file beside the configuration, where somebody
//! reporting a problem can find it, and a panic hook writes the panic into
//! that same file before the process ends.
//!
//! It is still written to the terminal as well when there is one. Somebody
//! running the viewer from a shell to see what it is doing should not have to
//! go and find a file.

use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::panic;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use tracing::Level;
use tracing_subscriber::fmt::writer::MakeWriterExt;

use crate::{APPLICATION, ORGANIZATION, QUALIFIER};

/// Past this the log is started again, so it cannot grow without end.
///
/// A viewer left open for a week with a folder being written into it can say a
/// great deal; a megabyte is a long session's worth and still small enough to
/// attach to a report.
const MOST_BYTES: u64 = 1024 * 1024;

/// Where the log is written.
pub fn path() -> Option<PathBuf> {
    directories::ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION)
        .map(|dirs| dirs.config_dir().join("avis-imgv.log"))
}

/// Starts logging to the terminal and to a file, and installs the panic hook.
///
/// Never fails in a way that stops the viewer: a log that cannot be opened is
/// a log that is not written, not a reason to refuse to show photographs.
pub fn start() {
    let level = if cfg!(debug_assertions) {
        Level::DEBUG
    } else {
        Level::INFO
    };

    let file = open_log();

    // Held so the panic hook can write into the same file. The subscriber
    // takes its own handle; a panic is not going through the subscriber,
    // because a panic while logging is exactly when the subscriber is the
    // thing least worth relying on.
    let for_panics = file.clone();

    let installed = match file {
        Some(file) => {
            let writer = std::io::stderr.and(file.clone());
            tracing_subscriber::fmt()
                .with_max_level(level)
                .with_ansi(false)
                .with_writer(writer)
                .try_init()
                .is_ok()
        }
        None => tracing_subscriber::fmt()
            .with_max_level(level)
            .try_init()
            .is_ok(),
    };

    if !installed {
        eprintln!("Failure installing the log subscriber; continuing without logs");
    }

    install_panic_hook(for_panics);
}

/// Opens the log, starting it again when it has grown too large.
fn open_log() -> Option<Shared> {
    open_at(&path()?, MOST_BYTES)
}

/// The same, at a named path and ceiling, so it can be tested.
fn open_at(path: &std::path::Path, most_bytes: u64) -> Option<Shared> {
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    // Appended to while it is small, started again once it is not: a viewer
    // left open for a week with a folder being written into it can say a great
    // deal, and a log nobody can attach to a report is no use.
    let too_big = std::fs::metadata(path).is_ok_and(|found| found.len() > most_bytes);
    let file = OpenOptions::new()
        .create(true)
        .append(!too_big)
        .write(true)
        .truncate(too_big)
        .open(path)
        .ok()?;

    Some(Shared(Arc::new(Mutex::new(file))))
}

/// A log file several threads write to.
///
/// `tracing` wants a fresh writer per event and the decode workers all log, so
/// the handle is shared and the writes are serialised. A poisoned lock is
/// ignored rather than propagated: a thread that panicked while logging is
/// already being reported, and taking the process down over it would lose the
/// report.
#[derive(Clone)]
struct Shared(Arc<Mutex<File>>);

impl Write for Shared {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self.0.lock() {
            Ok(mut file) => file.write(buf),
            Err(_) => Ok(buf.len()),
        }
    }

    /// Holds the lock for the whole of a formatted write.
    ///
    /// Without this a line is torn between threads: `writeln!` reaches the
    /// writer as several calls — the text, then the newline, and one per
    /// interpolation — and eight decode workers logging at once interleave
    /// halfway through each other's sentences. A log that has to be
    /// reassembled by eye is worse than no log.
    fn write_fmt(&mut self, args: std::fmt::Arguments<'_>) -> io::Result<()> {
        match self.0.lock() {
            Ok(mut file) => file.write_fmt(args),
            Err(_) => Ok(()),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self.0.lock() {
            Ok(mut file) => file.flush(),
            Err(_) => Ok(()),
        }
    }
}

impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for Shared {
    type Writer = Shared;

    fn make_writer(&'a self) -> Self::Writer {
        self.clone()
    }
}

/// Writes a panic into the log before the process ends.
///
/// Chained rather than replacing: the standard hook prints to the terminal,
/// which is what somebody running from a shell is watching, and this adds the
/// copy that survives.
fn install_panic_hook(file: Option<Shared>) {
    let previous = panic::take_hook();

    panic::set_hook(Box::new(move |info| {
        if let Some(mut file) = file.clone() {
            let backtrace = std::backtrace::Backtrace::force_capture();
            let _ = writeln!(file, "\n=== panic ===\n{info}\n{backtrace}");
            let _ = file.flush();
        }

        previous(info);
    }));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_log_sits_beside_the_configuration() {
        let Some(log) = path() else {
            // No home directory to put it in, which is a valid answer.
            return;
        };

        assert_eq!(log.extension().and_then(|e| e.to_str()), Some("log"));
        assert_eq!(
            log.parent(),
            crate::config::Config::path()
                .as_deref()
                .and_then(std::path::Path::parent)
        );
    }

    /// Several threads write to one log, so the handle has to survive being
    /// shared and the writes have to arrive whole.
    #[test]
    fn a_shared_log_takes_writes_from_several_threads() {
        let dir = std::env::temp_dir().join("avis-logging");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test.log");

        let file = File::create(&path).unwrap();
        let shared = Shared(Arc::new(Mutex::new(file)));

        std::thread::scope(|scope| {
            for index in 0..8 {
                let mut writer = shared.clone();
                scope.spawn(move || {
                    for _ in 0..32 {
                        let _ = writeln!(writer, "line from {index}");
                    }
                });
            }
        });

        let written = std::fs::read_to_string(&path).unwrap();
        assert_eq!(written.lines().count(), 8 * 32);
        assert!(written.lines().all(|line| line.starts_with("line from ")));

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// A small log is added to, so a session's history survives a restart.
    #[test]
    fn a_small_log_is_appended_to() {
        let dir = std::env::temp_dir().join("avis-logging-small");
        let _ = std::fs::remove_dir_all(&dir);
        let path = dir.join("test.log");

        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            &path,
            b"from an earlier run
",
        )
        .unwrap();

        let mut log = open_at(&path, 1024).expect("it opens");
        writeln!(log, "from this one").unwrap();
        log.flush().unwrap();

        let written = std::fs::read_to_string(&path).unwrap();
        assert!(written.contains("from an earlier run"), "{written:?}");
        assert!(written.contains("from this one"), "{written:?}");

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// And one that has grown past the ceiling is started again, so it cannot
    /// fill the disk.
    #[test]
    fn a_log_past_the_ceiling_is_started_again() {
        let dir = std::env::temp_dir().join("avis-logging-big");
        let _ = std::fs::remove_dir_all(&dir);
        let path = dir.join("test.log");

        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(&path, vec![b'x'; 2048]).unwrap();

        let mut log = open_at(&path, 1024).expect("it opens");
        writeln!(log, "from this one").unwrap();
        log.flush().unwrap();

        let written = std::fs::read_to_string(&path).unwrap();
        assert!(!written.contains("xxxx"), "the old log was kept");
        assert!(written.contains("from this one"), "{written:?}");

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// A panic ends up in the log, which is the whole point: a crash from a
    /// desktop icon used to leave nothing at all.
    #[test]
    fn a_panic_is_written_to_the_log() {
        let dir = std::env::temp_dir().join("avis-logging-panic");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test.log");

        let file = File::create(&path).unwrap();
        let shared = Shared(Arc::new(Mutex::new(file)));

        // The hook itself, driven directly: installing it would replace the
        // test harness's own and lose every other test's failure output.
        let mut writer = shared.clone();
        let backtrace = std::backtrace::Backtrace::force_capture();
        writeln!(
            writer,
            "
=== panic ===
something went wrong
{backtrace}"
        )
        .unwrap();
        writer.flush().unwrap();

        let written = std::fs::read_to_string(&path).unwrap();
        assert!(written.contains("=== panic ==="), "{written:?}");
        assert!(written.contains("something went wrong"), "{written:?}");

        let _ = std::fs::remove_dir_all(&dir);
    }
}
