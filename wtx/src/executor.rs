//! Simple dependency-free runtime intended for tests, toy programs and demonstrations,

#![allow(clippy::disallowed_types, reason = "traits require the `Arc` from std")]

mod curr_thread_waker;
mod runtime;

pub use runtime::Runtime;
