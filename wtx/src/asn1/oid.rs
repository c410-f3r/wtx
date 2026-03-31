use crate::{
  asn1::{Asn1Error, Len, OID_TAG, decode_asn1_tlv},
  codec::{
    Decode, Encode, FromRadix10, GenericCodec, GenericDecodeWrapper, GenericEncodeWrapper,
    u32_string,
  },
  collection::{ArrayString, ArrayStringU8, ArrayVectorU8},
  misc::bytes_split_once1,
};
use core::ops::Deref;

/// Object identifier
///
/// Unique, dot-separated numerical string.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Oid(ArrayStringU8<31>);

impl Oid {
  /// New instance where `str` must be a valid `ITU-T` sequence. For example, zero padding like in
  /// `1.01.2` is invalid.
  #[inline]
  pub const fn from_str_opt(str: &str) -> Option<Self> {
    if !is_valid(str.as_bytes()) {
      return None;
    }
    let inner = match ArrayStringU8::from_str_u8_opt(str) {
      Some(elem) => elem,
      None => return None,
    };
    Some(Self(inner))
  }
}

impl<'de> Decode<'de, GenericCodec<Option<u8>, ()>> for Oid {
  #[inline]
  fn decode(dw: &mut GenericDecodeWrapper<'de, Option<u8>>) -> crate::Result<Self> {
    let (OID_TAG, _, value, rest) = decode_asn1_tlv(dw.bytes)? else {
      return Err(Asn1Error::InvalidBase128ObjectIdentifier.into());
    };
    dw.bytes = rest;
    let (first_combined, remaining) = decode_base128_one(value)?;
    let (c1, c2) = if first_combined < 40 {
      (0u32, first_combined)
    } else if first_combined < 80 {
      (1u32, first_combined - 40)
    } else {
      (2u32, first_combined - 80)
    };
    let mut buffer = ArrayStringU8::<31>::new();
    let _ = buffer.push_strs([&u32_string(c1), ".", &u32_string(c2)])?;
    decode_base128(&mut buffer, remaining)?;
    Ok(Self(buffer))
  }
}

impl Encode<GenericCodec<(), ()>> for Oid {
  #[inline]
  fn encode(&self, ew: &mut GenericEncodeWrapper<'_, ()>) -> crate::Result<()> {
    let mut components = OidComponentIter(self.0.as_bytes());
    let (Some(c1_rslt), Some(c2_rslt)) = (components.next(), components.next()) else {
      return Err(Asn1Error::InvalidBase128ObjectIdentifier.into());
    };
    let c1 = c1_rslt?;
    let c2 = c2_rslt?;
    let mut encoded = ArrayVectorU8::<u8, 20>::new();
    let first = c1
      .checked_mul(40)
      .and_then(|elem| elem.checked_add(c2))
      .ok_or(Asn1Error::InvalidBase128ObjectIdentifier)?;
    encode_base128(&mut encoded, first)?;
    for component in components {
      encode_base128(&mut encoded, component?)?;
    }
    let _ = ew.buffer.extend_from_copyable_slices([
      &[OID_TAG][..],
      &*Len::from_usize(0, encoded.len().into())?,
      encoded.as_slice(),
    ])?;
    Ok(())
  }
}

impl Deref for Oid {
  type Target = str;

  #[inline]
  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

struct OidComponentIter<'any>(&'any [u8]);
impl<'any> Iterator for OidComponentIter<'any> {
  type Item = crate::Result<u32>;

  #[inline]
  fn next(&mut self) -> Option<Self::Item> {
    if self.0.is_empty() {
      return None;
    }
    let bytes = match bytes_split_once1(self.0, b'.') {
      Some((lhs, rhs)) => {
        self.0 = rhs;
        lhs
      }
      None => {
        let lhs = self.0;
        self.0 = &[];
        lhs
      }
    };
    Some(u32::from_radix_10(bytes))
  }
}

fn decode_base128(buffer: &mut ArrayString<u8, 31>, value: &[u8]) -> crate::Result<()> {
  let mut remaining = value;
  while !remaining.is_empty() {
    let (component, rest) = decode_base128_one(remaining)?;
    remaining = rest;
    let _ = buffer.push_strs([".", &u32_string(component)])?;
  }
  Ok(())
}

