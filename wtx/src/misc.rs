//! Miscellaneous

#[cfg(feature = "http2")]
pub(crate) mod bytes_transfer;
#[cfg(feature = "postgres")]
pub(crate) mod counter_writer;
mod hints;
#[cfg(any(feature = "http2", feature = "mysql", feature = "postgres", feature = "web-socket"))]
pub(crate) mod net;
#[cfg(feature = "http2")]
pub(crate) mod span;

mod connection_state;
#[cfg(feature = "aes-gcm")]
mod crypto;
mod either;
mod enum_var_strings;
mod env_vars;
#[cfg(any(feature = "http2", feature = "mysql", feature = "postgres", feature = "web-socket"))]
mod filled_buffer;
mod fn_fut;
mod incomplete_utf8_char;
mod interspace;
mod join_array;
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
mod try_arithmetic;
mod tuple_impls;
mod uri;
mod usize;
mod utf8_errors;
mod wrapper;

#[cfg(feature = "tokio-rustls")]
pub use self::tokio_rustls::{TokioRustlsAcceptor, TokioRustlsConnector};
use crate::{
  calendar::Instant,
  de::{U64String, u64_string},
};
pub use connection_state::ConnectionState;
use core::{any::type_name, future::poll_fn, pin::pin, task::Poll, time::Duration};
#[cfg(feature = "aes-gcm")]
pub use crypto::*;
pub use either::Either;
pub use enum_var_strings::EnumVarStrings;
pub use env_vars::{EnvVars, FromVars};
#[cfg(any(
  feature = "http2",
  feature = "mysql",
  feature = "postgres",
  feature = "web-socket"
))]
pub use filled_buffer::{FilledBuffer, FilledBufferVectorMut};
pub use fn_fut::{FnFut, FnFutWrapper, FnMutFut};
pub use hints::*;
pub use incomplete_utf8_char::{CompletionErr, IncompleteUtf8Char};
pub use interspace::Intersperse;
pub use join_array::JoinArray;
pub use lease::{Lease, LeaseMut};
pub use mem::*;
pub use optimization::*;
pub use poll_once::PollOnce;
#[cfg(feature = "secret")]
pub use secret::Secret;
pub use sensitive_bytes::SensitiveBytes;
pub use single_type_storage::SingleTypeStorage;
pub use suffix_writer::*;
pub use try_arithmetic::*;
pub use uri::{QueryWriter, Uri, UriArrayString, UriBox, UriCow, UriRef, UriReset, UriString};
pub use usize::Usize;
pub use utf8_errors::{BasicUtf8Error, ExtUtf8Error, StdUtf8Error};
pub use wrapper::Wrapper;

/// Hashes a password using the `argon2` algorithm.
#[cfg(feature = "argon2")]
pub fn argon2_pwd<const N: usize>(
  blocks: &mut crate::collection::Vector<argon2::Block>,
  pwd: &[u8],
  salt: &[u8],
) -> crate::Result<[u8; N]> {
  use crate::collection::ExpansionTy;
  use argon2::{Algorithm, Argon2, Params, Version};

  let params = const {
    let output_len = Some(N);
    let Ok(elem) = Params::new(
      Params::DEFAULT_M_COST,
      Params::DEFAULT_T_COST,
      Params::DEFAULT_P_COST,
      output_len,
    ) else {
      panic!();
    };
    elem
  };
  blocks.expand(ExpansionTy::Len(params.block_count()), argon2::Block::new())?;
  let mut out = [0; N];
  let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
  let rslt = argon2.hash_password_into_with_memory(pwd, salt, &mut out, &mut *blocks);
  blocks.clear();
  rslt?;
  Ok(out)
}

/// Deserializes a sequence of elements info `buffer`. Works with any deserializer of any format.
#[cfg(feature = "serde")]
pub fn deserialize_seq_into_buffer_with_serde<'de, D, T>(
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
#[inline]
#[track_caller]
pub fn into_rslt<T>(opt: Option<T>) -> crate::Result<T> {
  opt.ok_or(crate::Error::NoInnerValue(type_name::<T>().into()))
}

