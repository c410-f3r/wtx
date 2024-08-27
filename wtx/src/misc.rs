//! Miscellaneous

mod array_chunks;
mod array_string;
mod array_vector;
mod atomic_waker;
mod blocks_queue;
mod connection_state;
mod either;
mod enum_var_strings;
mod filled_buffer_writer;
mod fn_fut;
mod from_radix_10;
mod generic_time;
mod incomplete_utf8_char;
mod iter_wrapper;
mod lease;
mod lock;
mod mem_transfer;
mod optimization;
mod partitioned_filled_buffer;
mod query_writer;
mod queue;
mod queue_utils;
mod ref_counter;
mod rng;
mod role;
mod single_type_storage;
mod span;
mod stream;
#[cfg(feature = "tokio-rustls")]
mod tokio_rustls;
mod tuple_impls;
mod uri;
mod usize;
mod utf8_errors;
mod vector;

#[cfg(feature = "tokio-rustls")]
pub use self::tokio_rustls::{TokioRustlsAcceptor, TokioRustlsConnector};
pub use array_chunks::{ArrayChunks, ArrayChunksMut};
pub use array_string::{ArrayString, ArrayStringError};
pub use array_vector::{ArrayVector, ArrayVectorError};
pub use atomic_waker::AtomicWaker;
pub use blocks_queue::{Block, BlocksQueue, BlocksQueueError};
pub use connection_state::ConnectionState;
use core::{any::type_name, fmt::Write, ops::Range, time::Duration};
pub use either::Either;
pub use enum_var_strings::EnumVarStrings;
pub use filled_buffer_writer::FilledBufferWriter;
pub use fn_fut::*;
pub use from_radix_10::FromRadix10;
pub use generic_time::GenericTime;
pub use incomplete_utf8_char::{CompletionErr, IncompleteUtf8Char};
pub use iter_wrapper::IterWrapper;
pub use lease::{Lease, LeaseMut};
pub use lock::{Lock, SyncLock};
pub use optimization::*;
pub use query_writer::QueryWriter;
pub use queue::{Queue, QueueError};
pub use ref_counter::RefCounter;
pub use rng::*;
pub use role::Role;
pub use single_type_storage::SingleTypeStorage;
pub use stream::{BytesStream, Stream, StreamReader, StreamWithTls, StreamWriter};
pub use uri::{Uri, UriArrayString, UriRef, UriString};
pub use usize::Usize;
pub use utf8_errors::{BasicUtf8Error, ExtUtf8Error, StdUtf8Error};
pub use vector::{Vector, VectorError};
#[allow(unused_imports, reason = "used in other features")]
pub(crate) use {
  mem_transfer::_shift_copyable_chunks,
  partitioned_filled_buffer::PartitionedFilledBuffer,
  span::{_Entered, _Span},
};

/// Useful when a request returns an optional field but the actual usage is within a
/// [`core::result::Result`] context.
#[inline]
#[track_caller]
pub fn into_rslt<T>(opt: Option<T>) -> crate::Result<T> {
  opt.ok_or(crate::Error::NoInnerValue(type_name::<T>()))
}

/// Similar to `collect_seq` of `serde` but expects a `Result`.
#[cfg(feature = "serde")]
pub fn serde_collect_seq_rslt<E, I, S, T>(ser: S, into_iter: I) -> Result<(), E>
where
  E: From<S::Error>,
  I: IntoIterator<Item = Result<T, E>>,
  S: serde::Serializer<Ok = ()>,
  T: serde::Serialize,
{
  use serde::ser::SerializeSeq;

  fn iterator_len_hint<I>(iter: &I) -> Option<usize>
  where
    I: Iterator,
  {
    match iter.size_hint() {
      (lo, Some(hi)) if lo == hi => Some(lo),
      _ => None,
    }
  }

  let iter = into_iter.into_iter();
  let mut sq = ser.serialize_seq(iterator_len_hint(&iter))?;
  for elem in iter {
    sq.serialize_element(&elem?)?;
  }
  sq.end()?;
  Ok(())
}

/// Sleeps for the specified amount of time.
#[allow(clippy::unused_async, reason = "depends on the selected set of features")]
#[inline]
pub async fn sleep(duration: Duration) -> crate::Result<()> {
  #[cfg(feature = "tokio")]
  {
    tokio::time::sleep(duration).await;
    Ok(())
  }
  #[cfg(not(feature = "tokio"))]
  {
    let now = GenericTime::now();
    core::future::poll_fn(|cx| {
      if now.elapsed()? >= duration {
        return core::task::Poll::Ready(Ok(()));
      }
      cx.waker().wake_by_ref();
      core::task::Poll::Pending
    })
    .await
  }
}

