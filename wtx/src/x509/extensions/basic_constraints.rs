use crate::{
  asn1::{
    Asn1DecodeWrapper, Asn1EncodeWrapper, BOOLEAN_TAG, Boolean, INTEGER_TAG, Len, Opt,
    SEQUENCE_TAG, U32, asn1_writer, decode_asn1_tlv,
  },
  codec::{Decode, Encode, GenericCodec, GenericDecodeWrapper, GenericEncodeWrapper},
  x509::X509Error,
};

/// Identifies whether the subject of the certificate is a CA and the maximum depth of valid
/// certification paths that include this certificate.
#[derive(Debug, PartialEq)]
pub struct BasicConstraints {
  ca: bool,
  path_len_constraint: Option<U32>,
}

impl BasicConstraints {
  /// Checks the relationship between `ca` and `path_len_constraint`.
  pub fn new(ca: bool, path_len_constraint: Option<U32>) -> crate::Result<Self> {
    if path_len_constraint.is_some() && !ca {
      return Err(X509Error::InvalidExtensionBasicConstraint.into());
    }
    Ok(Self { ca, path_len_constraint })
  }

  /// Whether the certified subject may act as a CA.
  pub fn ca(&self) -> bool {
    self.ca
  }

  /// Maximum number of non-self-issued intermediate CA certificates that may follow this
  /// certificate in a valid certification path.
  pub fn path_len_constraint(&self) -> Option<U32> {
    self.path_len_constraint.clone()
  }
}

impl<'de> Decode<'de, GenericCodec<Asn1DecodeWrapper, ()>> for BasicConstraints {
  #[inline]
  fn decode(dw: &mut GenericDecodeWrapper<'de, Asn1DecodeWrapper>) -> crate::Result<Self> {
    let (SEQUENCE_TAG, _, value, rest) = decode_asn1_tlv(dw.bytes)? else {
      return Err(X509Error::InvalidExtensionBasicConstraint.into());
    };
    dw.bytes = value;
    let ca = Opt::decode(dw, BOOLEAN_TAG)?.0.unwrap_or(Boolean(false)).0;
    let path_len_constraint: Option<U32> = Opt::decode(dw, INTEGER_TAG)?.0;
    if path_len_constraint.is_some() && !ca {
      return Err(X509Error::InvalidExtensionBasicConstraint.into());
    }
    dw.bytes = rest;
    Ok(Self { ca, path_len_constraint })
  }
}

impl Encode<GenericCodec<(), Asn1EncodeWrapper>> for BasicConstraints {
  #[inline]
  fn encode(&self, ew: &mut GenericEncodeWrapper<'_, Asn1EncodeWrapper>) -> crate::Result<()> {
    asn1_writer(ew, Len::MAX_ONE_BYTE, SEQUENCE_TAG, |local_ew| {
      if self.ca {
        Boolean(self.ca).encode(local_ew)?;
      }
      Opt(&self.path_len_constraint).encode(local_ew, INTEGER_TAG)?;
      Ok(())
    })
  }
}
