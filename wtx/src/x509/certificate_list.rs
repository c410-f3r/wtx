use crate::{
  asn1::BitString,
  x509::{AlgorithmIdentifier, TbsCertList},
};

/// A digitally signed, time-stamped list published by a root CA containing revoked digital
/// certificates.
#[derive(Debug, PartialEq)]
pub struct CertificateList<'bytes> {
  /// See [`AlgorithmIdentifier`].
  pub signature_algorithm: AlgorithmIdentifier<'bytes>,
  /// Digital signature computed upon the ASN.1 DER encoded [`TbsCertList`].
  pub signature_value: BitString<&'bytes [u8]>,
  /// See [`TbsCertList`].
  pub tbs_cert_list: TbsCertList<'bytes>,
}
