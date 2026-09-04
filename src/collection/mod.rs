//! What is open, in what order, and what each photograph carries.
//!
//! Which photographs the viewer is showing is not a question about drawing,
//! and the answer to it does not need a window. It lived under `src/view/`
//! and `src/app/` because the two views and the shell are what *ask* — but
//! narrowing a folder, folding a burst into one cell, remembering which frames
//! are picked out and mapping a position in the collection onto a position in
//! the store are all arithmetic, and every one of these files was already free
//! of the toolkit.
//!
//! Moving them says so. `config` and `history` both read `Narrowing`, and
//! neither has any business pointing at the drawing layer to do it; the
//! history watches `Selection` for the same reason.
//!
//! This is a job the program does, so it is a directory named for the job —
//! not a directory of a kind of file. The drawing that acts on any of it stays
//! in `src/view/`, which is where drawing lives.

pub mod narrow;
pub mod place;
pub mod selection;
pub mod stacking;
pub mod stacks;
pub mod visible;
