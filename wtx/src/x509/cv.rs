// Chain Validation
//
// Static methods are called when their associated entities are instantiated.

pub(crate) mod cv_certificate;
pub(crate) mod cv_crl;
pub(crate) mod cv_crl_expiration;
pub(crate) mod cv_end_entity;
pub(crate) mod cv_evaluation_depth;
pub(crate) mod cv_policy;
pub(crate) mod cv_policy_mode;
pub(crate) mod cv_revoked_certificate;
pub(crate) mod cv_trust_anchor;

use crate::{
  asn1::{Asn1DecodeWrapper, OID_PKCS1_RSASSAPSS, OID_X509_COMMON_NAME, Oid},
  codec::{Decode, DecodeWrapper},
  crypto::SignatureTy,
  misc::Lease,
  x509::{
    FlaggedExtension, GeneralName, NameVector, RsassaPssParams, SubjectPublicKeyInfo, Validity,
    VerifiedPath, X509CvError,
    cv::{
      cv_certificate::CvCertificate, cv_crl_expiration::CvCrlExpiration,
      cv_evaluation_depth::CvEvaluationDepth, cv_policy::CvPolicy, cv_trust_anchor::CvTrustAnchor,
    },
    extensions::{
      AuthorityKeyIdentifier, BasicConstraints, ExtendedKeyUsage, KeyUsage, NameConstraints,
      SubjectKeyIdentifier,
    },
  },
};
use core::mem;

// Certs without SAN or Subject are valid
#[inline]
fn check_cn_or_nc<const IS_EE: bool>(
  is_ica: bool,
  cert: &CvCertificate<'_, '_, IS_EE>,
  last_err: &mut Option<X509CvError>,
  nc_opt: Option<&NameConstraints<'_>>,
) -> bool {
  if let Some(name_constraints) = nc_opt {
    if let Some(subject_alternative_name) = &cert.subject_alternative_name {
      for gn in &subject_alternative_name.extension.general_names.entries {
        if !check_gn_against_nc::<true>(gn, is_ica, last_err, name_constraints) {
          *last_err = Some(X509CvError::DoesNotHaveMatchedConstraints);
          return false;
        }
      }
      return true;
    }
    if !IS_EE {
      return true;
    }
    for rdn in cert.subject.lease().rdn_sequence.iter() {
      for atv in rdn.entries.iter() {
        if atv.oid != OID_X509_COMMON_NAME {
          continue;
        }
        let cn_data = atv.value.data();
        if let [b'0', b'x', ..] = cn_data {
          *last_err = Some(X509CvError::IpCanNotBeHex);
          return false;
        }
        let cn_gn = GeneralName::DnsName(cn_data);
        if !check_gn_against_nc::<false>(&cn_gn, is_ica, last_err, name_constraints) {
          *last_err = Some(X509CvError::DoesNotHaveMatchedConstraints);
          return false;
        }
      }
    }
  } else {
    if !IS_EE {
      return true;
    }
    for rdn in cert.subject.lease().rdn_sequence.iter() {
      for atv in rdn.entries.iter() {
        if atv.oid != OID_X509_COMMON_NAME {
          continue;
        }
        let cn_data = atv.value.data();
        if let [b'0', b'x', ..] = cn_data {
          *last_err = Some(X509CvError::IpCanNotBeHex);
          return false;
        }
      }
    }
  }
  true
}

#[inline]
fn check_gn_against_nc<const IS_SAN: bool>(
  gn: &GeneralName<'_>,
  is_ica: bool,
  last_err: &mut Option<X509CvError>,
  name_constraints: &NameConstraints<'_>,
) -> bool {
  if let Some(excluded) = &name_constraints.excluded_subtrees {
    for subtree in excluded.iter() {
      if matches_name::<IS_SAN>(&subtree.base, is_ica, gn) {
        *last_err = Some(X509CvError::HasExcludedCerts);
        return false;
      }
    }
  }
  if let Some(permitted) = &name_constraints.permitted_subtrees {
    let mut matched = false;
    let mut same_type_found = false;
    for subtree in permitted.iter() {
      if mem::discriminant(gn) == mem::discriminant(&subtree.base) {
        same_type_found = true;
      }
      if matches_name::<IS_SAN>(&subtree.base, is_ica, gn) {
        matched = true;
        break;
      }
    }
    if same_type_found && !matched {
      return false;
    }
  }
  true
}

