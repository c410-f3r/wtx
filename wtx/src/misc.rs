//! Miscellaneous

#[cfg(feature = "http2")]
pub(crate) mod bytes_transfer;
#[cfg(feature = "postgres")]
pub(crate) mod counter_writer;
pub(crate) mod hints;
#[cfg(any(feature = "http2", feature = "mysql", feature = "postgres", feature = "web-socket"))]
pub(crate) mod net;
#[cfg(feature = "http2")]
pub(crate) mod span;

mod auto_clear;
mod connection_state;
mod either;
mod enum_var_strings;
#[cfg(any(feature = "http2", feature = "mysql", feature = "postgres", feature = "web-socket"))]
mod filled_buffer;
mod fn_fut;
mod incomplete_utf8_char;
mod interspace;
mod lease;
mod optimization;
mod single_type_storage;
mod suffix_writer;
#[cfg(feature = "tokio-rustls")]
mod tokio_rustls;
mod tuple_impls;
mod uri;
mod usize;
mod utf8_errors;
mod wrapper;

#[cfg(feature = "tokio-rustls")]
pub use self::tokio_rustls::{TokioRustlsAcceptor, TokioRustlsConnector};
pub use auto_clear::AutoClear;
pub use connection_state::ConnectionState;
use core::{any::type_name, time::Duration};
pub use either::Either;
pub use enum_var_strings::EnumVarStrings;
#[cfg(any(
  feature = "http2",
  feature = "mysql",
  feature = "postgres",
  feature = "web-socket"
))]
pub use filled_buffer::{FilledBuffer, FilledBufferVectorMut};
pub use fn_fut::{FnFut, FnFutWrapper, FnMutFut};
pub use incomplete_utf8_char::{CompletionErr, IncompleteUtf8Char};
pub use interspace::Intersperse;
pub use lease::{Lease, LeaseMut};
pub use optimization::*;
pub use single_type_storage::SingleTypeStorage;
pub use suffix_writer::*;
pub use uri::{QueryWriter, Uri, UriArrayString, UriBox, UriCow, UriRef, UriReset, UriString};
pub use usize::Usize;
pub use utf8_errors::{BasicUtf8Error, ExtUtf8Error, StdUtf8Error};
pub use wrapper::Wrapper;

/// Useful when a request returns an optional field but the actual usage is within a
/// [`core::result::Result`] context.
#[track_caller]
#[inline]
pub fn into_rslt<T>(opt: Option<T>) -> crate::Result<T> {
  opt.ok_or(crate::Error::NoInnerValue(type_name::<T>()))
}

/// Sleeps for the specified amount of time.
#[allow(clippy::unused_async, reason = "depends on the selected set of features")]
#[inline]
pub async fn sleep(duration: Duration) -> crate::Result<()> {
  async fn _naive(duration: Duration) -> crate::Result<()> {
    let now = crate::calendar::Instant::now();
    core::future::poll_fn(|cx| {
      if now.elapsed()? >= duration {
        return core::task::Poll::Ready(Ok(()));
      }
      cx.waker().wake_by_ref();
      core::task::Poll::Pending
    })
    .await
  }

  #[cfg(feature = "executor")]
  return _naive(duration).await;
  #[cfg(all(feature = "tokio", not(any(feature = "executor"))))]
  {
    tokio::time::sleep(duration).await;
    Ok(())
  }
  #[cfg(not(any(feature = "executor", feature = "tokio")))]
  return _naive(duration).await;
}

/// A tracing register with optioned parameters.
#[cfg(feature = "_tracing-tree")]
pub fn tracing_tree_init(
  fallback_opt: Option<&str>,
) -> Result<(), tracing_subscriber::util::TryInitError> {
  use tracing_subscriber::{
    EnvFilter, prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt,
  };
  let fallback = fallback_opt.unwrap_or("");
  let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(fallback));
  let tracing_tree = tracing_tree::HierarchicalLayer::default()
    .with_deferred_spans(true)
    .with_indent_amount(2)
    .with_indent_lines(true)
    .with_span_retrace(true)
    .with_targets(true)
    .with_thread_ids(true)
    .with_thread_names(true)
    .with_timer(crate::calendar::TracingTreeTimer)
    .with_verbose_entry(false)
    .with_verbose_exit(false)
    .with_writer(std::io::stderr);
  tracing_subscriber::Registry::default().with(env_filter).with(tracing_tree).try_init()
}

// It is important to enforce the array length to avoid panics
#[expect(clippy::disallowed_methods, reason = "that is the only allowed place")]
pub(crate) fn char_slice(buffer: &mut [u8; 4], ch: char) -> &mut str {
  ch.encode_utf8(buffer)
}

#[cfg(all(feature = "foldhash", any(feature = "http2", feature = "mysql", feature = "postgres")))]
pub(crate) fn random_state<RNG>(rng: &mut RNG) -> foldhash::fast::FixedState
where
  RNG: crate::rng::Rng,
{
  let [a, b, c, d, e, f, g, h] = rng.u8_8();
  foldhash::fast::FixedState::with_seed(u64::from_ne_bytes([a, b, c, d, e, f, g, h]))
}

#[cfg(feature = "postgres")]
pub(crate) fn usize_range_from_u32_range(range: core::ops::Range<u32>) -> core::ops::Range<usize> {
  *Usize::from(range.start)..*Usize::from(range.end)
}
