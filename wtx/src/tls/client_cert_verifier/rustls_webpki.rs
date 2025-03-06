use crate::{
  misc::ArrayVector,
  tls::{ClientCertVerifier, SignatureScheme},
};
use core::time::Duration;
use rustls_pki_types::{CertificateDer, UnixTime};
use webpki::{
  EndEntityCert, ExpirationPolicy, KeyUsage, RevocationCheckDepth, UnknownStatusPolicy,
};

pub struct WebpkiCertVerifier<'any> {
  crls: &'any [&'any [u8]],
}

impl<'any> ClientCertVerifier for WebpkiCertVerifier<'any> {
  #[inline]
  fn verify_client_end_cert(
    &self,
    end_cert: &[u8],
    intermediates: &[&[u8]; 4],
    now: Duration,
  ) -> crate::Result<()> {
    let certificate_der = CertificateDer::from(end_cert);
    let eec = EndEntityCert::try_from(&certificate_der)?;
    let mut intermediates_array = ArrayVector::<_, 4>::new();
    for intermediate in intermediates {
      if intermediate.is_empty() {
        break;
      }
      intermediates_array.push(CertificateDer::from(*intermediate))?;
    }
    let crls_array = ArrayVector::<_, 4>::new();
    let revocation = if self.crls.is_empty() {
      None
    } else {
      Some(
        #[expect(clippy::unwrap_used, reason = "the error variant is unbelievably private")]
        webpki::RevocationOptionsBuilder::new(crls_array.as_slice())
          .unwrap()
          .with_depth(RevocationCheckDepth::Chain)
          .with_status_policy(UnknownStatusPolicy::Deny)
          .with_expiration_policy(ExpirationPolicy::Ignore)
          .build(),
      )
    };
    let _ = eec.verify_for_usage(
      &[][..],
      &[][..],
      intermediates_array.as_slice(),
      UnixTime::since_unix_epoch(now),
      KeyUsage::client_auth(),
      revocation,
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
    todo!()
  }
}
