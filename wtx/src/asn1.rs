//! Abstract Syntax Notation One (ASN.1)
//!
//! <https://www.itu.int/rec/T-REC-X.690>

mod any;
mod asn1_error;
mod bit_string;
mod boolean;
mod generalized_time;
mod integer;
mod len;
mod octetstring;
mod oid;
mod opt;
mod sequence_buffer;
mod sequence_decode_cb;
mod sequence_encode_iter;
mod utc_time;
#[rustfmt::skip]
mod oids;
mod set;
mod u32;

use crate::{
  calendar::{Date, DateTime, Day, Hour, Month, Sixty, Time, Utc, Year},
  codec::{Decode, DecodeWrapper, EncodeWrapper, FromRadix10, GenericCodec},
  collection::{ExpansionTy, TryExtend},
  misc::Pem,
};
pub use any::Any;
pub use asn1_error::Asn1Error;
pub use bit_string::BitString;
pub use boolean::Boolean;
use core::ops::Range;
pub use generalized_time::GeneralizedTime;
pub use integer::Integer;
pub use len::Len;
pub use octetstring::Octetstring;
pub use oid::Oid;
pub use oids::*;
pub use opt::Opt;
pub use sequence_buffer::SequenceBuffer;
pub use sequence_decode_cb::SequenceDecodeCb;
pub use sequence_encode_iter::SequenceEncodeIter;
pub use set::Set;
pub use u32::U32;
pub use utc_time::UtcTime;

pub(crate) const BIT_STRING_TAG: u8 = 3;
pub(crate) const BOOLEAN_TAG: u8 = 1;
#[cfg(feature = "x509")]
pub(crate) const ENUMERATED_TAG: u8 = 10;
pub(crate) const GENERALIZED_TIME_TAG: u8 = 24;
pub(crate) const INTEGER_TAG: u8 = 2;
pub(crate) const OCTET_STRING_TAG: u8 = 4;
pub(crate) const OID_TAG: u8 = 6;
#[cfg(feature = "x509")]
pub(crate) const SEQUENCE_TAG: u8 = 48;
pub(crate) const SET_TAG: u8 = 49;
pub(crate) const UTC_TIME_TAG: u8 = 23;

/// Unsigned integer expressed as the maximum allowed type. ASN.1 elements can still contain
/// integers with smaller types.
pub type MaxUintTy = u32;
/// Length expressed as the maximum allowed type. ASN.1 elements can still contain lengths
/// with smaller types.
pub type MaxSizeTy = u16;

/// Parses an DER element from PEM contents or in other words, from base64 data delimited by labels.
///
/// The [`Pem`] structure must delimit `buffer` through its indices.
pub fn parse_der_from_pem_range<'bytes, T>(
  bytes: &'bytes [u8],
  pem: &Pem<Range<usize>, 1>,
) -> crate::Result<T>
where
  T: Decode<'bytes, GenericCodec<Asn1DecodeWrapper, ()>>,
{
  let [(_label, range)] = pem.data.as_inner()?;
  T::decode(&mut DecodeWrapper::new(
    bytes.get(range.clone()).unwrap_or_default(),
    Asn1DecodeWrapper::default(),
  ))
}

/// Generalization of [`parse_der_from_pem_range`].
pub fn parse_der_from_pem_range_many<'bytes, 'pem, I, T, U>(
  buffer: &'bytes [u8],
  instances: &mut I,
  pems: impl IntoIterator<Item = &'pem Pem<Range<usize>, 1>>,
  mut cb: impl FnMut(T) -> crate::Result<U>,
) -> crate::Result<()>
where
  I: TryExtend<[U; 1]>,
  T: Decode<'bytes, GenericCodec<Asn1DecodeWrapper, ()>>,
{
  for pem in pems {
    let [(_label, range)] = pem.data.as_inner()?;
    let bytes = buffer.get(range.clone()).unwrap_or_default();
    instances.try_extend([cb(T::decode(&mut DecodeWrapper::new(
      bytes,
      Asn1DecodeWrapper::default(),
    ))?)?])?;
  }
  Ok(())
}

#[inline]
pub(crate) fn asn1_writer(
  ew: &mut EncodeWrapper<'_, Asn1EncodeWrapper>,
  len_guess: Len,
  tag: u8,
  cb: impl FnOnce(&mut EncodeWrapper<'_, Asn1EncodeWrapper>) -> crate::Result<()>,
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
    len_bytes.copy_from_slice(&len_actual);
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
    return Err(Asn1Error::LargeData.into());
  };
  let Some((value, rest)) = after_len.split_at_checked(len.size().into()) else {
    return Err(Asn1Error::InvalidTlv.into());
  };
  Ok((*tag, len, value, rest))
}

#[inline]
fn parse_datetime(year: i16, bytes: [&u8; 10]) -> crate::Result<DateTime<Utc>> {
  let [month0, month1, day0, day1, hour0, hour1, min0, min1, sec0, sec1] = bytes;
  let date = Date::from_ymd(
    Year::from_num(year)?,
    Month::from_num(u8::from_radix_10(&[*month0, *month1])?)?,
    Day::from_num(u8::from_radix_10(&[*day0, *day1])?)?,
  )?;
  let time = Time::from_hms(
    Hour::from_num(u8::from_radix_10(&[*hour0, *hour1])?)?,
    Sixty::from_num(u8::from_radix_10(&[*min0, *min1])?)?,
    Sixty::from_num(u8::from_radix_10(&[*sec0, *sec1])?)?,
  );
  Ok(DateTime::new(date, time, Utc))
}

/// Auxiliary wrapper used for decoding
#[derive(Debug, Default)]
pub struct Asn1DecodeWrapper {
  pub(crate) tag: Option<u8>,
}

/// Auxiliary wrapper used for encoding
#[derive(Debug, Default)]
pub struct Asn1EncodeWrapper {
  pub(crate) len_guess: Len,
  pub(crate) tag: Option<u8>,
}
