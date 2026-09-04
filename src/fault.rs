//! What went wrong, said once.
//!
//! The crate's error handling was already disciplined — five `unwrap`s outside
//! the tests — but its error *reporting* was not. Three hundred functions
//! returned `Option`, so a failure and an absence were the same value and the
//! reason for the failure was thrown away at the point it was known. What
//! reached the user was a sentence written at the call site, and the same
//! sentence was written at several of them.
//!
//! This module is the shape the modules' own errors take, not a single crate
//! error enum. Each job keeps its own — `decoder::DecodeError`,
//! `metadata::xmp::Error`, `organize::Error` — because a caller that wants to
//! know whether a sidecar could not be parsed or could not be written is
//! asking about XMP, not about the program. What is shared is the two
//! questions everything above them asks:
//!
//! - **How badly did it go?** [`Fault::severity`], so the notice bar can tell a
//!   warning from a failure without the call site deciding.
//! - **What was it about?** [`Fault::subject`], so the sentence can name the
//!   photograph without every call site remembering to interpolate it.
//!
//! # Why a trait and not an enum
//!
//! An enum over every failure in the program would put `decoder` and
//! `organize` in one type, which means every module that returns an error
//! depends on every module that has one. The trait is implemented where the
//! error is declared and nothing needs to know the others exist. It is only
//! ever used behind a reference — reporting is not on the draw path — so the
//! dynamic dispatch costs nothing that matters.
//!
//! # Saying it
//!
//! [`said`] is the sentence a person reads. It is written *by the error*, once,
//! rather than at each of the thirty call sites that used to write their own —
//! three of which wrote the same one. A call site that wants to add context
//! adds it to the error, where the next caller gets it too.

use std::fmt::Display;
use std::path::Path;

/// How badly something went, from the point of view of somebody looking at
/// photographs.
///
/// Not from the point of view of the code: a sidecar that could not be written
/// is a `Failure` because the user's keywords are not on disk, while a folder
/// entry that could not be read is a `Warning` because the other nine hundred
/// were.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum Severity {
    /// Worth saying, nothing lost.
    #[default]
    Said,
    /// Something did not happen, and the program carried on without it.
    Warning,
    /// Something the user asked for did not happen.
    Failure,
}

/// An error that can say for itself how bad it was and what it was about.
///
/// Implemented beside each module's own error type. The blanket parts —
/// [`said`] — work over any of them.
pub trait Fault: std::error::Error {
    /// How badly it went.
    ///
    /// Defaults to a failure, because an error type whose author has not
    /// thought about this is more likely to be one than not.
    fn severity(&self) -> Severity {
        Severity::Failure
    }

    /// The file it was about, where there is one.
    ///
    /// The sentence names it, so it is carried by the error rather than
    /// interpolated by the caller — which is what stops two call sites
    /// spelling the same failure differently.
    fn subject(&self) -> Option<&Path> {
        None
    }

    /// What the program was trying to do, as a verb phrase that fits into
    /// "Could not …".
    ///
    /// `"write the sidecar"`, `"read the folder"`. Lower case, no full stop.
    fn doing(&self) -> &'static str;
}

/// The sentence a person reads.
///
/// `Could not write the sidecar for DSC0142.jpg: permission denied.` — the verb
/// from [`Fault::doing`], the file from [`Fault::subject`], the reason from
/// `Display`. Written once here instead of thirty times at the call sites.
pub fn said<E: Fault + ?Sized>(fault: &E) -> String {
    let mut sentence = format!("Could not {}", fault.doing());

    if let Some(path) = fault.subject() {
        let named = path.file_name().map_or_else(
            || path.display().to_string(),
            |name| name.to_string_lossy().into_owned(),
        );

        sentence.push_str(" for ");
        sentence.push_str(&named);
    }

    let reason = fault.to_string();
    if !reason.is_empty() {
        sentence.push_str(": ");
        sentence.push_str(&reason);
    }

    sentence.push('.');
    sentence
}

