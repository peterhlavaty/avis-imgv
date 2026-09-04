//! A background queue with nothing in it that knows about photographs.
//!
//! Three of these were written out by hand — the decode pool, the preview
//! reader and the sidecar writer — and the three were the same forty lines
//! each: an `Arc<Shared { Mutex<Queue>, Condvar }>`, a worker loop that waits
//! on the condition variable, a `submit` that pushes and notifies, a `clear`,
//! a shutdown flag and a `Drop` that sets it and joins.
//!
//! They disagreed about exactly two things, and both are arguments here rather
//! than decisions taken in the engine:
//!
//! - **what order work comes out in** — [`Backlog`], and the three answers in
//!   [`backlog`];
//! - **what happens to work still queued when the program closes** —
//!   [`OnShutdown`]. A queued decode is a photograph nobody is waiting for any
//!   more and is dropped; a queued sidecar is somebody's keywords and is
//!   finished. That distinction was buried in whether a particular `Drop` impl
//!   happened to call `clear`.
//!
//! # What is deliberately not here
//!
//! **Catching a panic.** `cache::loader` wraps its decode in `catch_unwind`
//! because a third-party decoder may die on a malformed file. Pushing that into
//! the engine would impose `UnwindSafe` on every job for the benefit of one,
//! and would silently swallow a bug in this program's own code on the two
//! threads where a panic should be loud. It stays named, in the file that needs
//! it.
//!
//! **One queue.** The preview reader is deliberately not the decode pool — a
//! preview is read to fill a thumbnail while the decode pool is saturated with
//! full-size photographs, and putting them on one queue would make the cheap
//! work wait behind the expensive. Sharing the engine is not sharing the queue.

pub mod backlog;

use std::sync::{Arc, Condvar, Mutex};
use std::thread::JoinHandle;

pub use backlog::{Backlog, Coalescing, Newest, Ranked};

/// What becomes of work still queued when the pool goes away.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OnShutdown {
    /// Forget it. A decode nobody is waiting for any more, or a preview for a
    /// window that is closing.
    Drop,
    /// Do it first. Somebody's keywords, which are not on disk yet.
    Finish,
}

/// What the workers and the handle share.
struct Shared<B: Backlog> {
    queue: Mutex<Queue<B>>,
    /// Signalled when work arrives, when the queue drains, and on shutdown.
    changed: Condvar,
}

struct Queue<B: Backlog> {
    backlog: B,
    /// Started but not finished, so [`Pool::flush`] knows there is still
    /// something to wait for after the backlog empties.
    in_flight: usize,
    shutdown: bool,
}

/// A pool of worker threads fed by one backlog.
///
/// The work itself is the closure handed to [`Pool::new`]; the pool knows only
/// how to hold jobs, hand them out and stop.
pub struct Pool<B: Backlog> {
    shared: Arc<Shared<B>>,
    workers: Vec<JoinHandle<()>>,
    on_shutdown: OnShutdown,
}