#[inline]
fn check_name_constraints(
  is_ica: bool,
  last_err: &mut Option<X509CvError>,
  name_constraints: &NameConstraints<'_>,
  verified_path: &VerifiedPath<'_, '_>,
) -> bool {
  if !check_cn_or_nc(is_ica, verified_path.end_entity(), last_err, Some(name_constraints)) {
    return false;
  }
  for child in verified_path.intermediates() {
    let is_self_issued = child.issuer == child.subject;
    if is_self_issued {
      continue;
    }
    if !check_cn_or_nc(is_ica, child, last_err, Some(name_constraints)) {
      return false;
    }
  }
  true
}

#[inline]
fn check_revocation<const IS_EE: bool>(
  cert: &CvCertificate<'_, '_, IS_EE>,
  cv_policy: &CvPolicy<'_, '_>,
  depth: u8,
  issuer_ku: Option<KeyUsage>,
  last_err: &mut Option<X509CvError>,
) -> bool {
  if cv_policy.evaluation_depth() == CvEvaluationDepth::EndEntity && depth > 0 {
    return true;
  }

  for crl in cv_policy.crls() {
    if *cert.issuer.lease() != *crl.issuer.lease() {
      continue;
    }

    if let Some(elem) = &crl.issuing_distribution_point {
      if elem.only_contains_attribute_certs.unwrap_or(false) {
        continue;
      }
      if !IS_EE && elem.only_contains_user_certs.unwrap_or(false) {
        continue;
      }
      if IS_EE && elem.only_contains_ca_certs.unwrap_or(false) {
        continue;
      }
    }

    if let Some(key_usage) = issuer_ku
      && !key_usage.crl_sign()
    {
      *last_err = Some(X509CvError::HasIncompatibleKeyUsage);
      return false;
    }

    if cv_policy.expiration_policy() == CvCrlExpiration::Enforce {
      let is_valid = crl
        .next_update
        .as_ref()
        .is_some_and(|next_update| *cv_policy.validation_time() < next_update.date_time());
      if !is_valid {
        *last_err = Some(X509CvError::HasExpiredCerts);
        return false;
      }
    }

    let Some(revoked_certs) = crl.revoked_certs.lease() else {
      continue;
    };

    for revoked_cert in &revoked_certs.0 {
      if revoked_cert.user_certificate.bytes() == cert.serial.bytes() {
        *last_err = Some(X509CvError::HasRevokedCerts);
        return false;
      }
    }
  }

  true
}

#[inline]
#[rustfmt::skip]
fn matches_ip_address(lhs: &[u8], rhs: &[u8]) -> bool {
  match lhs.len() {
    4 => {
      let [b0, b1, b2, b3, b4, b5, b6, b7] = rhs else {
        return false;
      };
      let addr = [b0, b1, b2, b3];
      let mask = [b4, b5, b6, b7];
      for ((a, b), c) in lhs.iter().zip(addr).zip(mask) {
        if (a & c) != (b & c) {
          return false;
        }
      }
      true
    }
    16 => {
      let [
        b0, b1, b2, b3, b4, b5, b6, b7,
        b8, b9, b10, b11, b12, b13, b14, b15,
        b16, b17, b18, b19, b20, b21, b22, b23,
        b24, b25, b26, b27, b28, b29, b30, b31,
      ] = rhs
      else {
        return false;
      };
      let addr = [b0, b1, b2, b3, b4, b5, b6, b7, b8, b9, b10, b11, b12, b13, b14, b15];
      let mask = [b16, b17, b18, b19, b20, b21, b22, b23, b24, b25, b26, b27, b28, b29, b30, b31];
      for ((a, b), c) in lhs.iter().zip(addr).zip(mask) {
        if (a & c) != (b & c) {
          return false;
        }
      }
      true
    }
    _ => false,
  }
}

#[inline]
fn matches_name<const IS_SAN: bool>(
  constraint: &GeneralName<'_>,
  is_ica: bool,
  name: &GeneralName<'_>,
) -> bool {
  match (name, constraint) {
    (GeneralName::DirectoryName(lhs), GeneralName::DirectoryName(rhs)) => lhs == rhs,
    (GeneralName::DnsName(lhs), GeneralName::DnsName(rhs)) => {
      matches_name_domain::<IS_SAN>(is_ica, lhs, rhs)
    }
    (GeneralName::Rfc822Name(lhs), GeneralName::Rfc822Name(rhs)) => matches_rfc822(lhs, rhs),
    (GeneralName::IpAddress(lhs), GeneralName::IpAddress(rhs)) => matches_ip_address(lhs, rhs),
    (GeneralName::OtherName(lhs), GeneralName::OtherName(rhs)) => lhs == rhs,
    _ => false,
  }
}

