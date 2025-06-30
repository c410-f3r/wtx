//! Decode/Encode
//!
//! Groups different dependencies that transform different types of data.

#[macro_use]
mod macros;

mod csv;
mod de_controller;
mod dec_error;
mod decode;
mod encode;
pub mod format;
mod from_radix_10;
mod hex;
mod num_array;
mod percent_encoding;
pub mod protocol;

pub use csv::Csv;
pub use de_controller::DEController;
pub use dec_error::DecError;
pub use decode::{Decode, DecodeSeq};
pub use encode::Encode;
pub use from_radix_10::{FromRadix10, FromRadix10Error};
pub use hex::{HexDisplay, HexError, decode_hex_to_slice};
pub use num_array::{
  I16String, U8String, U16String, U32String, U64String, i16_string, u8_string, u16_string,
  u32_string, u64_string,
};
pub use percent_encoding::{AsciiSet, PercentDecode, PercentEncode};

/// Identifier used to track the number of issued requests.
pub type Id = usize;

/// Hashes a password using the `argon2` algorithm.
#[cfg(feature = "argon2")]
pub fn argon2_pwd<const N: usize>(
  blocks: &mut crate::collection::Vector<argon2::Block>,
  pwd: &[u8],
  salt: &[u8],
) -> crate::Result<[u8; N]> {
  use crate::collection::{ExpansionTy, IndexedStorageMut};
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
