//! Two ways to leave a value where the rest of the frame can find it.
//!
//! A widget drawn deep inside a panel has neither the program's fields nor its
//! configuration in hand, and threading either down to it would give thirty
//! call sites a parameter they pass on and never read. So the frame carries
//! two noticeboards instead:
//!
//! - a [`Published`] value, written once a frame by whoever owns it and read
//!   by anything drawing — which panels are up, what each key is called, how
//!   far a rail's handle travels;
//! - a [`Mailbox`], where a widget leaves one ask for whoever empties it — open
//!   the settings at this row, bind a key to this command, put this panel away.
//!
//! Both were written out by hand five times over, in three storage mechanisms,
//! and this is the one of each.
//!
//! # Why a thread, and not a lock
//!
//! Every one of the callers this replaces was a `Mutex` or an atomic, and not
//! one of them ever crossed a thread: the writer is the closure the windowing
//! library calls to build a frame, and so is every reader. They were locks
//! because a `static` has to be, not because anything was shared.
//!
//! Saying so in the type takes the lock off the draw path, and buys something
//! better than speed. Process-wide state cannot be held by two tests at once,
//! so every module that kept some also kept a `static ONE_AT_A_TIME:
//! Mutex<()>` and a line at the top of each test to take it — a serialiser a
//! new test can forget, which is a flaky test waiting to be written. State kept
//! per thread is a serialiser nobody can forget, because the test harness gives
//! each test its own thread.
//!
//! The narrowing is real and is the whole of the cost: a value a *worker* must
//! see is not this. `annotations::sidecar::ADOBE_NAMING` is read on a
//! background writer thread and stays an atomic where it is.
//!
//! # Publishing without allocating
//!
//! [`Published::refill`] exists because the interesting boards hold a list, and
//! the list is rewritten every frame from data that has usually not changed.
//! Building a fresh `Vec` for that is an allocation a frame, which in a program
//! whose per-frame costs multiply by the size of a folder is not a rounding
//! error. `refill` writes over the rows that are there, grows only when the new
//! list is longer, and truncates when it is shorter, so a frame on which
//! nothing moved allocates nothing at all.

use std::cell::RefCell;
use std::thread::LocalKey;

/// A value one owner republishes each frame, read by anything drawing.
///
/// Construct it in a `static` beside the `thread_local!` that holds the value:
///
/// ```
/// use std::cell::RefCell;
/// use avis_imgv::board::Published;
///
/// thread_local! {
///     static WIDTHS_CELL: RefCell<Vec<f32>> = const { RefCell::new(Vec::new()) };
/// }
/// static WIDTHS: Published<Vec<f32>> = Published::kept_in(&WIDTHS_CELL);
///
/// WIDTHS.publish(|widths| widths.push(220.0));
/// assert_eq!(WIDTHS.read(|widths| widths.len()), Some(1));
/// ```
pub struct Published<T: 'static> {
    cell: &'static LocalKey<RefCell<T>>,
}

impl<T: 'static> Published<T> {
    /// Names the slot this board lives in.
    ///
    /// `const` so the caller stays a `static` rather than becoming a
    /// `LazyLock`, which would put a check on every read.
    pub const fn kept_in(cell: &'static LocalKey<RefCell<T>>) -> Self {
        Self { cell }
    }

    /// Reads what is published.
    ///
    /// `None` where the value is already being written, which means a read
    /// from inside a publish: the caller's mistake, and not a state worth
    /// stopping the program over — a menu that draws nothing this frame is
    /// better than a window that closes.
    pub fn read<R>(&self, with: impl FnOnce(&T) -> R) -> Option<R> {
        self.cell
            .with(|held| held.try_borrow().ok().map(|it| with(&it)))
    }

    /// Writes in place. `None` where it is already being read.
    pub fn publish<R>(&self, put: impl FnOnce(&mut T) -> R) -> Option<R> {
        self.cell
            .with(|held| held.try_borrow_mut().ok().map(|mut it| put(&mut it)))
    }

    /// Puts the board back to nothing.
    ///
    /// For a test that wants to start from a known state; the program itself
    /// republishes rather than clearing.
    pub fn forget(&self)
    where
        T: Default,
    {
        self.publish(|it| *it = T::default());
    }
}

