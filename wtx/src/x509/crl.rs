use crate::{
  asn1::{
    Asn1DecodeWrapperAux, Asn1EncodeWrapperAux, BitString, Len, SEQUENCE_TAG, asn1_writer,
    decode_asn1_tlv, parse_der_from_pem_range,
  },
  codec::{Decode, DecodeWrapper, Encode, EncodeWrapper, GenericCodec},
  collections::TryExtend,
  misc::{Lease, LeaseMut, Pem},
  x509::{AlgorithmIdentifier, TbsCertList, X509Error},
};

/// A digitally signed, time-stamped list published by a root CA containing revoked digital
/// certificates.
#[derive(Debug, PartialEq)]
pub struct Crl<B>
where
  B: Lease<[u8]>,
{
  /// See [`TbsCertList`].
  pub tbs_cert_list: TbsCertList<B>,
  /// See [`AlgorithmIdentifier`].
  pub signature_algorithm: AlgorithmIdentifier<B>,
  /// Digital signature computed upon the ASN.1 DER encoded [`TbsCertList`].
  pub signature_value: BitString<B>,
}

impl<B> Crl<B>
where
  B: Lease<[u8]>,
{
  /// From PEM data
  #[inline]
  pub fn from_pem<'this, BUF>(
    buffer: &'this mut BUF,
    bytes: &[u8],
  ) -> crate::Result<(Self, &'this [u8], Asn1DecodeWrapperAux)>
  where
    B: TryFrom<&'this [u8]>,
    BUF: LeaseMut<[u8]> + TryExtend<(u8, usize)>,
    <B as TryFrom<&'this [u8]>>::Error: Into<crate::Error>,
  {
    let pem = Pem::decode(&mut DecodeWrapper::new(bytes, &mut *buffer))?;
    let slice: &'this [u8] = <BUF as Lease<[u8]>>::lease(buffer);
    parse_der_from_pem_range(slice, &pem)
  }
}

impl<'de, B> Decode<'de, GenericCodec<Asn1DecodeWrapperAux, ()>> for Crl<B>
where
  B: Lease<[u8]> + TryFrom<&'de [u8]>,
  B::Error: Into<crate::Error>,
{
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'de, Asn1DecodeWrapperAux>) -> crate::Result<Self> {
    let (SEQUENCE_TAG, _, value, rest) = decode_asn1_tlv(dw.bytes)? else {
      return Err(X509Error::InvalidCrl.into());
    };
    dw.bytes = value;
    let tbs_cert_list = TbsCertList::decode(dw)?;
    let signature_algorithm = AlgorithmIdentifier::decode(dw)?;
    let signature_value = BitString::decode(dw)?;
    dw.bytes = rest;
    Ok(Self { tbs_cert_list, signature_algorithm, signature_value })
  }
}

impl<B> Encode<GenericCodec<(), Asn1EncodeWrapperAux>> for Crl<B>
where
  B: Lease<[u8]>,
{
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_, Asn1EncodeWrapperAux>) -> crate::Result<()> {
    asn1_writer(ew, Len::MAX_THREE_BYTES, SEQUENCE_TAG, |local_ew| {
      self.tbs_cert_list.encode(local_ew)?;
      self.signature_algorithm.encode(local_ew)?;
      self.signature_value.encode(local_ew)?;
      Ok(())
    })
  }
}