#[inline]
fn matches_name_domain<const IS_SAN: bool>(is_ica: bool, other: &[u8], domain: &[u8]) -> bool {
  #[inline]
  fn slices<'other>(other: &'other [u8], domain: &[u8]) -> Option<(&'other [u8], &'other [u8])> {
    other.len().checked_sub(domain.len()).and_then(|idx| other.split_at_checked(idx))
  }

  #[inline]
  fn manage_wildcard<const IS_SAN: bool>(is_ica: bool, other: &[u8], rest: &[u8]) -> bool {
    if !IS_SAN || is_ica {
      return false;
    }
    let Some((other_begin, other_domain)) = slices(other, rest) else {
      return false;
    };
    match other_begin {
      [] | [b'.'] | [.., 0..=45 | 47..=255] => false,
      [local_rest @ .., b'.'] => other_domain == rest && !local_rest.contains(&b'.'),
    }
  }

  if let [b'*', b'.', rest @ ..] = domain {
    manage_wildcard::<IS_SAN>(is_ica, other, rest)
  } else if let [b'*', b'.', rest @ ..] = other {
    manage_wildcard::<IS_SAN>(is_ica, other, rest)
  } else {
    let Some((other_begin, other_domain)) = slices(other, domain) else {
      return false;
    };
    if let [b'.'] | [.., 0..=45 | 47..=255] = other_begin { false } else { other_domain == domain }
  }
}

#[inline]
fn matches_rfc822(other: &[u8], domain: &[u8]) -> bool {
  let slices = other.len().checked_sub(domain.len()).and_then(|idx| other.split_at_checked(idx));
  let Some((other_rest, other_domain)) = slices else {
    return false;
  };
  match other_rest {
    [] => domain.ends_with(other_domain),
    [non_at @ .., b'@'] => {
      if non_at.is_empty() || non_at.contains(&b'@') {
        false
      } else {
        domain.ends_with(other_domain)
      }
    }
    _ => false,
  }
}

#[inline]
fn params_oid(subject_public_key_info: &SubjectPublicKeyInfo<'_>) -> Option<Oid> {
  let params = subject_public_key_info.algorithm.parameters.as_ref()?;
  let mut dw = DecodeWrapper::new(params.data(), Asn1DecodeWrapper::default());
  if subject_public_key_info.algorithm.algorithm == OID_PKCS1_RSASSAPSS {
    Some(RsassaPssParams::decode(&mut dw).ok()?.hash_algorithm?.algorithm)
  } else {
    Oid::decode(&mut dw).ok()
  }
}

