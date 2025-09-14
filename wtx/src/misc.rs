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
#[cfg(feature = "aes-gcm")]
mod crypto;
mod either;
mod enum_var_strings;
#[cfg(any(feature = "http2", feature = "mysql", feature = "postgres", feature = "web-socket"))]
mod filled_buffer;
mod fn_fut;
mod incomplete_utf8_char;
mod interspace;
mod lease;
mod mem;
mod optimization;
mod poll_once;
#[cfg(feature = "secret")]
mod secret;
mod sensitive_bytes;
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
#[cfg(feature = "aes-gcm")]
pub use crypto::*;
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
pub use mem::*;
pub use optimization::*;
pub use poll_once::PollOnce;
#[cfg(feature = "secret")]
pub use secret::Secret;
pub use sensitive_bytes::SensitiveBytes;
pub use single_type_storage::SingleTypeStorage;
pub use suffix_writer::*;
pub use uri::{QueryWriter, Uri, UriArrayString, UriBox, UriCow, UriRef, UriReset, UriString};
pub use usize::Usize;
pub use utf8_errors::{BasicUtf8Error, ExtUtf8Error, StdUtf8Error};
pub use wrapper::Wrapper;

/// Deserializes a sequence of elements info `buffer`. Works with any deserializer of any format.
#[cfg(feature = "serde")]
pub fn collect_seq_with_serde<'de, D, T>(
  deserializer: D,
  buffer: &mut crate::collection::Vector<T>,
) -> crate::Result<()>
where
  D: serde::de::Deserializer<'de>,
  T: serde::Deserialize<'de>,
  crate::Error: From<D::Error>,
{
  use crate::collection::Vector;
  use core::{any::type_name, fmt::Formatter};
  use serde::{
    Deserialize,
    de::{Error, SeqAccess, Visitor},
  };

  struct LocalVisitor<'any, T>(&'any mut Vector<T>);

  impl<'de, T> Visitor<'de> for LocalVisitor<'_, T>
  where
    T: Deserialize<'de>,
  {
    type Value = ();

    #[inline]
    fn expecting(&self, formatter: &mut Formatter<'_>) -> core::fmt::Result {
      formatter.write_fmt(format_args!("a sequence of `{}`", type_name::<T>()))
    }

    #[inline]
    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
      A: SeqAccess<'de>,
    {
      if let Some(elem) = seq.size_hint() {
        self.0.reserve(elem).map_err(A::Error::custom)?;
      }
      while let Some(elem) = seq.next_element()? {
        self.0.push(elem).map_err(A::Error::custom)?;
      }
      Ok(())
    }
  }

  deserializer.deserialize_seq(LocalVisitor(buffer))?;
  Ok(())
}

/// Useful when a request returns an optional field but the actual usage is within a
/// [`core::result::Result`] context.
#[track_caller]
#[inline]
pub fn into_rslt<T>(opt: Option<T>) -> crate::Result<T> {
  opt.ok_or(crate::Error::NoInnerValue(type_name::<T>().into()))
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
