use crate::{
  asn1::{Asn1DecodeWrapper, Asn1EncodeWrapper, BitString},
  codec::{Decode, Encode, GenericCodec, GenericDecodeWrapper, GenericEncodeWrapper},
  x509::X509Error,
};

/// Defines the purpose (e.g., encipherment, signature, certificate signing) of the key contained
/// in the certificate.
#[derive(Debug, PartialEq)]
pub struct KeyUsage {
  bytes: (u8, Option<u8>),
  unused_bits: u8,
}

impl KeyUsage {
  /// Returns `true` if the digitalSignature bit (0) is set.
  pub fn digital_signature(&self) -> bool {
    self.bytes.0 & 0b1000_0000 != 0
  }

  /// Returns `true` if the nonRepudiation / contentCommitment bit (1) is set.
  pub fn non_repudiation(&self) -> bool {
    self.bytes.0 & 0b0100_0000 != 0
  }

  /// Returns `true` if the keyEncipherment bit (2) is set.
  pub fn key_encipherment(&self) -> bool {
    self.bytes.0 & 0b0010_0000 != 0
  }

  /// Returns `true` if the dataEncipherment bit (3) is set.
  pub fn data_encipherment(&self) -> bool {
    self.bytes.0 & 0b0001_0000 != 0
  }

  /// Returns `true` if the keyAgreement bit (4) is set.
  pub fn key_agreement(&self) -> bool {
    self.bytes.0 & 0b0000_1000 != 0
  }

  /// Returns `true` if the keyCertSign bit (5) is set.
  pub fn key_cert_sign(&self) -> bool {
    self.bytes.0 & 0b0000_0100 != 0
  }

  /// Returns `true` if the cRLSign bit (6) is set.
  pub fn crl_sign(&self) -> bool {
    self.bytes.0 & 0b0000_0010 != 0
  }

  /// Returns `true` if the encipherOnly bit (7) is set.
  pub fn encipher_only(&self) -> bool {
    self.bytes.0 & 0b0000_0001 != 0
  }

  /// Returns `true` if the decipherOnly bit (8) is set.
  pub fn decipher_only(&self) -> bool {
    self.bytes.1.is_some_and(|el| el & 0b1000_0000 != 0)
  }
}

impl<'de> Decode<'de, GenericCodec<Asn1DecodeWrapper, ()>> for KeyUsage {
  #[inline]
  fn decode(dw: &mut GenericDecodeWrapper<'de, Asn1DecodeWrapper>) -> crate::Result<Self> {
    let bit_string = BitString::decode(dw)?;
    let bytes = match bit_string.bytes() {
      [a] => (*a, None),
      [a, b] => (*a, Some(*b)),
      _ => return Err(X509Error::InvalidExtensionKeyUsage.into()),
    };
    Ok(Self { bytes, unused_bits: bit_string.unused_bits() })
  }
}

impl Encode<GenericCodec<(), Asn1EncodeWrapper>> for KeyUsage {
  #[inline]
  fn encode(&self, ew: &mut GenericEncodeWrapper<'_, Asn1EncodeWrapper>) -> crate::Result<()> {
    let slice = match self.bytes {
      (a, None) => &[a][..],
      (a, Some(b)) => &[a, b][..],
    };
    // SAFETY: `unused_bits` comes from a valid `BitString` instance when decoding
    unsafe {
      BitString::new_unchecked(slice, self.unused_bits).encode(ew)?;
    }
    Ok(())
  }
}