fn decode_base128_one(bytes: &[u8]) -> crate::Result<(u32, &[u8])> {
  if bytes.is_empty() {
    return Err(Asn1Error::InvalidBase128ObjectIdentifier.into());
  }
  let mut value: u32 = 0;
  for (idx, &byte) in bytes.iter().enumerate() {
    let shift = value.checked_shl(7).ok_or(Asn1Error::InvalidBase128ObjectIdentifier)?;
    value = shift | u32::from(byte & 0b0111_1111);
    if byte & 0b1000_0000 == 0 {
      return Ok((value, bytes.get(idx.wrapping_add(1)..).unwrap_or_default()));
    }
  }
  Err(Asn1Error::InvalidBase128ObjectIdentifier.into())
}

fn encode_base128(buffer: &mut ArrayVectorU8<u8, 20>, mut val: u32) -> crate::Result<()> {
  if val == 0 {
    buffer.push(0)?;
    return Ok(());
  }
  let mut local_buffer = [0u8; 5];
  let mut idx = local_buffer.len();
  while val > 0 {
    idx = idx.wrapping_sub(1);
    local_buffer[idx] = u8::try_from(val & 0b0111_1111)?;
    val >>= 7;
  }
  for local_idx in idx..local_buffer.len().wrapping_sub(1) {
    if let Some(elem) = local_buffer.get_mut(local_idx) {
      *elem |= 0x80;
    }
  }
  for elem in local_buffer.get(idx..).unwrap_or_default() {
    buffer.push(*elem)?;
  }
  Ok(())
}

#[inline]
const fn is_valid(mut bytes: &[u8]) -> bool {
  let first = {
    let [first, b'.', rest @ ..] = bytes else {
      return false;
    };
    bytes = rest;
    *first
  };
  match first {
    b'0' | b'1' => {
      match bytes {
        [b'0'] => return true,
        [b'0', b'.', rest @ ..] => {
          bytes = rest;
        }
        [b'1'..=b'9'] => return true,
        [b'1'..=b'9', b'.', rest @ ..] => {
          bytes = rest;
        }
        [b'1'..=b'3', b'0'..=b'9'] => return true,
        [b'1'..=b'3', b'0'..=b'9', b'.', rest @ ..] => {
          bytes = rest;
        }
        _ => return false,
      }
      if bytes.is_empty() {
        return false;
      }
    }
    b'2' => {
      if bytes.is_empty() {
        return false;
      }
    }
    _ => return false,
  }
  while let [first, rest @ ..] = bytes {
    bytes = rest;
    if !is_valid_node(*first, &mut bytes) {
      return false;
    }
  }
  true
}

#[inline]
const fn is_valid_node(first: u8, bytes: &mut &[u8]) -> bool {
  match first {
    b'0' => match *bytes {
      [] => return true,
      [b'.', rest @ ..] => {
        *bytes = rest;
        return !bytes.is_empty();
      }
      _ => return false,
    },
    b'1'..=b'9' => {}
    _ => return false,
  }
  while let [elem, rest @ ..] = bytes {
    *bytes = rest;
    match elem {
      b'0'..=b'9' => {}
      b'.' => return !bytes.is_empty(),
      _ => return false,
    }
  }
  true
}

#[cfg(test)]
mod tests {
  use crate::asn1::Oid;

  #[test]
  fn invalid_oids() {
    assert!(Oid::from_str_opt("").is_none());
    assert!(Oid::from_str_opt("1").is_none());
    assert!(Oid::from_str_opt("3.0").is_none());
    assert!(Oid::from_str_opt("0.40").is_none());
    assert!(Oid::from_str_opt("1.40").is_none());
    assert!(Oid::from_str_opt("0.00").is_none());
    assert!(Oid::from_str_opt("0.01").is_none());
    assert!(Oid::from_str_opt("1.2.").is_none());
    assert!(Oid::from_str_opt("2.").is_none());
    assert!(Oid::from_str_opt("2.0.").is_none());
    assert!(Oid::from_str_opt("1.2.00").is_none());
    assert!(Oid::from_str_opt("1.2.01").is_none());
  }

  #[test]
  fn valid_oids() {
    let _ = Oid::from_str_opt("0.0").unwrap();
    let _ = Oid::from_str_opt("0.39").unwrap();
    let _ = Oid::from_str_opt("1.0").unwrap();
    let _ = Oid::from_str_opt("1.0.8571.2").unwrap();
    let _ = Oid::from_str_opt("1.2.3").unwrap();
    let _ = Oid::from_str_opt("1.2.840.10045.3.1.7").unwrap();
    let _ = Oid::from_str_opt("2.0").unwrap();
    let _ = Oid::from_str_opt("2.0.3").unwrap();
    let _ = Oid::from_str_opt("2.100.3").unwrap();
    let _ = Oid::from_str_opt("2.999999999999999999.3").unwrap();
  }
}