impl<T: 'static> Published<Vec<T>> {
    /// Rewrites the list from an iterator, keeping the rows it already has.
    ///
    /// The rows are written over rather than dropped and built again, so a
    /// frame on which nothing changed allocates nothing — which for a row
    /// holding a `String` is the difference between one allocation a frame and
    /// ninety.
    ///
    /// `over` is handed the row to write into and the item to write.
    pub fn refill<I>(&self, items: I, over: impl Fn(&mut T, I::Item))
    where
        I: IntoIterator,
        T: Default,
    {
        self.publish(|held| {
            let mut written = 0;

            for item in items {
                if written == held.len() {
                    held.push(T::default());
                }

                over(&mut held[written], item);
                written += 1;
            }

            held.truncate(written);
        });
    }
}

/// One ask, left for whoever empties the box.
///
/// The last ask of a frame wins. That is not a compromise: the asks are the
/// answer to a menu row being clicked, at most one row is clicked in a frame,
/// and every caller this replaces already behaved this way.
pub struct Mailbox<T: 'static> {
    cell: &'static LocalKey<RefCell<Option<T>>>,
}

impl<T: 'static> Mailbox<T> {
    /// Names the slot this mailbox lives in.
    pub const fn kept_in(cell: &'static LocalKey<RefCell<Option<T>>>) -> Self {
        Self { cell }
    }

    /// Leaves an ask.
    pub fn ask(&self, what: T) {
        self.cell.with(|held| {
            if let Ok(mut held) = held.try_borrow_mut() {
                *held = Some(what);
            }
        });
    }

    /// Takes whatever was asked for, leaving the box empty.
    pub fn take(&self) -> Option<T> {
        self.cell
            .with(|held| held.try_borrow_mut().ok().and_then(|mut held| held.take()))
    }

    /// Takes what was asked for, but only if it is what the caller wants.
    ///
    /// For a box several surfaces read and only one should empty: the
    /// keyboard's ask names the surface whose menu to open, and the other
    /// twenty-nine have to leave it where it is.
    pub fn take_if(&self, wanted: impl FnOnce(&T) -> bool) -> Option<T> {
        self.cell.with(|held| {
            let mut held = held.try_borrow_mut().ok()?;

            match held.as_ref() {
                Some(asked) if wanted(asked) => held.take(),
                _ => None,
            }
        })
    }

    /// Whether anything is waiting, without taking it.
    pub fn waiting(&self) -> bool {
        self.cell
            .with(|held| held.try_borrow().is_ok_and(|held| held.is_some()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    thread_local! {
        static NUMBERS_CELL: RefCell<Vec<u32>> = const { RefCell::new(Vec::new()) };
        static NAMES_CELL: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
        static ASK_CELL: RefCell<Option<&'static str>> = const { RefCell::new(None) };
    }

    static NUMBERS: Published<Vec<u32>> = Published::kept_in(&NUMBERS_CELL);
    static NAMES: Published<Vec<String>> = Published::kept_in(&NAMES_CELL);
    static ASKED: Mailbox<&'static str> = Mailbox::kept_in(&ASK_CELL);

    #[test]
    fn nothing_is_published_to_begin_with() {
        assert_eq!(NUMBERS.read(Vec::len), Some(0));
    }

    #[test]
    fn what_is_published_is_what_is_read() {
        NUMBERS.forget();
        NUMBERS.publish(|held| held.extend([3, 5, 8]));

        assert_eq!(NUMBERS.read(|held| held.clone()), Some(vec![3, 5, 8]));
    }

    /// The board is the frame's, so a reader inside a writer is a caller that
    /// has tied a knot. It gets nothing rather than a panic: a menu drawing
    /// nothing for one frame beats a window closing.
    #[test]
    fn reading_from_inside_a_publish_answers_nothing() {
        NUMBERS.forget();

        let inner = NUMBERS.publish(|_| NUMBERS.read(|held| held.len()));

        assert_eq!(inner, Some(None));
    }

    #[test]
    fn refilling_a_longer_list_grows_it() {
        NAMES.forget();

        NAMES.refill(["a", "b"], |row, name| {
            row.clear();
            row.push_str(name);
        });

        assert_eq!(
            NAMES.read(|held| held.clone()),
            Some(vec!["a".to_string(), "b".to_string()])
        );
    }

    #[test]
    fn refilling_a_shorter_list_truncates_it() {
        NAMES.forget();
        NAMES.refill(["a", "b", "c"], |row, name| {
            row.clear();
            row.push_str(name);
        });

        NAMES.refill(["z"], |row, name| {
            row.clear();
            row.push_str(name);
        });

        assert_eq!(NAMES.read(|held| held.clone()), Some(vec!["z".to_string()]));
    }

    /// The whole reason `refill` exists rather than an assignment: the rows
    /// are the same allocations from one frame to the next, so a board of
    /// ninety strings that did not change costs nothing to republish.
    #[test]
    fn refilling_keeps_the_rows_it_already_had() {
        NAMES.forget();
        NAMES.refill(["first"], |row, name| {
            row.clear();
            row.push_str(name);
        });

        let before = NAMES.read(|held| held[0].as_ptr());

        NAMES.refill(["other"], |row, name| {
            row.clear();
            row.push_str(name);
        });

        // Same capacity, so the same buffer: written over, not built again.
        assert_eq!(NAMES.read(|held| held[0].as_ptr()), before);
        assert_eq!(
            NAMES.read(|held| held[0].clone()),
            Some("other".to_string())
        );
    }

    #[test]
    fn refilling_with_nothing_empties_the_board() {
        NAMES.forget();
        NAMES.refill(["a"], |row, name| {
            row.clear();
            row.push_str(name);
        });

        NAMES.refill(std::iter::empty::<&str>(), |row: &mut String, name| {
            row.clear();
            row.push_str(name);
        });

        assert_eq!(NAMES.read(Vec::len), Some(0));
    }

    #[test]
    fn an_ask_is_taken_once() {
        ASKED.take();

        ASKED.ask("settings");

        assert!(ASKED.waiting());
        assert_eq!(ASKED.take(), Some("settings"));
        assert_eq!(ASKED.take(), None);
        assert!(!ASKED.waiting());
    }

    /// At most one row of one menu is clicked in a frame, so the last ask
    /// wins and every caller this replaces already worked that way.
    #[test]
    fn the_last_ask_of_a_frame_wins() {
        ASKED.take();

        ASKED.ask("first");
        ASKED.ask("second");

        assert_eq!(ASKED.take(), Some("second"));
    }

    /// The keyboard's ask names one surface and thirty read the box, so all
    /// but the one named have to leave it where it is.
    #[test]
    fn an_ask_meant_for_somebody_else_is_left_alone() {
        ASKED.take();
        ASKED.ask("the filmstrip");

        assert_eq!(ASKED.take_if(|asked| *asked == "the photograph"), None);
        assert!(ASKED.waiting());

        assert_eq!(
            ASKED.take_if(|asked| *asked == "the filmstrip"),
            Some("the filmstrip")
        );
        assert!(!ASKED.waiting());
    }

    #[test]
    fn taking_from_an_empty_box_asks_nothing() {
        ASKED.take();

        let mut asked_about = false;
        let taken = ASKED.take_if(|_| {
            asked_about = true;
            true
        });

        assert_eq!(taken, None);
        assert!(!asked_about, "there was nothing to ask about");
    }

    /// The property that lets the three `ONE_AT_A_TIME` serialisers go: two
    /// tests cannot see each other's boards, so none of them has to queue.
    #[test]
    fn a_board_is_this_thread_s_and_no_other_thread_s() {
        NUMBERS.forget();
        NUMBERS.publish(|held| held.push(1));

        let elsewhere = std::thread::spawn(|| NUMBERS.read(Vec::len))
            .join()
            .expect("the thread runs");

        assert_eq!(elsewhere, Some(0));
        assert_eq!(NUMBERS.read(Vec::len), Some(1));
    }
}
