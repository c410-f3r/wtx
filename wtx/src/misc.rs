//! Miscellaneous

mod array_chunks;
mod array_string;
mod array_vector;
mod blocks_deque;
mod buffer_mode;
mod bytes_fmt;
mod clear;
mod connection_state;
pub(crate) mod counter_writer;
mod decode;
mod decontroller;
mod deque;
mod either;
mod encode;
mod enum_var_strings;
pub(crate) mod facades;
mod filled_buffer;
mod fn_fut;
mod from_radix_10;
mod fun;
mod generic_time;
mod incomplete_utf8_char;
mod interspace;
mod lease;
mod lock;
pub(crate) mod mem_transfer;
mod noop_waker;
mod optimization;
pub(crate) mod partitioned_filled_buffer;
mod percent_encoding;
mod query_writer;
mod ref_counter;
mod rng;
mod role;
mod single_type_storage;
mod stream;
mod suffix_writer;
mod sync;
#[cfg(feature = "tokio-rustls")]
mod tokio_rustls;
mod tuple_impls;
mod uri;
mod usize;
mod utf8_errors;
mod vector;
mod wrapper;

#[cfg(feature = "tokio-rustls")]
pub use self::tokio_rustls::{TokioRustlsAcceptor, TokioRustlsConnector};
pub use array_chunks::{ArrayChunks, ArrayChunksMut};
pub use array_string::{ArrayString, ArrayStringError};
pub use array_vector::{ArrayVector, ArrayVectorError, IntoIter};
pub use blocks_deque::{Block, BlocksDeque, BlocksDequeBuilder, BlocksDequeError};
pub use buffer_mode::BufferMode;
pub use bytes_fmt::BytesFmt;
pub use connection_state::ConnectionState;
use core::{any::type_name, fmt::Write as _, ops::Range, time::Duration};
pub use decode::{Decode, DecodeSeq};
pub use decontroller::DEController;
pub use deque::{Deque, DequeueError};
pub use either::Either;
pub use encode::Encode;
pub use enum_var_strings::EnumVarStrings;
pub use facades::arc::Arc;
pub use filled_buffer::{FilledBuffer, FilledBufferVectorMut};
pub use fn_fut::*;
pub use from_radix_10::{FromRadix10, FromRadix10Error};
pub use fun::Fun;
pub use generic_time::GenericTime;
pub use incomplete_utf8_char::{CompletionErr, IncompleteUtf8Char};
pub use interspace::Intersperse;
pub use lease::{Lease, LeaseMut};
pub use lock::Lock;
pub use noop_waker::NOOP_WAKER;
pub use optimization::*;
pub use percent_encoding::{AsciiSet, PercentDecode, PercentEncode};
pub use query_writer::QueryWriter;
pub use ref_counter::RefCounter;
pub use rng::*;
pub use role::Role;
pub use single_type_storage::SingleTypeStorage;
pub use stream::{BytesStream, Stream, StreamReader, StreamWithTls, StreamWriter};
pub use suffix_writer::{SuffixWriter, SuffixWriterFbvm, SuffixWriterMut};
pub use sync::*;
pub use uri::{Uri, UriArrayString, UriCow, UriRef, UriString};
pub use usize::Usize;
pub use utf8_errors::{BasicUtf8Error, ExtUtf8Error, StdUtf8Error};
pub use vector::{Vector, VectorError};
pub use wrapper::Wrapper;

pub(crate) type U64Array = ArrayVector<u8, 20>;

/// Hashes a password using the `argon2` algorithm.
#[cfg(feature = "argon2")]
#[inline]
pub fn argon2_pwd<const N: usize>(
  blocks: &mut Vector<argon2::Block>,
  pwd: &[u8],
  salt: &[u8],
) -> crate::Result<[u8; N]> {
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
  blocks.expand(BufferMode::Len(params.block_count()), argon2::Block::new())?;
  let mut out = [0; N];
  Argon2::new(Algorithm::Argon2id, Version::V0x13, params)
    .hash_password_into_with_memory(pwd, salt, &mut out, blocks)?;
  Ok(out)
}

/// Useful when a request returns an optional field but the actual usage is within a
/// [`core::result::Result`] context.
#[inline]
#[track_caller]
pub fn into_rslt<T>(opt: Option<T>) -> crate::Result<T> {
  opt.ok_or(crate::Error::NoInnerValue(type_name::<T>()))
}

/// Similar to `collect_seq` of `serde` but expects a `Result`.
#[cfg(feature = "serde")]
#[inline]
pub fn serde_collect_seq_rslt<E, I, S, T>(ser: S, into_iter: I) -> Result<S::Ok, S::Error>
where
  E: core::fmt::Display,
  I: IntoIterator<Item = Result<T, E>>,
  S: serde::Serializer,
  T: serde::Serialize,
{
  use serde::ser::{Error, SerializeSeq};
  let iter = into_iter.into_iter();
  let mut sq = ser.serialize_seq(_conservative_size_hint_len(iter.size_hint()))?;
  for elem in iter {
    sq.serialize_element(&elem.map_err(S::Error::custom)?)?;
  }
  sq.end()
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
#[inline]
pub fn tracing_tree_init(
  fallback_opt: Option<&str>,
) -> Result<(), tracing_subscriber::util::TryInitError> {
  use tracing_subscriber::{
    EnvFilter, prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt,
  };
  let fallback = fallback_opt.unwrap_or("debug");
  let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(fallback));
  let mut tracing_tree = tracing_tree::HierarchicalLayer::default();
  #[cfg(feature = "std")]
  {
    tracing_tree = tracing_tree.with_writer(std::io::stderr);
  }
  tracing_tree = tracing_tree
    .with_deferred_spans(true)
    .with_indent_amount(2)
    .with_indent_lines(true)
    .with_span_retrace(true)
    .with_targets(true)
    .with_thread_ids(true)
    .with_thread_names(true)
    .with_verbose_entry(false)
    .with_verbose_exit(false);
  tracing_subscriber::Registry::default().with(env_filter).with(tracing_tree).try_init()
}

