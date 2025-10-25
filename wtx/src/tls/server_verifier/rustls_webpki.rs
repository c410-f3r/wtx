use crate::{
  collection::{ArrayVector, ArrayVectorU8},
  tls::{ClientVerifier, SignatureScheme, TrustAnchor},
};
use core::time::Duration;
use rustls::DigitallySignedStruct;
use rustls_pki_types::{CertificateDer, UnixTime};
use webpki::{
  EndEntityCert, ExpirationPolicy, KeyUsage, RevocationCheckDepth, UnknownStatusPolicy,
};

pub struct WebpkiCertVerifier<'any> {
  revocation_certs: &'any [&'any [u8]],
  signatures: &'any [SignatureScheme],
  trust_anchors: &'any [TrustAnchor<'any>],
}

impl<'any> ClientVerifier for WebpkiCertVerifier<'any> {
  #[inline]
  fn verify_client_end_cert(
    &self,
    end_cert: &[u8],
    intermediates: ArrayVectorU8<&[u8], 2>,
    now: Duration,
  ) -> crate::Result<()> {
    let certificate_der = CertificateDer::from(end_cert);
    let eec = EndEntityCert::try_from(&certificate_der)?;
    let mut intermediates_der = ArrayVectorU8::<_, 2>::new();
    for intermediate in intermediates {
      intermediates_der.push(CertificateDer::from(intermediate))?;
    }
    let crls_array = ArrayVectorU8::<_, 2>::new();
    let revocation_options = if self.revocation_certs.is_empty() {
      None
    } else {
      Some(
        webpki::RevocationOptionsBuilder::new(crls_array.as_slice())
          .map_err(|_err| webpki::Error::InvalidCertValidity)?
          .with_depth(RevocationCheckDepth::Chain)
          .with_expiration_policy(ExpirationPolicy::Enforce)
          .with_status_policy(UnknownStatusPolicy::Deny)
          .build(),
      )
    };
    let _ = eec.verify_for_usage(
      &[][..],
      &[][..],
      intermediates_der.as_slice(),
      UnixTime::since_unix_epoch(now),
      KeyUsage::client_auth(),
      revocation_options,
      None,
    )?;
    Ok(())
  }

  #[inline]
  fn verify_client_signature(
    &self,
    cert: &[u8],
    signer: (SignatureScheme, &[u8]),
    msg: &[u8],
  ) -> crate::Result<()> {
    let certificate_der = CertificateDer::from(cert);
    let eec = EndEntityCert::try_from(&certificate_der)?;
    //let dss = DigitallySignedStruct::new();
    //eec.verify_signature(alg, msg, dss.signature())?;
    Ok(())
  }
}
