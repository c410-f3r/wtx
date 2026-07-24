use crate::{
  asn1::{
    Any, Asn1DecodeWrapperAux, Asn1EncodeWrapperAux, Len, OID_PKCS1_RSASSAPSS, Oid, SEQUENCE_TAG,
    asn1_writer, decode_asn1_tlv,
  },
  codec::{Decode, DecodeWrapper, Encode, EncodeWrapper, GenericCodec},
  misc::Lease,
  x509::{RsassaPssParams, X509Error},
};

/// The algorithm identifier is used to identify a cryptographic algorithm.
#[derive(Clone, Default, Debug, PartialEq)]
pub struct AlgorithmIdentifier<B> {
  /// The OID that uniquely identifies the algorithm.
  pub algorithm: Oid,
  /// Optional DER-encoded algorithm parameters (may be NULL or absent).
  pub parameters: Option<Any<B>>,
}

impl<B> AlgorithmIdentifier<B> {
  /// Shortcut
  #[inline]
  pub const fn from_algorithm(algorithm: Oid) -> Self {
    Self { algorithm, parameters: None }
  }

  /// Shortcut
  #[inline]
  pub const fn new(algorithm: Oid, parameters: Option<Any<B>>) -> Self {
    Self { algorithm, parameters }
  }

  /// Additional algorithm metadata
  #[inline]
  pub fn params_oid(&self) -> Option<Oid>
  where
    B: Lease<[u8]>,
  {
    let bytes = self.parameters.as_ref()?.bytes();
    let mut dw = DecodeWrapper::new(bytes.lease(), Asn1DecodeWrapperAux::default());
    if self.algorithm == OID_PKCS1_RSASSAPSS {
      Some(RsassaPssParams::<&[u8]>::decode(&mut dw).ok()?.hash_algorithm?.algorithm)
    } else {
      Oid::decode(&mut dw).ok()
    }
  }
}

impl<'de, B> Decode<'de, GenericCodec<Asn1DecodeWrapperAux, ()>> for AlgorithmIdentifier<B>
where
  B: Lease<[u8]> + TryFrom<&'de [u8]>,
  B::Error: Into<crate::Error>,
{
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'de, Asn1DecodeWrapperAux>) -> crate::Result<Self> {
    let (SEQUENCE_TAG, _, value, rest) = decode_asn1_tlv(dw.bytes)? else {
      return Err(X509Error::InvalidAlgorithmIdentifier.into());
    };
    dw.bytes = value;
    let algorithm = Oid::decode(dw)?;
    let parameters = if dw.bytes.is_empty() { None } else { Some(Any::decode(dw)?) };
    dw.bytes = rest;
    Ok(Self { algorithm, parameters })
  }
}

impl<B> Encode<GenericCodec<(), Asn1EncodeWrapperAux>> for AlgorithmIdentifier<B>
where
  B: Lease<[u8]>,
{
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_, Asn1EncodeWrapperAux>) -> crate::Result<()> {
    asn1_writer(ew, Len::MAX_ONE_BYTE, SEQUENCE_TAG, |local_ew| {
      self.algorithm.encode(local_ew)?;
      if let Some(params) = &self.parameters {
        params.encode(local_ew)?;
      }
      Ok(())
    })
  }
}

#[cfg(test)]
mod tests {
  use crate::{
    asn1::{
      Any, Asn1DecodeWrapperAux, Asn1EncodeWrapperAux, Len, OID_NIST_HASH_SHA256,
      OID_PKCS1_RSASSAPSS,
    },
    codec::{Decode, DecodeWrapper, Encode, EncodeWrapper},
    collections::Vector,
    x509::AlgorithmIdentifier,
  };

  #[test]
  fn pss_params() {
    let ai = AlgorithmIdentifier {
      algorithm: OID_PKCS1_RSASSAPSS,
      parameters: Some(Any::new(
        &[
          48, 52, 160, 15, 48, 13, 6, 9, 96, 134, 72, 1, 101, 3, 4, 2, 1, 5, 0, 161, 28, 48, 26, 6,
          9, 42, 134, 72, 134, 247, 13, 1, 1, 8, 48, 13, 6, 9, 96, 134, 72, 1, 101, 3, 4, 2, 1, 5,
          0, 162, 3, 2, 1, 32,
        ][..],
        48,
        Len::from_u8(52),
      )),
    };
    assert_eq!(ai.params_oid(), Some(OID_NIST_HASH_SHA256));

    let mut encoded = Vector::new();
    ai.encode(&mut EncodeWrapper::new(&mut encoded, Asn1EncodeWrapperAux::default())).unwrap();
    assert_eq!(
      AlgorithmIdentifier::<&[u8]>::decode(&mut DecodeWrapper::new(
        &encoded,
        Asn1DecodeWrapperAux::default()
      ))
      .unwrap(),
      ai
    );
  }
}
