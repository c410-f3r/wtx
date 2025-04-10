//! Some of the `no_std` structures located in this module were copied from the
//! <https://github.com/crossbeam-rs/crossbeam> project and modified to fit into `wtx`. On the
//! other hand, some structures are just wrappers or facades of third-parties.

#[macro_use]
mod macros;

mod arc;
mod atomic_cell;
mod atomic_waker;
mod back_off;
mod cache_padded;
mod fence;
mod ordering;
mod seq_lock;

pub use arc::Arc;
pub use atomic_cell::AtomicCell;
pub use atomic_waker::AtomicWaker;
pub use back_off::Backoff;
pub use cache_padded::CachePadded;
pub use fence::fence;
pub use ordering::Ordering;
pub use seq_lock::SeqLock;

create_atomic_primitive!(AtomicBool, bool);
create_atomic_primitive!(AtomicU32, u32);
create_atomic_primitive!(AtomicU64, u64);
create_atomic_primitive!(AtomicUsize, usize);
