#[cfg(not(any(feature = "loom", feature = "portable-atomic")))]
pub use core::sync::atomic::Ordering;
#[cfg(all(feature = "loom", not(any(feature = "portable-atomic"))))]
pub use loom::sync::atomic::Ordering;
#[cfg(feature = "portable-atomic")]
pub use portable_atomic::Ordering;