/// The line for the log, which wants the whole path and the chain of causes
/// rather than the file's name.
///
/// The user's sentence and the log's line are different questions and used to
/// be the same string, which is why the log said `DSC0142.jpg` when what was
/// needed to find the fault was the directory it was in.
pub fn logged<E: Fault + ?Sized>(fault: &E) -> String {
    let mut line = format!("could not {}", fault.doing());

    if let Some(path) = fault.subject() {
        line.push_str(&format!(" [{}]", path.display()));
    }

    line.push_str(&format!(": {fault}"));

    let mut cause = std::error::Error::source(fault);
    while let Some(next) = cause {
        line.push_str(&format!("\n  caused by: {next}"));
        cause = next.source();
    }

    line
}

/// Something that went wrong which has no type of its own yet.
///
/// A staging post, not a destination: it exists so a call site can be moved off
/// a bare `Option` in one commit and given a real error in the next, rather
/// than the two changes having to land together. Every use of it is a place
/// still owing a typed error.
#[derive(Debug, thiserror::Error)]
#[error("{reason}")]
pub struct Untyped {
    reason: String,
    doing: &'static str,
    severity: Severity,
    subject: Option<std::path::PathBuf>,
}

impl Untyped {
    pub fn new(doing: &'static str, reason: impl Display) -> Self {
        Self {
            reason: reason.to_string(),
            doing,
            severity: Severity::Failure,
            subject: None,
        }
    }

    pub fn about(mut self, path: impl Into<std::path::PathBuf>) -> Self {
        self.subject = Some(path.into());
        self
    }

    pub fn at(mut self, severity: Severity) -> Self {
        self.severity = severity;
        self
    }
}

impl Fault for Untyped {
    fn severity(&self) -> Severity {
        self.severity
    }

    fn subject(&self) -> Option<&Path> {
        self.subject.as_deref()
    }

    fn doing(&self) -> &'static str {
        self.doing
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[derive(Debug, thiserror::Error)]
    #[error("permission denied")]
    struct Denied(PathBuf);

    impl Fault for Denied {
        fn subject(&self) -> Option<&Path> {
            Some(&self.0)
        }

        fn doing(&self) -> &'static str {
            "write the sidecar"
        }
    }

    #[derive(Debug, thiserror::Error)]
    #[error("not a directory")]
    struct Skipped;

    impl Fault for Skipped {
        fn severity(&self) -> Severity {
            Severity::Warning
        }

        fn doing(&self) -> &'static str {
            "read the folder"
        }
    }

    #[test]
    fn a_sentence_names_the_verb_the_file_and_the_reason() {
        let fault = Denied(PathBuf::from("/photos/DSC0142.jpg"));

        assert_eq!(
            said(&fault),
            "Could not write the sidecar for DSC0142.jpg: permission denied."
        );
    }

    /// The user's sentence names the file; the log's line names the path, so
    /// somebody reading it afterwards can find which folder it was in.
    #[test]
    fn the_log_gets_the_whole_path() {
        let fault = Denied(PathBuf::from("/photos/holiday/DSC0142.jpg"));

        assert!(logged(&fault).contains("/photos/holiday/DSC0142.jpg"));
        assert!(!said(&fault).contains("holiday"));
    }

    #[test]
    fn a_fault_about_no_file_says_only_what_it_was_doing() {
        assert_eq!(
            said(&Skipped),
            "Could not read the folder: not a directory."
        );
    }

    /// A failure is what the user asked for not happening; a warning is the
    /// program carrying on without something. The call site used to decide,
    /// which is why the same failure reached the bar two ways.
    #[test]
    fn how_badly_it_went_is_the_error_s_answer_and_not_the_caller_s() {
        assert_eq!(Skipped.severity(), Severity::Warning);
        assert_eq!(
            Denied(PathBuf::from("/a.jpg")).severity(),
            Severity::Failure,
            "the default, for an error whose author did not say"
        );
    }

    #[test]
    fn an_untyped_fault_carries_what_it_was_given() {
        let fault = Untyped::new("open the folder", "no such directory")
            .about("/photos/gone")
            .at(Severity::Warning);

        assert_eq!(fault.severity(), Severity::Warning);
        assert_eq!(
            said(&fault),
            "Could not open the folder for gone: no such directory."
        );
    }

    /// The trait is used behind a reference, so it has to stay object safe.
    #[test]
    fn a_fault_can_be_carried_as_a_reference() {
        let faults: Vec<&dyn Fault> = vec![&Skipped];

        assert_eq!(
            faults.iter().map(|f| said(*f)).collect::<Vec<_>>(),
            vec!["Could not read the folder: not a directory."]
        );
    }
}
