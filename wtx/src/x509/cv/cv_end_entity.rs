use crate::{
  asn1::OID_X509_COMMON_NAME,
  collections::ArrayVectorU8,
  misc::{Lease, bytes_split_once1},
  x509::{
    CvCertificate, CvEndEntity, CvIntermediate, CvPolicy, CvTrustAnchor, GeneralName, ServerName,
    VerifiedPath, X509CvError, extensions::SubjectAlternativeName,
  },
};

impl<'any> CvEndEntity<&'any [u8]> {
  /// Checks that a valid path exists when walking through the provided intermediate certificates.
  /// A valid path is constructed when it hits one of the trust anchors and the associated
  /// constraints like expirations times are satisfied.
  ///
  /// It is worth noting that this method is not cheap and the number of intermediates is a
  /// considerable factor.
  #[inline]
  pub fn validate_chain<B>(
    &'any self,
    intermediates: &'any [CvIntermediate<&'any [u8]>],
    cv_policy: &'any CvPolicy<B>,
    trust_anchors: &'any [CvTrustAnchor<B>],
  ) -> crate::Result<VerifiedPath<'any, B>>
  where
    B: Lease<[u8]>,
  {
    let mut verified_path = VerifiedPath::new(
      self,
      ArrayVectorU8::new(),
      trust_anchors.first().ok_or(X509CvError::HasNotTrustAnchor)?,
    );
    let mut last_err = None;
    let found = crate::x509::cv::validate_chain::<B, true>(
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

  /// Matches `sn` with [`SubjectAlternativeName`] if it exists, otherwise tries to match `sn`
  /// with the common name.
  #[inline]
  pub fn validate_subject_name<'sn>(
    &self,
    sn_iter: impl IntoIterator<Item = ServerName<&'sn [u8]>>,
  ) -> crate::Result<()> {
    let Some(sn) = sn_iter.into_iter().next() else {
      return Ok(());
    };
    let ip_buffer = &mut [0; 16];
    let sn_bytes = sn.bytes(ip_buffer);
    if let Some(subject_alternative_name) = &self.subject_alternative_name {
      if validate_sn(sn_bytes, subject_alternative_name.extension()) {
        return Ok(());
      }
    } else if validate_sn_from_subject(self, sn_bytes) {
      return Ok(());
    }
    Err(X509CvError::UnknownSubjectName.into())
  }
}

#[inline]
fn matches_san(lhs: &[u8], rhs: &[u8]) -> bool {
  let [b'*', b'.', rest @ ..] = lhs else {
    return lhs.eq_ignore_ascii_case(rhs);
  };
  let Some((_, el)) = bytes_split_once1(rhs, b'.') else {
    return false;
  };
  rest.eq_ignore_ascii_case(el)
}

#[inline]
fn validate_sn<B>(sn: &[u8], subject_alternative_name: &SubjectAlternativeName<B>) -> bool
where
  B: Lease<[u8]>,
{
  for gn in &subject_alternative_name.general_names.entries {
    match gn {
      GeneralName::DnsName(elem) if matches_san(elem.lease(), sn) => {
        return true;
      }
      GeneralName::IpAddress(elem) if elem.lease() == sn => {
        return true;
      }
      GeneralName::Rfc822Name(elem) => {
        return elem.lease() == sn;
      }
      GeneralName::DirectoryName(_)
      | GeneralName::DnsName(_)
      | GeneralName::EdiPartyName(_)
      | GeneralName::IpAddress(_)
      | GeneralName::OtherName(_)
      | GeneralName::RegisteredId(_)
      | GeneralName::UniformResourceIdentifier(_)
      | GeneralName::X400Address(_) => {}
    }
  }
  false
}

#[inline]
fn validate_sn_from_subject<B>(cert: &CvCertificate<B, true>, sn: &[u8]) -> bool
where
  B: Lease<[u8]>,
{
  for rdn in cert.subject.rdn_sequence() {
    for atv in &rdn.entries {
      if atv.oid != OID_X509_COMMON_NAME {
        continue;
      }
      if matches_san(atv.value.data(), sn) {
        return true;
      }
    }
  }
  false
}
