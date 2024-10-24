// All `no_std` structures located in this module were copied from the
// https://github.com/crossbeam-rs/crossbeam project and modified to fit into `wtx`.
//
// As well as many other structures of many other modules, everything here should have been
// provided by the standard library. Unfortunately, several individuals spread across different
// levels of authority defend a minimum std with "reference" crates.

mod atomic_cell;
mod atomic_waker;
mod back_off;
mod cache_padded;
mod seq_lock;

pub use atomic_cell::AtomicCell;
pub use atomic_waker::AtomicWaker;
pub use back_off::Backoff;
pub use cache_padded::CachePadded;
