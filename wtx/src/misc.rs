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

mod array_chunks;
mod clear;
mod connection_state;
mod de_controller;
mod decode;
mod either;
mod encode;
mod enum_var_strings;
#[cfg(any(feature = "http2", feature = "mysql", feature = "postgres", feature = "web-socket"))]
mod filled_buffer;
mod fn_fut;
mod from_radix_10;
mod incomplete_utf8_char;
mod interspace;
mod lease;
mod lock;
mod num_array;
mod optimization;
mod percent_encoding;
mod query_writer;
mod ref_counter;
mod role;
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
pub use array_chunks::{ArrayChunks, ArrayChunksMut};
pub use clear::Clear;
pub use connection_state::ConnectionState;
use core::{any::type_name, time::Duration};
pub use de_controller::DEController;
pub use decode::{Decode, DecodeSeq};
pub use either::Either;
pub use encode::Encode;
pub use enum_var_strings::EnumVarStrings;
#[cfg(any(
  feature = "http2",
  feature = "mysql",
  feature = "postgres",
  feature = "web-socket"
))]
pub use filled_buffer::{FilledBuffer, FilledBufferVectorMut};
pub use fn_fut::*;
pub use from_radix_10::{FromRadix10, FromRadix10Error};
pub use incomplete_utf8_char::{CompletionErr, IncompleteUtf8Char};
pub use interspace::Intersperse;
pub use lease::{Lease, LeaseMut};
pub use lock::Lock;
pub use num_array::*;
pub use optimization::*;
pub use percent_encoding::{AsciiSet, PercentDecode, PercentEncode};
pub use query_writer::QueryWriter;
pub use ref_counter::RefCounter;
pub use role::Role;
pub use single_type_storage::SingleTypeStorage;
pub use suffix_writer::*;
pub use uri::{Uri, UriArrayString, UriCow, UriRef, UriString};
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

/// Useful when a request returns an optional field but the actual usage is within a
/// [`core::result::Result`] context.
#[track_caller]
#[inline]
pub fn into_rslt<T>(opt: Option<T>) -> crate::Result<T> {
  opt.ok_or(crate::Error::NoInnerValue(type_name::<T>()))
}

/// Similar to `collect_seq` of `serde` but expects a `Result`.
#[cfg(feature = "serde")]
pub fn serde_collect_seq_rslt<E, I, S, T>(ser: S, into_iter: I) -> Result<S::Ok, S::Error>
where
  E: core::fmt::Display,
  I: IntoIterator<Item = Result<T, E>>,
  S: serde::Serializer,
  T: serde::Serialize,
{
  fn conservative_size_hint_len(size_hint: (usize, Option<usize>)) -> Option<usize> {
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
}

/// A tracing register with optioned parameters.
#[cfg(feature = "_tracing-tree")]
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

#[expect(clippy::cast_possible_truncation, reason = "`match` correctly handles truncations")]
pub(crate) fn char_slice(buffer: &mut [u8; 4], ch: char) -> &[u8] {
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
