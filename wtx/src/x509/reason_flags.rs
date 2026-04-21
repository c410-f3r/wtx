use crate::{
  asn1::{Asn1DecodeWrapper, Asn1EncodeWrapper, BitString},
  codec::{Decode, DecodeWrapper, Encode, EncodeWrapper, GenericCodec},
  x509::X509Error,
};

/// A `BIT STRING` variation of `ReasonCode`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ReasonFlags {
  bytes: (u8, u8),
  unused_bits: u8,
}

impl ReasonFlags {
  /// Not used
  #[inline]
  pub const fn unused(&self) -> bool {
    self.bytes.0 & 0b1000_0000 != 0
  }

  /// Subject's private key has been compromised
  #[inline]
  pub const fn key_compromise(&self) -> bool {
    self.bytes.0 & 0b0100_0000 != 0
  }

  /// Issuing CA's private key has been compromised
  #[inline]
  pub const fn ca_compromise(&self) -> bool {
    self.bytes.0 & 0b0010_0000 != 0
  }

  /// Subject is no longer affiliated with the issuing organization
  #[inline]
  pub const fn affiliation_changed(&self) -> bool {
    self.bytes.0 & 0b0001_0000 != 0
  }

  /// Certificate has been replaced by a new one
  #[inline]
  pub const fn superseded(&self) -> bool {
    self.bytes.0 & 0b0000_1000 != 0
  }

  /// Subject has ceased operation
  #[inline]
  pub const fn cessation_of_operation(&self) -> bool {
    self.bytes.0 & 0b0000_0100 != 0
  }

  /// Certificate is temporarily on hold
  #[inline]
  pub const fn certificate_hold(&self) -> bool {
    self.bytes.0 & 0b0000_0010 != 0
  }

  /// Privileges granted to the subject have been withdrawn
  #[inline]
  pub const fn privilege_withdrawn(&self) -> bool {
    self.bytes.0 & 0b0000_0001 != 0
  }

  /// Authority attribute (AA) has been compromised
  #[inline]
  pub fn aa_compromise(&self) -> bool {
    self.bytes.1 & 0b1000_0000 != 0
  }
}

impl<'de> Decode<'de, GenericCodec<Asn1DecodeWrapper, ()>> for ReasonFlags {
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'de, Asn1DecodeWrapper>) -> crate::Result<Self> {
    let bit_string = BitString::decode(dw)?;
    let bytes = match bit_string.bytes() {
      [a] => (*a, 0),
      [a, b] => (*a, *b),
      _ => return Err(X509Error::InvalidExtensionKeyUsage.into()),
    };
    Ok(Self { bytes, unused_bits: bit_string.unused_bits() })
  }
}

impl Encode<GenericCodec<(), Asn1EncodeWrapper>> for ReasonFlags {
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_, Asn1EncodeWrapper>) -> crate::Result<()> {
    let slice =
      if self.bytes.1 == 0 { &[self.bytes.0][..] } else { &[self.bytes.0, self.bytes.1][..] };
    // SAFETY: `unused_bits` comes from a valid `BitString` instance when decoding
    unsafe {
      BitString::new_unchecked(slice, self.unused_bits).encode(ew)?;
    }
    Ok(())
  }
}