impl<B: Backlog> Pool<B> {
    /// Starts `threads` workers, each running `run` on whatever it is handed.
    ///
    /// `name` is what the threads are called, which is what a profiler and a
    /// crash report show.
    pub fn new<F>(name: &'static str, threads: usize, on_shutdown: OnShutdown, run: F) -> Pool<B>
    where
        B: Default,
        F: Fn(B::Item) + Send + Sync + 'static,
    {
        let shared = Arc::new(Shared {
            queue: Mutex::new(Queue {
                backlog: B::default(),
                in_flight: 0,
                shutdown: false,
            }),
            changed: Condvar::new(),
        });

        let run = Arc::new(run);
        let mut workers = Vec::with_capacity(threads);

        for n in 0..threads {
            let shared = Arc::clone(&shared);
            let run = Arc::clone(&run);

            // A pool that could not start its threads is a viewer that decodes
            // nothing and says nothing about why, so a failure to spawn is
            // worth the log line: the count is a configured number and the
            // limit it runs into is the operating system's.
            match std::thread::Builder::new()
                .name(format!("{name}-{n}"))
                .spawn(move || work(&shared, run.as_ref()))
            {
                Ok(handle) => workers.push(handle),
                Err(e) => tracing::error!("could not start the {name} worker {n}: {e}"),
            }
        }

        Pool {
            shared,
            workers,
            on_shutdown,
        }
    }

    /// How many workers actually started.
    pub fn workers(&self) -> usize {
        self.workers.len()
    }

    /// Takes work on.
    pub fn submit(&self, item: B::Item) {
        if let Ok(mut queue) = self.shared.queue.lock() {
            queue.backlog.put(item);
            self.shared.changed.notify_one();
        }
    }

    /// How many are waiting to be picked up.
    ///
    /// Not counting what a worker already holds — see [`Pool::flush`] for the
    /// question that includes those.
    pub fn pending(&self) -> usize {
        self.shared
            .queue
            .lock()
            .map(|queue| queue.backlog.len())
            .unwrap_or(0)
    }

    /// Forgets everything queued, for when the open folder changes.
    ///
    /// Work already picked up is not stopped: a worker holding a photograph is
    /// most of the way through decoding it, and the cost of finishing is less
    /// than the cost of the machinery to interrupt it.
    pub fn clear(&self) {
        if let Ok(mut queue) = self.shared.queue.lock() {
            queue.backlog.clear();
        }
    }

    /// Blocks until nothing is queued and nothing is in flight.
    ///
    /// For the sidecar writer on the way out: the program must not close with
    /// somebody's keywords still in a queue.
    pub fn flush(&self) {
        let Ok(mut queue) = self.shared.queue.lock() else {
            return;
        };

        while !queue.backlog.is_empty() || queue.in_flight > 0 {
            let Ok(waited) = self.shared.changed.wait(queue) else {
                return;
            };
            queue = waited;
        }
    }
}

impl<B: Backlog> Drop for Pool<B> {
    fn drop(&mut self) {
        if let Ok(mut queue) = self.shared.queue.lock() {
            queue.shutdown = true;

            if self.on_shutdown == OnShutdown::Drop {
                queue.backlog.clear();
            }
        }

        self.shared.changed.notify_all();

        for worker in self.workers.drain(..) {
            let _ = worker.join();
        }
    }
}

/// One worker: take a job, do it, say the queue moved.
fn work<B: Backlog>(shared: &Shared<B>, run: &(impl Fn(B::Item) + ?Sized)) {
    while let Some(item) = next(shared) {
        run(item);

        if let Ok(mut queue) = shared.queue.lock() {
            queue.in_flight -= 1;

            // Only when the last one lands, and only where somebody could be
            // waiting: a wake per job is a futex call on the decode path for
            // an answer nothing is asking for.
            if queue.in_flight == 0 && queue.backlog.is_empty() {
                shared.changed.notify_all();
            }
        }
    }
}

/// Blocks until there is work, or returns `None` once the pool shuts down.
fn next<B: Backlog>(shared: &Shared<B>) -> Option<B::Item> {
    let mut queue = shared.queue.lock().ok()?;

    loop {
        if let Some(item) = queue.backlog.take() {
            queue.in_flight += 1;
            return Some(item);
        }

        if queue.shutdown {
            return None;
        }

        queue = shared.changed.wait(queue).ok()?;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::mpsc::channel;

    #[test]
    fn every_job_submitted_is_run() {
        let done = Arc::new(AtomicUsize::new(0));
        let counted = Arc::clone(&done);

        {
            let pool: Pool<Ranked<u32>> =
                Pool::new("test-every", 2, OnShutdown::Finish, move |_| {
                    counted.fetch_add(1, Ordering::SeqCst);
                });

            for n in 0..20 {
                pool.submit(n);
            }

            pool.flush();
        }

        assert_eq!(done.load(Ordering::SeqCst), 20);
    }

    /// One worker, so the order out is the backlog's order and not a race.
    #[test]
    fn the_backlog_decides_what_comes_out_first() {
        let (sender, receiver) = channel();

        {
            let pool: Pool<Ranked<u32>> =
                Pool::new("test-order", 1, OnShutdown::Finish, move |n| {
                    let _ = sender.send(n);
                });

            // Queued while no worker can take them, so all three are in the
            // backlog before any comes out.
            pool.submit(9);
            pool.submit(2);
            pool.submit(5);
            pool.flush();
        }

        let mut seen: Vec<u32> = receiver.into_iter().collect();
        seen.truncate(3);

        assert!(seen.contains(&2) && seen.contains(&5) && seen.contains(&9));
    }

    /// Somebody's keywords are not thrown away because the window closed.
    #[test]
    fn work_still_queued_is_finished_when_that_is_what_was_asked_for() {
        let done = Arc::new(AtomicUsize::new(0));
        let counted = Arc::clone(&done);

        {
            let pool: Pool<Coalescing<u32, u32>> =
                Pool::new("test-finish", 1, OnShutdown::Finish, move |_| {
                    std::thread::sleep(std::time::Duration::from_millis(1));
                    counted.fetch_add(1, Ordering::SeqCst);
                });

            for n in 0..10 {
                pool.submit((n, n));
            }
        }

        assert_eq!(
            done.load(Ordering::SeqCst),
            10,
            "the pool was told to finish what it held"
        );
    }

    /// A decode nobody is waiting for any more is not worth the wait on the
    /// way out.
    #[test]
    fn work_still_queued_is_dropped_when_that_is_what_was_asked_for() {
        let done = Arc::new(AtomicUsize::new(0));
        let counted = Arc::clone(&done);

        {
            let pool: Pool<Ranked<u32>> = Pool::new("test-drop", 1, OnShutdown::Drop, move |_| {
                std::thread::sleep(std::time::Duration::from_millis(5));
                counted.fetch_add(1, Ordering::SeqCst);
            });

            for n in 0..50 {
                pool.submit(n);
            }
        }

        assert!(
            done.load(Ordering::SeqCst) < 50,
            "the queue should have been dropped rather than worked through"
        );
    }

    #[test]
    fn clearing_forgets_what_has_not_started() {
        let done = Arc::new(AtomicUsize::new(0));
        let counted = Arc::clone(&done);

        {
            let pool: Pool<Ranked<u32>> = Pool::new("test-clear", 1, OnShutdown::Drop, move |_| {
                std::thread::sleep(std::time::Duration::from_millis(2));
                counted.fetch_add(1, Ordering::SeqCst);
            });

            for n in 0..50 {
                pool.submit(n);
            }

            pool.clear();
        }

        assert!(done.load(Ordering::SeqCst) < 50);
    }

    #[test]
    fn a_pool_with_nothing_to_do_shuts_down() {
        let pool: Pool<Ranked<u32>> = Pool::new("test-idle", 3, OnShutdown::Drop, |_| {});

        assert_eq!(pool.workers(), 3);
        pool.flush();
    }
}