/// Transforms an `u32` into a [`ArrayString`].
#[inline]
pub fn u32_array_string(n: u32) -> ArrayString<10> {
  let mut str = ArrayString::new();
  let _rslt = str.write_fmt(format_args!("{n}"));
  str
}

/// Fills an `u64` into a [`U64Array`].
#[inline]
pub fn u64_array(data: &mut U64Array, mut value: u64) -> &str {
  let mut idx: u8 = 20;
  let mut buffer = [0u8; 20];
  for local_idx in 1..=20 {
    idx = 20u8.wrapping_sub(local_idx);
    let Some(elem) = buffer.get_mut(usize::from(idx)) else {
      break;
    };
    *elem = u8::try_from(value % 10).unwrap_or_default().wrapping_add(48);
    value /= 10;
    if value == 0 {
      break;
    }
  }
  *data = U64Array::from_array([0; 20]);
  let len = 20u16.wrapping_sub(idx.into());
  let slice = data.get_mut(..usize::from(len)).unwrap_or_default();
  slice.copy_from_slice(buffer.get(usize::from(idx)..).unwrap_or_default());
  // SAFETY: Numbers are ASCII
  unsafe { core::str::from_utf8_unchecked(slice) }
}

#[inline]
pub(crate) fn _32_bytes_seed() -> [u8; 32] {
  let seed = simple_seed();
  let mut rng = Xorshift64::from(seed);
  let [a0, b0, c0, d0, e0, f0, g0, h0, i0, j0, k0, l0, m0, n0, o0, p0] = rng.u8_16();
  let [a1, b1, c1, d1, e1, f1, g1, h1, i1, j1, k1, l1, m1, n1, o1, p1] = rng.u8_16();
  [
    a0, b0, c0, d0, e0, f0, g0, h0, i0, j0, k0, l0, m0, n0, o0, p0, a1, b1, c1, d1, e1, f1, g1, h1,
    i1, j1, k1, l1, m1, n1, o1, p1,
  ]
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
pub(crate) fn _conservative_size_hint_len(size_hint: (usize, Option<usize>)) -> Option<usize> {
  match size_hint {
    (lo, Some(hi)) if lo == hi => Some(lo),
    _ => None,
  }
}

#[cfg(feature = "std")]
#[inline]
pub(crate) fn _number_or_available_parallelism(n: Option<usize>) -> crate::Result<usize> {
  Ok(if let Some(elem) = n { elem } else { usize::from(std::thread::available_parallelism()?) })
}

#[cfg(feature = "foldhash")]
#[inline]
pub(crate) fn _random_state(mut rng: impl Rng) -> foldhash::fast::FixedState {
  let [a, b, c, d, e, f, g, h] = rng.u8_8();
  foldhash::fast::FixedState::with_seed(u64::from_ne_bytes([a, b, c, d, e, f, g, h]))
}

#[inline]
pub(crate) async fn _read_header<const BEGIN: usize, const LEN: usize, SR>(
  buffer: &mut [u8],
  read: &mut usize,
  stream_reader: &mut SR,
) -> crate::Result<[u8; LEN]>
where
  [u8; LEN]: Default,
  SR: StreamReader,
{
  loop {
    let (lhs, rhs) = buffer.split_at_mut_checked(*read).unwrap_or_default();
    if let Some(slice) = lhs.get(BEGIN..BEGIN.wrapping_add(LEN)) {
      return Ok(slice.try_into().unwrap_or_default());
    }
    let local_read = stream_reader.read(rhs).await?;
    if local_read == 0 {
      return Err(crate::Error::ClosedConnection);
    }
    *read = read.wrapping_add(local_read);
  }
}

#[inline]
pub(crate) async fn _read_payload<SR>(
  (header_len, payload_len): (usize, usize),
  pfb: &mut partitioned_filled_buffer::PartitionedFilledBuffer,
  read: &mut usize,
  stream: &mut SR,
) -> crate::Result<()>
where
  SR: StreamReader,
{
  let frame_len = header_len.wrapping_add(payload_len);
  pfb._reserve(frame_len)?;
  loop {
    if *read >= frame_len {
      break;
    }
    let local_buffer = pfb._following_rest_mut().get_mut(*read..).unwrap_or_default();
    let local_read = stream.read(local_buffer).await?;
    if local_read == 0 {
      return Err(crate::Error::ClosedConnection);
    }
    *read = read.wrapping_add(local_read);
  }
  pfb._set_indices(
    pfb._current_end_idx().wrapping_add(header_len),
    payload_len,
    read.wrapping_sub(frame_len),
  )?;
  Ok(())
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

#[inline]
pub(crate) fn _usize_range_from_u32_range(range: Range<u32>) -> Range<usize> {
  *Usize::from(range.start)..*Usize::from(range.end)
}

#[cfg(test)]
pub(crate) mod tests {
  use crate::misc::{ArrayVector, u64_array};

  #[test]
  fn has_correct_stmt_number() {
    let mut data = ArrayVector::new();
    assert_eq!(u64_array(&mut data, 0), "0");
    assert_eq!(u64_array(&mut data, 12), "12");
    assert_eq!(u64_array(&mut data, 1844674407370955161), "1844674407370955161");
    assert_eq!(u64_array(&mut data, 18446744073709551615), "18446744073709551615");
  }
}
