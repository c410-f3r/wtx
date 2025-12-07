//! Synchronizing primitives
//!
//! Some of the `no_std` structures located in this module were copied from the
//! <https://github.com/crossbeam-rs/crossbeam> project and modified to fit into `wtx`. On the
//! other hand, some structures are just wrappers or facades of third-parties.

mod arc;
mod async_mutex;
mod atomic_cell;
mod atomic_waker;
mod back_off;
mod cache_padded;
mod fence;
mod mpmc;
mod primitives;
mod ref_counter;
mod seq_lock;
mod sync_mutex;

pub use arc::Arc;
pub use async_mutex::{AsyncMutex, AsyncMutexGuard, AsyncMutexGuardFuture};
pub use atomic_cell::AtomicCell;
pub use atomic_waker::AtomicWaker;
pub use back_off::Backoff;
pub use cache_padded::CachePadded;
pub use fence::fence;
pub use mpmc::*;
pub use primitives::*;
pub use ref_counter::RefCounter;
pub(crate) use seq_lock::SeqLock;
pub use sync_mutex::{SyncMutex, SyncMutexGuard};
