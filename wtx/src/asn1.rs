//! Abstract Syntax Notation One (ASN.1)
//!
//! <https://www.itu.int/rec/T-REC-X.690>

mod any;
mod asn1_error;
mod bit_string;
mod boolean;
mod integer;
mod len;
mod octetstring;
mod oid;
mod time;
#[rustfmt::skip]
mod oids;
mod set;

use crate::{codec::GenericEncodeWrapper, collection::ExpansionTy};
pub use any::Any;
pub use asn1_error::Asn1Error;
pub use bit_string::BitString;
pub use boolean::Boolean;
pub use integer::Integer;
pub use len::Len;
pub use octetstring::Octetstring;
pub use oid::Oid;
pub use oids::*;
pub use set::Set;
pub use time::Time;

pub(crate) const BIT_STRING_TAG: u8 = 3;
pub(crate) const BOOLEAN_TAG: u8 = 1;
pub(crate) const GENERALIZED_TIME_TAG: u8 = 24;
pub(crate) const INTEGER_TAG: u8 = 2;
pub(crate) const OCTET_STRING_TAG: u8 = 4;
pub(crate) const OID_TAG: u8 = 6;
pub(crate) const SET_TAG: u8 = 49;
pub(crate) const UTC_TIME_TAG: u8 = 23;

#[cfg(feature = "x509")]
pub(crate) const EXTENSIONS_TAG: u8 = 163;
#[cfg(feature = "x509")]
pub(crate) const ISSUER_UID_TAG: u8 = 129;
#[cfg(feature = "x509")]
pub(crate) const SEQUENCE_TAG: u8 = 48;
#[cfg(feature = "x509")]
pub(crate) const SUBJECT_UID_TAG: u8 = 130;
#[cfg(feature = "x509")]
pub(crate) const VERSION_TAG: u8 = 160;

#[inline]
pub(crate) fn asn1_writer(
  ew: &mut GenericEncodeWrapper<'_, ()>,
  len_guess: Len,
  tag: u8,
  cb: impl FnOnce(&mut GenericEncodeWrapper<'_, ()>) -> crate::Result<()>,
) -> crate::Result<()> {
  let _ = ew.buffer.extend_from_copyable_slices([&[tag][..], &*len_guess])?;
  let before = ew.buffer.len();
  cb(ew)?;
  let after = ew.buffer.len();
  let len_guess_size = len_guess.len();
  let header_guess_size = 1usize.wrapping_add(len_guess_size);
  let tlv_start = before.wrapping_sub(header_guess_size);
  let encoded_len = after.wrapping_sub(before);
  let len_actual = Len::from_usize(0, encoded_len)?;
  let len_actual_size = len_actual.len();
  if len_guess_size != len_actual_size {
    let header_actual_size = 1usize.wrapping_add(len_actual_size);
    if len_actual_size > len_guess_size {
      let diff = len_actual_size.wrapping_sub(len_guess_size);
      ew.buffer.expand(ExpansionTy::Additional(diff), 0)?;
    }
    if let Some(slice) = ew.buffer.get_mut(tlv_start..) {
      slice.copy_within(
        header_guess_size..header_guess_size.wrapping_add(encoded_len),
        header_actual_size,
      );
    }
    if len_actual_size < len_guess_size {
      let diff = len_guess_size.wrapping_sub(len_actual_size);
      ew.buffer.truncate(after.wrapping_sub(diff));
    }
  }
  if let Some(slice) = ew.buffer.get_mut(tlv_start.wrapping_add(1)..)
    && let Some((len_bytes, _)) = slice.split_at_mut_checked(len_actual_size)
  {
    len_bytes.copy_from_slice(&*len_actual);
  }
  Ok(())
}

#[inline]
pub(crate) fn decode_asn1_tlv(bytes: &[u8]) -> crate::Result<(u8, Len, &[u8], &[u8])> {
  let [tag, maybe_len, maybe_after_len @ ..] = bytes else {
    return Err(Asn1Error::InvalidTlv.into());
  };
  let (len, after_len) = if *maybe_len <= 127 {
    (Len::from_u8(*maybe_len), maybe_after_len)
  } else if *maybe_len == 129 {
    let [a, rest @ ..] = maybe_after_len else {
      return Err(Asn1Error::InvalidTlv.into());
    };
    if *a < 128 {
      return Err(Asn1Error::InvalidTlv.into());
    }
    (Len::from_u8(*a), rest)
  } else if *maybe_len == 130 {
    let [a, b, rest @ ..] = maybe_after_len else {
      return Err(Asn1Error::InvalidTlv.into());
    };
    let len = u16::from_be_bytes([*a, *b]);
    if len < 256 {
      return Err(Asn1Error::InvalidTlv.into());
    }
    (Len::from_u16(len), rest)
  } else {
    return Err(Asn1Error::InvalidTlv.into());
  };
  let Some((value, rest)) = after_len.split_at_checked(len.num().into()) else {
    return Err(Asn1Error::InvalidTlv.into());
  };
  Ok((*tag, len, value, rest))
}