/// Deserializes a sequence passing each element to `cb`. Works with any deserializer of any format.
#[cfg(feature = "serde")]
pub fn deserialize_seq_into_cb_with_serde<'de, D, E, T>(
  deserializer: D,
  cb: impl FnMut(T) -> Result<(), E>,
) -> crate::Result<()>
where
  D: serde::de::Deserializer<'de>,
  E: core::fmt::Display,
  T: serde::Deserialize<'de>,
  crate::Error: From<D::Error>,
{
  use core::{any::type_name, fmt::Formatter, marker::PhantomData};
  use serde::{
    Deserialize,
    de::{SeqAccess, Visitor},
  };

  struct LocalVisitor<E, F, T>(PhantomData<E>, F, PhantomData<T>);

  impl<'de, E, F, T> Visitor<'de> for LocalVisitor<E, F, T>
  where
    E: core::fmt::Display,
    F: FnMut(T) -> Result<(), E>,
    T: Deserialize<'de>,
  {
    type Value = ();

    #[inline]
    fn expecting(&self, formatter: &mut Formatter<'_>) -> core::fmt::Result {
      formatter.write_fmt(format_args!("a sequence of `{}`", type_name::<T>()))
    }

    #[inline]
    fn visit_seq<A>(mut self, mut seq: A) -> Result<Self::Value, A::Error>
    where
      A: SeqAccess<'de>,
    {
      while let Some(elem) = seq.next_element()? {
        (self.1)(elem).map_err(serde::de::Error::custom)?;
      }
      Ok(())
    }
  }

  deserializer.deserialize_seq(LocalVisitor(PhantomData, cb, PhantomData))?;
  Ok(())
}

/// Recursively searches `file` starting at `dir`.
#[cfg(feature = "std")]
#[inline]
pub fn find_file(dir: &mut std::path::PathBuf, file: &std::path::Path) -> std::io::Result<()> {
  dir.push(file);
  match std::fs::metadata(&dir) {
    Ok(elem) => {
      if elem.is_file() {
        return Ok(());
      }
    }
    Err(err) => {
      if err.kind() != std::io::ErrorKind::NotFound {
        return Err(err);
      }
    }
  }
  let _ = dir.pop();
  if dir.pop() {
    find_file(dir, file)
  } else {
    Err(std::io::Error::new(
      std::io::ErrorKind::NotFound,
      alloc::format!("`{}` not found", file.display()),
    ))
  }
}

/// A version of `serde_json::from_slice` that aggregates the payload in case of an error.
#[cfg(feature = "serde_json")]
pub fn serde_json_deserialize_from_slice<'any, T>(slice: &'any [u8]) -> crate::Result<T>
where
  T: serde::de::Deserialize<'any>,
{
  match serde_json::from_slice(slice) {
    Ok(elem) => Ok(elem),
    Err(err) => {
      use core::{fmt::Write, str};
      let mut string = alloc::string::String::new();
      let idx = slice.len().min(1024);
      let payload = slice.get(..idx).and_then(|el| str::from_utf8(el).ok()).unwrap_or_default();
      string.write_fmt(format_args!("Error: {err}. Payload: {payload}"))?;
      Err(crate::Error::SerdeJsonDeserialize(string.into()))
    }
  }
}

/// Similar to `collect_seq` of `serde` but expects a `Result`.
#[cfg(feature = "serde")]
pub fn serialize_seq_with_serde<E, I, S, T>(ser: S, into_iter: I) -> Result<S::Ok, S::Error>
where
  E: core::fmt::Display,
  I: IntoIterator<Item = Result<T, E>>,
  S: serde::Serializer,
  T: serde::Serialize,
{
  const fn conservative_size_hint_len(size_hint: (usize, Option<usize>)) -> Option<usize> {
    match size_hint {
      (lo, Some(hi)) if lo == hi => Some(lo),
      _ => None,
    }
  }
  use serde::ser::{Error, SerializeSeq};
  let iter = into_iter.into_iter();
  let mut sq = ser.serialize_seq(conservative_size_hint_len(iter.size_hint()))?;
  for elem in iter {
    sq.serialize_element(&elem.map_err(S::Error::custom)?)?;
  }
  sq.end()
}

