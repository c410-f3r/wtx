use crate::{
  asn1::OID_X509_COMMON_NAME,
  collection::ArrayVectorU8,
  crypto::SignatureTy,
  misc::{Lease, bytes_split_once1},
  x509::{
    CvCertificate, CvEndEntity, CvPolicy, CvTrustAnchor, GeneralName, ServerName, VerifiedPath,
    X509CvError, cv::params_oid, extensions::SubjectAlternativeName,
  },
};

impl<'any, 'bytes> CvEndEntity<'any, 'bytes> {
  /// Checks that a valid path exists when walking through the provided intermediate certificates.
  /// A valid path is constructed when it hits one of the trust anchors and the associated
  /// constraints like expirations times are satisfied.
  ///
  /// It is worth noting that this method is not cheap and the number of intermediates is a
  /// considerable factor.
  #[inline]
  pub fn validate_chain(
    &'any self,
    intermediates: &'any [CvCertificate<'any, 'bytes, false>],
    cv_policy: &'any CvPolicy<'any, 'bytes>,
    trust_anchors: &'any [CvTrustAnchor<'any, 'bytes>],
  ) -> crate::Result<VerifiedPath<'any, 'bytes>> {
    let mut verified_path = VerifiedPath::new(
      self,
      ArrayVectorU8::new(),
      trust_anchors.first().ok_or(X509CvError::HasNotTrustAnchor)?,
    );
    let mut last_err = None;
    let found = crate::x509::cv::validate_chain::<true>(
      self,
      cv_policy,
      0,
      intermediates,
      &mut last_err,
      trust_anchors,
      &mut verified_path,
    );
    if !found {
      return Err(last_err.unwrap_or(X509CvError::ChainValidationDidNotFindPath).into());
    }
    Ok(verified_path)
  }

  /// Verifies `signature` over `msg` using the public key contained in this certificate.
  #[inline]
  pub fn validate_signature(&self, msg: &[u8], signature: &[u8]) -> crate::Result<()> {
    let subject_public_key_info = &self.subject_public_key_info.lease();
    let params_oid = params_oid(subject_public_key_info);
    let signature_ty =
      SignatureTy::try_from((&subject_public_key_info.algorithm.algorithm, params_oid.as_ref()))?;
    signature_ty.validate_signature(
      subject_public_key_info.subject_public_key.bytes(),
      msg,
      signature,
    )?;
    Ok(())
  }

  /// Matches `sn` with [`SubjectAlternativeName`] if it exists, otherwise tries to match `sn`
  /// with the common name.
  #[inline]
  pub fn validate_subject_name<'sn>(
    &self,
    sn_slice: impl IntoIterator<Item = ServerName<&'sn [u8]>>,
  ) -> crate::Result<()> {
    let Some(sn) = sn_slice.into_iter().next() else {
      return Ok(());
    };
    let ip_buffer = &mut [0; 16];
    let sn_bytes = sn.bytes(ip_buffer);
    if let Some(subject_alternative_name) = &self.subject_alternative_name {
      if validate_sn(sn_bytes, &subject_alternative_name.extension)? {
        return Ok(());
      }
    } else {
      if validate_sn_from_subject(self, sn_bytes)? {
        return Ok(());
      }
    }
    Err(X509CvError::UnknownSubjectName.into())
  }
}

#[inline]
fn matches_san(lhs: &[u8], rhs: &[u8]) -> bool {
  let [b'*', b'.', rest @ ..] = lhs else {
    return lhs.eq_ignore_ascii_case(rhs);
  };
  let Some((_, rhs)) = bytes_split_once1(rhs, b'.') else {
    return false;
  };
  rest.eq_ignore_ascii_case(rhs)
}

#[inline]
fn validate_sn(
  sn: &[u8],
  subject_alternative_name: &SubjectAlternativeName<'_>,
) -> crate::Result<bool> {
  for gn in &subject_alternative_name.general_names.entries {
    match gn {
      GeneralName::DnsName(elem) if matches_san(elem, sn) => {
        return Ok(true);
      }
      GeneralName::IpAddress(elem) if *elem == sn => {
        return Ok(true);
      }
      GeneralName::Rfc822Name(elem) => {
        return Ok(*elem == sn);
      }
      _ => {}
    }
  }
  Ok(false)
}

#[inline]
fn validate_sn_from_subject(cert: &CvCertificate<'_, '_, true>, sn: &[u8]) -> crate::Result<bool> {
  for rdn in cert.subject.lease().rdn_sequence.iter() {
    for atv in rdn.entries.iter() {
      if atv.oid != OID_X509_COMMON_NAME {
        continue;
      }
      if matches_san(atv.value.data(), sn) {
        return Ok(true);
      }
    }
  }
  Ok(false)
}
