//! A bunch of common structures that implement `Future`.
//!
//! This module is only intended for internal usage. Please use the `futures` crate if a feature
//! is not available here.

mod fn_fut;
mod join_array_vector;
mod poll_once;
mod sleep;
mod timeout;

pub use fn_fut::{FnFut, FnFutWrapper, FnMutFut};
pub use join_array_vector::{JoinArrayVector, TryJoinArrayVector};
pub use poll_once::PollOnce;
pub use sleep::Sleep;
pub use timeout::Timeout;