/// A tracing register with optioned parameters.
#[cfg(feature = "_tracing-tree")]
pub fn tracing_tree_init() -> Result<(), tracing_subscriber::util::TryInitError> {
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

#[expect(clippy::as_conversions, reason = "`match` correctly handles conversions")]
#[expect(clippy::cast_possible_truncation, reason = "`match` correctly handles truncations")]
#[inline]
pub(crate) fn char_slice(buffer: &mut [u8; 4], ch: char) -> &[u8] {
  #[inline]
  const fn shift(number: u32, len: u8) -> u8 {
    (number >> len) as u8
  }

  const BYTES2: u8 = 0b1100_0000;
  const BYTES3: u8 = 0b1110_0000;
  const BYTES4: u8 = 0b1111_0000;
  const CONTINUATION: u8 = 0b1000_0000;

  const MASK3: u8 = 0b0000_0111;
  const MASK4: u8 = 0b0000_1111;
  const MASK5: u8 = 0b0001_1111;
  const MASK6: u8 = 0b0011_1111;

  let number = u32::from(ch);
  match number {
    0..=127 => {
      buffer[0] = shift(number, 0);
      &buffer[0..1]
    }
    128..=2047 => {
      buffer[0] = shift(number, 6) & MASK5 | BYTES2;
      buffer[1] = shift(number, 0) & MASK6 | CONTINUATION;
      &buffer[0..2]
    }
    2048..=65535 => {
      buffer[0] = shift(number, 12) & MASK4 | BYTES3;
      buffer[1] = shift(number, 6) & MASK6 | CONTINUATION;
      buffer[2] = shift(number, 0) & MASK6 | CONTINUATION;
      &buffer[0..3]
    }
    _ => {
      buffer[0] = shift(number, 18) & MASK3 | BYTES4;
      buffer[1] = shift(number, 12) & MASK6 | CONTINUATION;
      buffer[2] = shift(number, 6) & MASK6 | CONTINUATION;
      buffer[3] = shift(number, 0) & MASK6 | CONTINUATION;
      buffer
    }
  }
}

#[inline]
pub(crate) fn _interspace<E, T, W>(
  write: &mut W,
  mut iter: impl Iterator<Item = T>,
  mut cb: impl for<'args> FnMut(&mut W, T) -> Result<(), E>,
  mut interspace: impl FnMut(&mut W) -> Result<(), E>,
) -> Result<(), E>
where
  E: From<crate::Error>,
  W: Write,
{
  if let Some(elem) = iter.next() {
    cb(write, elem)?;
  }
  for elem in iter {
    interspace(write)?;
    cb(write, elem)?;
  }
  Ok(())
}

#[cfg(feature = "std")]
pub(crate) fn _number_or_available_parallelism(n: Option<usize>) -> crate::Result<usize> {
  Ok(if let Some(elem) = n { elem } else { usize::from(std::thread::available_parallelism()?) })
}

#[cfg(feature = "ahash")]
pub(crate) fn _random_state(mut rng: impl Rng) -> ahash::RandomState {
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

pub(crate) async fn _read_until<const LEN: usize, SR>(
  buffer: &mut [u8],
  read: &mut usize,
  start: usize,
  stream_reader: &mut SR,
) -> crate::Result<[u8; LEN]>
where
  [u8; LEN]: Default,
  SR: StreamReader,
{
  let until = start.wrapping_add(LEN);
  for _ in 0..LEN {
    let has_enough_data = *read >= until;
    if has_enough_data {
      break;
    }
    let actual_buffer = buffer.get_mut(*read..).unwrap_or_default();
    let local_read = stream_reader.read(actual_buffer).await?;
    if local_read == 0 {
      return Err(crate::Error::UnexpectedStreamReadEOF);
    }
    *read = read.wrapping_add(local_read);
  }
  Ok(buffer.get(start..until).and_then(|el| el.try_into().ok()).unwrap_or_else(_unlikely_dflt))
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
  panic!("Entered in a branch that should be impossible, which is likely a programming error");
}

pub(crate) fn _usize_range_from_u32_range(range: Range<u32>) -> Range<usize> {
  *Usize::from(range.start)..*Usize::from(range.end)
}
