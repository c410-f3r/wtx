use crate::{
  asn1::{Asn1DecodeWrapperAux, Asn1EncodeWrapperAux, BitString},
  codec::{Decode, DecodeWrapper, Encode, EncodeWrapper, GenericCodec},
  x509::X509Error,
};

/// Defines the purpose (e.g., encipherment, signature, certificate signing) of the key contained
/// in the certificate.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KeyUsage {
  bytes: (u8, u8),
}

impl KeyUsage {
  /// Removes the last 7 bits of the second byte.
  #[inline]
  pub const fn new(bytes: (u8, u8)) -> Self {
    let first = bytes.0;
    let second = bytes.1 & 0b1000_0000;
    Self { bytes: (first, second) }
  }

  /// Raw bytes
  #[inline]
  pub const fn bytes(&self) -> (u8, u8) {
    self.bytes
  }

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
    self.set_bit0(0b1000_0000, value);
  }

  /// Also known as `contentCommitment`.
  ///
  /// Sets the `nonRepudiation` bit.
  #[inline]
  pub fn set_non_repudiation(&mut self, value: bool) {
    self.set_bit0(0b0100_0000, value);
  }

  /// Sets the `keyEncipherment` bit.
  #[inline]
  pub fn set_key_encipherment(&mut self, value: bool) {
    self.set_bit0(0b0010_0000, value);
  }

  /// Sets the `dataEncipherment` bit.
  #[inline]
  pub fn set_data_encipherment(&mut self, value: bool) {
    self.set_bit0(0b0001_0000, value);
  }

  /// Sets the `keyAgreement` bit.
  #[inline]
  pub fn set_key_agreement(&mut self, value: bool) {
    self.set_bit0(0b0000_1000, value);
  }

  /// Sets the `keyCertSign` bit.
  #[inline]
  pub fn set_key_cert_sign(&mut self, value: bool) {
    self.set_bit0(0b0000_0100, value);
  }

  /// Sets the `cRLSign` bit.
  #[inline]
  pub fn set_crl_sign(&mut self, value: bool) {
    self.set_bit0(0b0000_0010, value);
  }

  /// Sets the `encipherOnly` bit.
  #[inline]
  pub fn set_encipher_only(&mut self, value: bool) {
    self.set_bit0(0b0000_0001, value);
  }

  /// Sets the `decipherOnly` bit.
  #[inline]
  pub fn set_decipher_only(&mut self, value: bool) {
    if value {
      self.bytes.1 = 0b1000_0000;
    } else {
      self.bytes.1 = 0;
    }
  }

  #[inline]
  fn set_bit0(&mut self, mask: u8, value: bool) {
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
    Self { bytes: (0, 0) }
  }
}

impl<'de> Decode<'de, GenericCodec<Asn1DecodeWrapperAux, ()>> for KeyUsage {
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'de, Asn1DecodeWrapperAux>) -> crate::Result<Self> {
    let bit_string = BitString::<&[u8]>::decode(dw)?;
    let bytes = match bit_string.bytes() {
      [] => (0, 0),
      [b0] => (*b0, 0),
      [b0, b1] => (*b0, *b1),
      _ => return Err(X509Error::InvalidExtensionKeyUsage.into()),
    };
    Ok(Self { bytes })
  }
}

impl Encode<GenericCodec<(), Asn1EncodeWrapperAux>> for KeyUsage {
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_, Asn1EncodeWrapperAux>) -> crate::Result<()> {
    let (slice, unused_bits) = if self.bytes.1 != 0 {
      (
        &[self.bytes.0, self.bytes.1][..],
        self.bytes.1.trailing_zeros().try_into().unwrap_or_default(),
      )
    } else if self.bytes.0 != 0 {
      (&[self.bytes.0][..], self.bytes.0.trailing_zeros().try_into().unwrap_or_default())
    } else {
      (&[][..], 0)
    };
    // SAFETY: `unused_bits` is dynamically guaranteed to be between 0 and 7.
    unsafe {
      BitString::new_unchecked(slice, unused_bits).encode(ew)?;
    }
    Ok(())
  }
}