#[inline]
fn validate_chain<'any, 'bytes, const IS_EE: bool>(
  cert: &'any CvCertificate<'any, 'bytes, IS_EE>,
  cv_policy: &CvPolicy<'any, 'bytes>,
  depth: u8,
  intermediates: &'any [CvCertificate<'any, 'bytes, false>],
  last_err: &mut Option<X509CvError>,
  trust_anchors: &'any [CvTrustAnchor<'any, 'bytes>],
  verified_path: &mut VerifiedPath<'any, 'bytes>,
) -> bool {
  if IS_EE {
    if !check_cn_or_nc(false, cert, last_err, None) {
      return false;
    }

    // A `validate_ee_dyn` function is impossible at the current time.
    if let Err(err) = validate_eku::<IS_EE>(&cert.extended_key_usage, cv_policy) {
      *last_err = Some(err);
      return false;
    }
  }

  if let CvEvaluationDepth::Chain(max) = cv_policy.evaluation_depth()
    && depth > max
  {
    *last_err = Some(X509CvError::ExceedDepth);
    return false;
  }

  if !validate_ica_dyn(
    &cert.authority_key_identifier,
    cv_policy,
    cert.is_self_signed,
    last_err,
    &cert.subject_key_identifier,
    &cert.validity,
  ) {
    return false;
  }

  if cert.issuer != cert.subject && cert.authority_key_identifier.is_none() {
    *last_err = Some(X509CvError::HasIncompatibleSignature);
    return false;
  }

  if cert.subject.lease().rdn_sequence.is_empty()
    && !cert.subject_alternative_name.as_ref().is_some_and(|el| el.critical)
  {
    *last_err = Some(X509CvError::SanMustBeCritical);
    return false;
  }

  for trust_anchor in trust_anchors {
    if *trust_anchor.subject() != cert.issuer {
      continue;
    }

    if !validate_ica_dyn(
      trust_anchor.authority_key_identifier(),
      cv_policy,
      trust_anchor.is_self_signed(),
      last_err,
      trust_anchor.subject_key_identifier(),
      trust_anchor.validity(),
    ) {
      return false;
    }

    if cv_policy.mode().is_strict()
      && !trust_anchor.is_self_signed()
      && trust_anchor.authority_key_identifier().is_none()
    {
      continue;
    }

    if !check_revocation(cert, cv_policy, depth, trust_anchor.key_usage(), last_err) {
      continue;
    }
    let parent = trust_anchor.subject_public_key_info();
    if !validate_chain_signature(cert, last_err, parent) {
      continue;
    }

    if let Some(name_constraints) = trust_anchor.name_constraints()
      && !check_name_constraints(true, last_err, name_constraints, verified_path)
    {
      continue;
    }
    *verified_path.trust_anchor_mut() = trust_anchor;
    return true;
  }

  for intermediate in intermediates {
    if cert.issuer != intermediate.subject {
      continue;
    }
    if !validate_chain_signature(cert, last_err, intermediate.subject_public_key_info.lease()) {
      continue;
    }
    if validate_eku::<false>(&intermediate.extended_key_usage, cv_policy).is_err() {
      continue;
    }
    if let Some(basic_constraints) = &intermediate.basic_constraints {
      if !basic_constraints.extension.ca() {
        continue;
      }
      if let Some(plc) = basic_constraints.extension.path_len_constraint()
        && u32::from(depth) > plc
      {
        continue;
      }
    }
    if let Some(key_usage) = &intermediate.key_usage
      && !key_usage.key_cert_sign()
    {
      continue;
    }

    if !check_revocation(cert, cv_policy, depth, intermediate.key_usage, last_err) {
      continue;
    }

    if let Some(name_constraints) = &intermediate.name_constraints
      && !check_name_constraints(true, last_err, name_constraints, verified_path)
    {
      continue;
    }

    if verified_path.intermediates_mut().push(intermediate).is_err() {
      *last_err = Some(X509CvError::ExceedDepth);
      return false;
    }

    let next_depth = if intermediate.is_self_signed { depth } else { depth.wrapping_add(1) };

    if validate_chain::<false>(
      intermediate,
      cv_policy,
      next_depth,
      intermediates,
      last_err,
      trust_anchors,
      verified_path,
    ) {
      return true;
    }

    let _ = verified_path.intermediates_mut().pop();
  }

  false
}

#[inline]
fn validate_chain_signature<const IS_EE: bool>(
  child: &CvCertificate<'_, '_, IS_EE>,
  last_err: &mut Option<X509CvError>,
  parent: &SubjectPublicKeyInfo<'_>,
) -> bool {
  let child_sig_alg = child.signature_algorithm.lease();
  let par_params_oid = params_oid(parent);
  match SignatureTy::try_from((&child_sig_alg.algorithm, par_params_oid.as_ref())).and_then(|el| {
    el.validate_signature(parent.subject_public_key.bytes(), child.signature_msg, child.signature)
  }) {
    Ok(_) => true,
    Err(_err) => {
      *last_err = Some(X509CvError::HasIncompatibleSignature);
      false
    }
  }
}

#[inline]
fn validate_common_static(
  basic_constraints: Option<FlaggedExtension<BasicConstraints>>,
  has_unknown_critical_extension: bool,
  key_usage: Option<KeyUsage>,
  last_err: &mut Option<X509CvError>,
) -> bool {
  if has_unknown_critical_extension {
    *last_err = Some(X509CvError::CertsMustNotHaveCriticalUnknownExtensions);
    return false;
  }
  let is_ca =
    basic_constraints.as_ref().is_some_and(|basic_constraints| basic_constraints.extension.ca());
  let key_cert_sign_set = key_usage.as_ref().is_some_and(|key_usage| key_usage.key_cert_sign());
  if !is_ca && key_cert_sign_set {
    *last_err = Some(X509CvError::HasIncompatibleKeyUsage);
    return false;
  }
  true
}

