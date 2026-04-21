use crate::{
  asn1::{Asn1DecodeWrapper, Asn1EncodeWrapper, BitString},
  codec::{Decode, DecodeWrapper, Encode, EncodeWrapper, GenericCodec},
  x509::X509Error,
};

/// Defines the purpose (e.g., encipherment, signature, certificate signing) of the key contained
/// in the certificate.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct KeyUsage {
  bytes: (u8, u8),
  unused_bits: u8,
}

impl KeyUsage {
  /// Returns `true` if the `digitalSignature` bit is set.
  #[inline]
  pub fn digital_signature(&self) -> bool {
    self.bytes.0 & 0b1000_0000 != 0
  }

  /// Returns `true` if the `nonRepudiation`/`contentCommitment` bit is set.
  #[inline]
  pub fn non_repudiation(&self) -> bool {
    self.bytes.0 & 0b0100_0000 != 0
  }

  /// Returns `true` if the `keyEncipherment` bit is set.
  #[inline]
  pub fn key_encipherment(&self) -> bool {
    self.bytes.0 & 0b0010_0000 != 0
  }

  /// Returns `true` if the `dataEncipherment` bit is set.
  #[inline]
  pub fn data_encipherment(&self) -> bool {
    self.bytes.0 & 0b0001_0000 != 0
  }

  /// Returns `true` if the `keyAgreement` bit is set.
  #[inline]
  pub fn key_agreement(&self) -> bool {
    self.bytes.0 & 0b0000_1000 != 0
  }

  /// Returns `true` if the `keyCertSign` bit is set.
  #[inline]
  pub fn key_cert_sign(&self) -> bool {
    self.bytes.0 & 0b0000_0100 != 0
  }

  /// Returns `true` if the `crlSign` bit is set.
  #[inline]
  pub fn crl_sign(&self) -> bool {
    self.bytes.0 & 0b0000_0010 != 0
  }

  /// Returns `true` if the `encipherOnly` bit is set.
  #[inline]
  pub fn encipher_only(&self) -> bool {
    self.bytes.0 & 0b0000_0001 != 0
  }

  /// Returns `true` if the `decipherOnly` bit is set.
  #[inline]
  pub fn decipher_only(&self) -> bool {
    self.bytes.1 & 0b1000_0000 != 0
  }

  /// Sets the `digitalSignature` bit.
  #[inline]
  pub fn set_digital_signature(&mut self, value: bool) {
    self.set_bit(0b1000_0000, value);
  }

  /// Also known as `contentCommitment`.
  ///
  /// Sets the `nonRepudiation` bit.
  #[inline]
  pub fn set_non_repudiation(&mut self, value: bool) {
    self.set_bit(0b0100_0000, value);
  }

  /// Sets the `keyEncipherment` bit.
  #[inline]
  pub fn set_key_encipherment(&mut self, value: bool) {
    self.set_bit(0b0010_0000, value);
  }

  /// Sets the `dataEncipherment` bit.
  #[inline]
  pub fn set_data_encipherment(&mut self, value: bool) {
    self.set_bit(0b0001_0000, value);
  }

  /// Sets the `keyAgreement` bit.
  #[inline]
  pub fn set_key_agreement(&mut self, value: bool) {
    self.set_bit(0b0000_1000, value);
  }

  /// Sets the `keyCertSign` bit.
  #[inline]
  pub fn set_key_cert_sign(&mut self, value: bool) {
    self.set_bit(0b0000_0100, value);
  }

  /// Sets the `cRLSign` bit.
  #[inline]
  pub fn set_crl_sign(&mut self, value: bool) {
    self.set_bit(0b0000_0010, value);
  }

  /// Sets the `encipherOnly` bit.
  #[inline]
  pub fn set_encipher_only(&mut self, value: bool) {
    self.set_bit(0b0000_0001, value);
  }

  /// Sets the `decipherOnly` bit.
  #[inline]
  pub fn set_decipher_only(&mut self, value: bool) {
    if value {
      self.bytes.1 = 0b1000_0000;
      self.unused_bits = 7;
    } else {
      self.bytes.0 = 0;
      self.unused_bits = 8;
    }
  }

  #[inline]
  fn set_bit(&mut self, mask: u8, value: bool) {
    if value {
      self.bytes.0 |= mask;
    } else {
      self.bytes.0 &= !mask;
    }
  }
}

impl Default for KeyUsage {
  #[inline]
  fn default() -> Self {
    Self { bytes: (0, 0), unused_bits: 8 }
  }
}

impl<'de> Decode<'de, GenericCodec<Asn1DecodeWrapper, ()>> for KeyUsage {
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

impl Encode<GenericCodec<(), Asn1EncodeWrapper>> for KeyUsage {
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_, Asn1EncodeWrapper>) -> crate::Result<()> {
    let slice =
      if self.bytes.1 == 0 { &[self.bytes.0][..] } else { &[self.bytes.0, self.bytes.1][..] };
    // SAFETY: `unused_bits` comes from a valid `BitString`
    unsafe {
      BitString::new_unchecked(slice, self.unused_bits).encode(ew)?;
    }
    Ok(())
  }
}
