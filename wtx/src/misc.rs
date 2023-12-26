//! Miscellaneous

#![allow(
  // Used by other features
  unused_imports
)]

mod array_chunks;
mod async_bounds;
mod basic_utf8_error;
mod enum_var_strings;
mod filled_buffer_writer;
mod fn_mut_fut;
mod generic_time;
mod partitioned_filled_buffer;
mod query_writer;
mod stream;
#[cfg(feature = "_tokio-rustls-client")]
mod tokio_rustls;
mod traits;
mod uri;
mod wrapper;

#[cfg(feature = "_tokio-rustls-client")]
pub use self::tokio_rustls::*;
#[cfg(test)]
use alloc::string::String;
pub(crate) use array_chunks::ArrayChunksMut;
pub use async_bounds::AsyncBounds;
pub use basic_utf8_error::BasicUtf8Error;
use core::{any::type_name, time::Duration};
pub use enum_var_strings::EnumVarStrings;
pub use filled_buffer_writer::FilledBufferWriter;
pub use fn_mut_fut::FnMutFut;
pub use generic_time::GenericTime;
pub(crate) use partitioned_filled_buffer::PartitionedFilledBuffer;
pub use query_writer::QueryWriter;
pub use stream::{BytesStream, Stream, TlsStream};
pub use traits::SingleTypeStorage;
pub use uri::{Uri, UriRef, UriString};
pub use wrapper::Wrapper;

/// Internally uses `simdutf8` if the feature was activated.
#[inline]
pub fn from_utf8_basic_rslt(bytes: &[u8]) -> Result<&str, BasicUtf8Error> {
  #[cfg(feature = "simdutf8")]
  return simdutf8::basic::from_utf8(bytes).ok().ok_or(BasicUtf8Error {});
  #[cfg(not(feature = "simdutf8"))]
  return core::str::from_utf8(bytes).ok().ok_or(BasicUtf8Error {});
}

/// Useful when a request returns an optional field but the actual usage is within a
/// [core::result::Result] context.
#[inline]
#[track_caller]
pub fn into_rslt<T>(opt: Option<T>) -> crate::Result<T> {
  opt.ok_or(crate::Error::NoInnerValue(type_name::<T>()))
}

/// Sleeps for the specified amount of time.
///
/// Intended for asynchronous usage, i.e., won't block threads.
#[allow(
  // Depends on the selected set of features.
  clippy::unused_async
)]
#[inline]
pub async fn sleep(duration: Duration) -> crate::Result<()> {
  #[cfg(all(feature = "async-std", not(feature = "tokio")))]
  {
    async_std::task::sleep(duration).await;
    Ok(())
  }
  #[cfg(all(feature = "tokio", not(feature = "async-std")))]
  {
    tokio::time::sleep(duration).await;
    Ok(())
  }
  #[cfg(any(
    all(feature = "async-std", feature = "tokio"),
    all(not(feature = "tokio"), not(feature = "async-std"))
  ))]
  {
    // Open to better alternatives
    let now = GenericTime::now()?;
    loop {
      if now.elapsed()? >= duration {
        return Ok(());
      }
    }
  }
}

/// A tracing register with optioned parameters.
#[cfg(feature = "_tracing-subscriber")]
pub fn tracing_subscriber_init() -> Result<(), tracing_subscriber::util::TryInitError> {
  use tracing_subscriber::{
    prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt, EnvFilter,
  };
  let env_filter = EnvFilter::from_default_env();

  let mut tracing_tree = tracing_tree::HierarchicalLayer::default();
  #[cfg(feature = "std")]
  {
    tracing_tree = tracing_tree.with_writer(std::io::stderr);
  }
  tracing_tree = tracing_tree
    .with_indent_lines(true)
    .with_indent_amount(2)
    .with_thread_names(false)
    .with_thread_ids(true)
    .with_verbose_exit(false)
    .with_verbose_entry(false)
    .with_targets(true);
  tracing_subscriber::Registry::default().with(env_filter).with(tracing_tree).try_init()
}

#[cfg(not(feature = "atoi"))]
pub(crate) fn _atoi<T>(bytes: &[u8]) -> crate::Result<T>
where
  T: core::str::FromStr,
  T::Err: Into<crate::Error>,
{
  Ok(from_utf8_basic_rslt(bytes)?.parse().map_err(Into::into)?)
}
#[cfg(feature = "atoi")]
pub(crate) fn _atoi<T>(bytes: &[u8]) -> crate::Result<T>
where
  T: atoi::FromRadix10SignedChecked,
{
  atoi::atoi(bytes).ok_or(crate::Error::AtoiInvalidBytes)
}

#[cfg(test)]
pub(crate) fn _uri() -> UriString {
  use core::sync::atomic::{AtomicU32, Ordering};
  static PORT: AtomicU32 = AtomicU32::new(7000);
  let uri = alloc::format!("http://127.0.0.1:{}", PORT.fetch_add(1, Ordering::Relaxed));
  UriString::new(uri)
}

pub(crate) async fn _read_until<const LEN: usize, S>(
  buffer: &mut [u8],
  read: &mut usize,
  start: usize,
  stream: &mut S,
) -> crate::Result<[u8; LEN]>
where
  [u8; LEN]: Default,
  S: Stream,
{
  let until = start.wrapping_add(LEN);
  for _ in 0..LEN {
    let has_enough_data = *read >= until;
    if has_enough_data {
      break;
    }
    let actual_buffer = buffer.get_mut(*read..).unwrap_or_default();
    let local_read = stream.read(actual_buffer).await?;
    if local_read == 0 {
      return Err(crate::Error::UnexpectedEOF);
    }
    *read = read.wrapping_add(local_read);
  }
  Ok(buffer.get(start..until).and_then(|el| el.try_into().ok()).unwrap_or_default())
}