#[inline]
fn validate_ee_static<'bytes>(
  last_err: &mut Option<X509CvError>,
  name_constraints: &Option<NameConstraints<'bytes>>,
) -> bool {
  if name_constraints.is_some() {
    *last_err = Some(X509CvError::InvalidNameConstraints);
    return false;
  }
  true
}

#[inline]
fn validate_eku<const IS_EE: bool>(
  cert_ek: &Option<FlaggedExtension<ExtendedKeyUsage>>,
  cv_policy: &CvPolicy<'_, '_>,
) -> Result<(), X509CvError> {
  if let Some(elem) = cert_ek {
    let FlaggedExtension { extension, critical } = elem;
    if IS_EE && *critical {
      return Err(X509CvError::EeCanNotHaveACriticalEku);
    }
    if extension.len() == 0 {
      if cv_policy.mode().is_strict() {
        return Err(X509CvError::EkuCanNotBeEmpty);
      }
    }
    if extension.any() {
      if cv_policy.mode().is_strict() {
        return Err(X509CvError::EkuCanNotBeAny);
      }
    }
    if cv_policy.extended_key_usage().server_auth() && !extension.server_auth() {
      return Err(X509CvError::EkuMismatch);
    }
    if cv_policy.extended_key_usage().client_auth() && !extension.client_auth() {
      return Err(X509CvError::EkuMismatch);
    }
    if cv_policy.extended_key_usage().code_signing() && !extension.code_signing() {
      return Err(X509CvError::EkuMismatch);
    }
    if cv_policy.extended_key_usage().email_protection() && !extension.email_protection() {
      return Err(X509CvError::EkuMismatch);
    }
    if cv_policy.extended_key_usage().time_stamping() && !extension.time_stamping() {
      return Err(X509CvError::EkuMismatch);
    }
    if cv_policy.extended_key_usage().ocsp_signing() && !extension.ocsp_signing() {
      return Err(X509CvError::EkuMismatch);
    }
  } else if IS_EE && cv_policy.mode().is_strict() {
    return Err(X509CvError::EeMustHaveEku);
  }
  Ok(())
}

#[inline]
fn validate_ica_dyn(
  aki_opt: &Option<AuthorityKeyIdentifier>,
  cv_policy: &CvPolicy<'_, '_>,
  is_self_signed: bool,
  last_err: &mut Option<X509CvError>,
  ski_opt: &Option<FlaggedExtension<SubjectKeyIdentifier>>,
  validity: &Validity,
) -> bool {
  let not_before = validity.not_before.date_time();
  let not_after = validity.not_after.date_time();
  if *cv_policy.validation_time() < not_before || *cv_policy.validation_time() > not_after {
    *last_err = Some(X509CvError::HasExpiredCerts);
    return false;
  }
  {
    // TODO(UPSTREAM): Dynamic because of <https://github.com/C2SP/x509-limbo/issues/598>
    let Some(subject_key_identifier) = ski_opt else {
      *last_err = Some(X509CvError::IcasMustHaveSki);
      return false;
    };
    if subject_key_identifier.critical {
      *last_err = Some(X509CvError::SubjectKeyIdentifierMustNotBeCritical);
      return false;
    }
    if cv_policy.mode().is_strict() && is_self_signed {
      let Some(ki) = aki_opt.as_ref().and_then(|el| el.key_identifier.as_ref()) else {
        *last_err = Some(X509CvError::RootCasMustHaveKeyIdentifiers);
        return false;
      };
      if ki.bytes() != subject_key_identifier.extension.key_identifier.bytes() {
        *last_err = Some(X509CvError::RootCasMustHaveMatchingAkiAndSki);
        return false;
      }
    }
  }
  true
}

#[inline]
fn validate_ica_static<'bytes>(
  basic_constraints: Option<FlaggedExtension<BasicConstraints>>,
  last_err: &mut Option<X509CvError>,
  subject: &NameVector<'bytes>,
) -> bool {
  if basic_constraints.is_none() || !basic_constraints.as_ref().is_some_and(|el| el.critical) {
    *last_err = Some(X509CvError::IcasMustHaveCriticalBasicConstraints);
    return false;
  }
  if subject.rdn_sequence.is_empty() {
    *last_err = Some(X509CvError::IcasMustHaveASubjectSequence);
    return false;
  }
  true
}