/// Sleeps for the specified amount of time.
///
/// Defaults to the selected runtime's reactor, for example, `tokio`. Fallbacks to a naive
/// spin-like approach if no runtime is selected.
#[allow(clippy::unused_async, reason = "depends on the selected set of features")]
#[inline]
pub async fn sleep(duration: Duration) -> crate::Result<()> {
  async fn _naive(duration: Duration) -> crate::Result<()> {
    let now = Instant::now();
    poll_fn(|cx| {
      if now.elapsed()? >= duration {
        return Poll::Ready(Ok(()));
      }
      cx.waker().wake_by_ref();
      Poll::Pending
    })
    .await
  }

  #[cfg(feature = "executor")]
  _naive(duration).await?;

  #[cfg(all(
    feature = "async-net",
    not(any(feature = "embassy-time", feature = "executor", feature = "tokio"))
  ))]
  {
    async_io::Timer::after(duration).await;
  }

  #[cfg(all(
    feature = "embassy-time",
    not(any(feature = "async-net", feature = "executor", feature = "tokio"))
  ))]
  {
    embassy_time::Timer::after(duration.try_into()?).await
  }

  #[cfg(all(
    feature = "tokio",
    not(any(feature = "async-net", feature = "embassy-time", feature = "executor"))
  ))]
  {
    tokio::time::sleep(duration).await;
  }

  #[cfg(not(any(
    feature = "async-net",
    feature = "embassy-time",
    feature = "executor",
    feature = "tokio"
  )))]
  _naive(duration).await?;

  Ok(())
}

/// Requires a `Future` to complete within the specified `duration`.
#[inline]
pub async fn timeout<F>(fut: F, duration: Duration) -> crate::Result<F::Output>
where
  F: Future,
{
  let mut fut_pin = pin!(fut);
  let mut timeout_pin = pin!(sleep(duration));
  poll_fn(|cx| {
    let fut_poll = fut_pin.as_mut().poll(cx);
    let timeout_poll = timeout_pin.as_mut().poll(cx);
    match (fut_poll, timeout_poll) {
      (Poll::Ready(el), Poll::Pending | Poll::Ready(_)) => Poll::Ready(Ok(el)),
      (Poll::Pending, Poll::Ready(_)) => Poll::Ready(Err(crate::Error::ExpiredFuture)),
      (Poll::Pending, Poll::Pending) => {
        cx.waker().wake_by_ref();
        Poll::Pending
      }
    }
  })
  .await
}

/// The current time in milliseconds as a string.
#[inline]
pub fn timestamp_millis_str() -> crate::Result<(u64, U64String)> {
  let number = Instant::now_timestamp(0).map(|el| el.as_millis())?.try_into()?;
  Ok((number, u64_string(number)))
}

/// The current time in nanoseconds as a string.
#[inline]
pub fn timestamp_nanos_str() -> crate::Result<(u64, U64String)> {
  let number = Instant::now_timestamp(0).map(|el| el.as_nanos())?.try_into()?;
  Ok((number, u64_string(number)))
}

/// A tracing register with optioned parameters.
#[cfg(feature = "_tracing-tree")]
#[inline]
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
pub(crate) const fn char_slice(buffer: &mut [u8; 4], ch: char) -> &mut str {
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

#[cfg(test)]
mod tests {
  use crate::{executor::Runtime, misc::sleep};
  use core::time::Duration;

  #[test]
  fn timeout() {
    Runtime::new().block_on(async {
      assert_eq!(crate::misc::timeout(async { 1 }, Duration::from_millis(10)).await.unwrap(), 1);
      assert!(
        crate::misc::timeout(
          async {
            sleep(Duration::from_millis(20)).await.unwrap();
            async { 1 }
          },
          Duration::from_millis(10)
        )
        .await
        .is_err()
      )
    });
  }
}
