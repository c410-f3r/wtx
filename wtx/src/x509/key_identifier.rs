use crate::{
  asn1::{Asn1DecodeWrapper, Asn1EncodeWrapper, Octetstring},
  codec::{Decode, Encode, GenericCodec, GenericDecodeWrapper, GenericEncodeWrapper},
  x509::X509Error,
};

/// The value is typically a hash of the subject's public key.
#[derive(Debug, PartialEq)]
pub enum KeyIdentifier {
  /// Four-bit type field with the value 0100 followed by the least significant 60 bits of
  /// the SHA-1 hash of the value of the BIT STRING subjectPublicKey.
  Composed([u8; 8]),
  /// SHA-1 hash of the value of the BIT STRING subjectPublicKey.
  Sha1([u8; 20]),
}

impl<'de> Decode<'de, GenericCodec<Asn1DecodeWrapper, ()>> for KeyIdentifier {
  #[inline]
  fn decode(dw: &mut GenericDecodeWrapper<'de, Asn1DecodeWrapper>) -> crate::Result<Self> {
    let bytes = *Octetstring::decode(dw)?.bytes();
    Ok(match bytes {
      [a, b, c, d, e, f, g, h] => Self::Composed([*a, *b, *c, *d, *e, *f, *g, *h]),
      [a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p, q, r, s, t] => {
        Self::Sha1([*a, *b, *c, *d, *e, *f, *g, *h, *i, *j, *k, *l, *m, *n, *o, *p, *q, *r, *s, *t])
      }
      _ => return Err(X509Error::InvalidKeyIdentifier.into()),
    })
  }
}

impl Encode<GenericCodec<(), Asn1EncodeWrapper>> for KeyIdentifier {
  #[inline]
  fn encode(&self, ew: &mut GenericEncodeWrapper<'_, Asn1EncodeWrapper>) -> crate::Result<()> {
    let slice = match self {
      KeyIdentifier::Composed(el) => el.as_slice(),
      KeyIdentifier::Sha1(el) => el.as_slice(),
    };
    Octetstring::new(slice).encode(ew)?;
    Ok(())
  }
}
