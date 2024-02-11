//! Miscellaneous

mod array_chunks;
mod async_bounds;
mod either;
mod enum_var_strings;
mod filled_buffer_writer;
mod fn_mut_fut;
mod fx_hasher;
mod generic_time;
mod incomplete_utf8_char;
mod mem_transfer;
mod optimization;
mod partitioned_filled_buffer;
mod poll_once;
mod query_writer;
mod stream;
#[cfg(feature = "tokio-rustls")]
mod tokio_rustls;
mod traits;
mod uri;
mod usize;
mod utf8_errors;

#[cfg(feature = "tokio-rustls")]
pub use self::tokio_rustls::{TokioRustlsAcceptor, TokioRustlsConnector};
pub use async_bounds::AsyncBounds;
use core::{any::type_name, ops::Range, time::Duration};
pub use either::Either;
pub use enum_var_strings::EnumVarStrings;
pub use filled_buffer_writer::FilledBufferWriter;
pub use fn_mut_fut::FnMutFut;
pub use fx_hasher::FxHasher;
pub use generic_time::GenericTime;
pub use incomplete_utf8_char::{CompletionErr, IncompleteUtf8Char};
pub use optimization::*;
pub use poll_once::PollOnce;
pub use query_writer::QueryWriter;
pub use stream::{BytesStream, Stream, TlsStream};
pub use traits::SingleTypeStorage;
pub use uri::{Uri, UriRef, UriString};
pub use usize::Usize;
pub use utf8_errors::{BasicUtf8Error, ExtUtf8Error, StdUtf8Error};
#[allow(
  // Used by other features
  unused_imports
)]
pub(crate) use {
  array_chunks::{ArrayChunks, ArrayChunksMut},
  mem_transfer::_shift_bytes,
  partitioned_filled_buffer::PartitionedFilledBuffer,
};

/// Useful when a request returns an optional field but the actual usage is within a
/// [core::result::Result] context.
#[inline]
#[track_caller]
pub fn into_rslt<T>(opt: Option<T>) -> crate::Result<T> {
  opt.ok_or(crate::Error::NoInnerValue(type_name::<T>()))
}

/// Sleeps for the specified amount of time.
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

#[cfg(feature = "ahash")]
pub(crate) fn _random_state(mut rng: impl crate::rng::Rng) -> ahash::RandomState {
  let (seed0, seed1) = {
    let [a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p] = rng.u8_16();
    (u64::from_ne_bytes([a, b, c, d, e, f, g, h]), u64::from_ne_bytes([i, j, k, l, m, n, o, p]))
  };
  let (seed2, seed3) = {
    let [a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p] = rng.u8_16();
    (u64::from_ne_bytes([a, b, c, d, e, f, g, h]), u64::from_ne_bytes([i, j, k, l, m, n, o, p]))
  };
  ahash::RandomState::with_seeds(seed0, seed1, seed2, seed3)
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
  Ok(buffer.get(start..until).and_then(|el| el.try_into().ok()).unwrap_or_else(_unlikely_dflt))
}

#[cfg(test)]
pub(crate) fn _uri() -> UriString {
  use core::sync::atomic::{AtomicU32, Ordering};
  static PORT: AtomicU32 = AtomicU32::new(7000);
  let uri = alloc::format!("http://127.0.0.1:{}", PORT.fetch_add(1, Ordering::Relaxed));
  UriString::new(uri)
}

#[cold]
#[inline(never)]
#[track_caller]
pub(crate) fn _unlikely_cb<T>(cb: impl FnOnce() -> T) -> T {
  cb()
}

#[cold]
#[inline(never)]
#[track_caller]
pub(crate) fn _unlikely_dflt<T>() -> T
where
  T: Default,
{
  T::default()
}

#[cold]
#[inline(never)]
#[track_caller]
pub(crate) fn _unlikely_elem<T>(elem: T) -> T {
  elem
}

#[cold]
#[inline(never)]
#[track_caller]
pub(crate) const fn _unreachable() -> ! {
  panic!("Entered in a branch that should be impossible. This is a bug!");
}

pub(crate) fn _usize_range_from_u32_range(range: Range<u32>) -> Range<usize> {
  *Usize::from(range.start)..*Usize::from(range.end)
}
