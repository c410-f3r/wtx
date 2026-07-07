//! Miscellaneous

#[cfg(feature = "http2")]
pub(crate) mod bytes_transfer;
#[cfg(any(feature = "postgres", feature = "tls"))]
pub(crate) mod counter_writer;
mod hints;
#[cfg(feature = "http2")]
pub(crate) mod span;

mod ascii;
mod connection_state;
mod default_array;
mod either;
mod enum_var_strings;
mod env_vars;
mod error_info;
mod from_vars;
pub(crate) mod int_conv;
mod interspace;
mod lease;
mod mem;
mod optimizations;
mod partial_char;
mod pem;
mod ppm;
mod role;
#[cfg(feature = "secret")]
mod secret;
mod sensitive_bytes;
mod single_type_storage;
mod tcp_params;
mod try_arithmetic;
mod tuple_impls;
mod uri;
mod usize;
mod utf8_errors;
mod wrapper;

use crate::collections::ShortStrU8;
pub use ascii::*;
pub use connection_state::ConnectionState;
use core::any::type_name;
pub use default_array::DefaultArray;
pub use either::{Either, RefOrOwned};
pub use enum_var_strings::EnumVarStrings;
pub use env_vars::EnvVars;
pub use error_info::ErrorInfo;
pub use from_vars::FromVars;
pub use hints::*;
pub use interspace::Intersperse;
pub use lease::{Lease, LeaseMut};
pub use mem::*;
pub use optimizations::*;
pub use partial_char::{CompletionErr, PartialChar, process_utf8_stream};
pub use pem::Pem;
pub use ppm::Ppm;
pub use role::{Client, Role, RoleTy, Server};
#[cfg(feature = "secret")]
pub use secret::{Secret, SecretContext};
pub use sensitive_bytes::SensitiveBytes;
pub use single_type_storage::SingleTypeStorage;
pub use tcp_params::TcpParams;
pub use try_arithmetic::*;
pub use uri::{QueryWriter, Uri, UriArrayString, UriBox, UriCow, UriRef, UriReset, UriString};
pub use usize::Usize;
pub use utf8_errors::{BasicUtf8Error, ExtUtf8Error, StdUtf8Error};
pub use wrapper::Wrapper;

/// Hashes a password using the `argon2` algorithm.
#[cfg(feature = "argon2")]
#[inline]
pub fn argon2_pwd<const N: usize>(
  blocks: &mut crate::collections::Vector<argon2::Block>,
  pwd: &[u8],
  salt: &[u8],
) -> crate::Result<[u8; N]> {
  use crate::collections::ExpansionTy;
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

/// Only works with elements that implement `Copy`.
//
// FIXME(STABLE): Constant operations
#[inline]
pub const fn const_ok<E, T>(rslt: Result<T, E>) -> Option<T>
where
  E: Copy,
  T: Copy,
{
  if let Ok(elem) = rslt { Some(elem) } else { None }
}

/// Deserializes a sequence of elements info `buffer`. Works with any deserializer of any format.
#[cfg(feature = "serde")]
#[inline]
pub fn deserialize_seq_into_buffer_with_serde<'de, D, T>(
  deserializer: D,
  buffer: &mut crate::collections::Vector<T>,
) -> crate::Result<()>
where
  D: serde::de::Deserializer<'de>,
  T: serde::Deserialize<'de>,
  crate::Error: From<D::Error>,
{
  use crate::collections::Vector;
  use core::fmt::Formatter;
  use serde::{
    Deserialize,
    de::{Error as _, SeqAccess, Visitor},
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
  opt.ok_or(crate::Error::NoInnerValue(ShortStrU8::new_truncated_u8(type_name::<T>())))
}

/// Deserializes a sequence passing each element to `cb`. Works with any deserializer of any format.
#[cfg(feature = "serde")]
#[inline]
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
  use core::{fmt::Formatter, marker::PhantomData};
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
#[inline]
pub fn serde_json_deserialize_from_slice<'any, T>(slice: &'any [u8]) -> crate::Result<T>
where
  T: serde::de::Deserialize<'any>,
{
  match serde_json::from_slice(slice) {
    Ok(elem) => Ok(elem),
    Err(err) => {
      use core::fmt::Write as _;
      let mut string = alloc::string::String::new();
      let idx = slice.len().min(1024);
      let payload = slice.get(..idx).and_then(|el| from_utf8_basic(el).ok()).unwrap_or_default();
      string.write_fmt(format_args!("Error: {err}. Payload: {payload}"))?;
      Err(crate::Error::SerdeJsonDeserialize(string.try_into()?))
    }
  }
}

/// Similar to `collect_seq` of `serde` but expects a `Result`.
#[cfg(feature = "serde")]
#[inline]
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
  use serde::ser::{Error as _, SerializeSeq as _};
  let iter = into_iter.into_iter();
  let mut sq = ser.serialize_seq(conservative_size_hint_len(iter.size_hint()))?;
  for elem in iter {
    sq.serialize_element(&elem.map_err(S::Error::custom)?)?;
  }
  sq.end()
}

/// A tracing register with optioned parameters.
#[cfg(feature = "_tracing-tree")]
#[inline]
pub fn tracing_tree_init(
  fallback_opt: Option<&str>,
) -> Result<(), tracing_subscriber::util::TryInitError> {
  use tracing_subscriber::{
    EnvFilter, prelude::__tracing_subscriber_SubscriberExt as _, util::SubscriberInitExt as _,
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
    .with_timer(crate::calendar::TracingTreeTimer {})
    .with_verbose_entry(false)
    .with_verbose_exit(false)
    .with_writer(std::io::stderr);
  tracing_subscriber::Registry::default().with(env_filter).with(tracing_tree).try_init()
}

/// A version of `std::env::var` where the name of the variable appears in errors.
#[cfg(feature = "std")]
#[inline]
pub fn var<K>(key: K) -> crate::Result<alloc::string::String>
where
  K: AsRef<std::ffi::OsStr>,
{
  match std::env::var(key.as_ref()) {
    Err(std::env::VarError::NotPresent) => Err(crate::error::Error::VarIsNotPresent(
      key.as_ref().to_os_string().into_string().unwrap_or_default().try_into()?,
    )),
    Err(std::env::VarError::NotUnicode(_)) => Err(crate::error::Error::VarIsNotUnicode(
      key.as_ref().to_os_string().into_string().unwrap_or_default().try_into()?,
    )),
    Ok(elem) => Ok(elem),
  }
}

// It is important to enforce the array length to avoid panics
pub(crate) const fn char_slice(buffer: &mut [u8; 4], ch: char) -> &mut str {
  ch.encode_utf8(buffer)
}

#[cfg(all(feature = "foldhash", any(feature = "http2", feature = "postgres")))]
pub(crate) fn random_state<RNG>(rng: &mut RNG) -> foldhash::fast::FixedState
where
  RNG: crate::rng::Rng,
{
  let [b0, b1, b2, b3, b4, b5, b6, b7] = rng.u8_8();
  foldhash::fast::FixedState::with_seed(u64::from_be_bytes([b0, b1, b2, b3, b4, b5, b6, b7]))
}

#[inline]
pub(crate) fn strip_new_line(bytes: &[u8]) -> (u8, &[u8]) {
  match bytes {
    [rest @ .., b'\r', b'\n'] => (2, rest),
    [rest @ .., b'\n'] => (1, rest),
    _ => (0, bytes),
  }
}

#[cfg(feature = "postgres")]
pub(crate) fn usize_range_from_u32_range(range: core::ops::Range<u32>) -> core::ops::Range<usize> {
  *Usize::from(range.start)..*Usize::from(range.end)
}
