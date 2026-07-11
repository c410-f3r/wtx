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
  asn1::OID_X509_COMMON_NAME,
  crypto::SignatureTy,
  misc::Lease,
  x509::{
    AttributeTypeAndValue, CvIntermediate, FlaggedExtension, GeneralName, Name,
    SubjectPublicKeyInfo, Validity, VerifiedPath, X509CvError,
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

#[inline]
pub(crate) fn validate_chain<'any, B, const IS_EE: bool>(
  cert: &'any CvCertificate<&'any [u8], IS_EE>,
  cv_policy: &CvPolicy<B>,
  depth: u8,
  intermediates: &'any [CvIntermediate<&'any [u8]>],
  last_err: &mut Option<X509CvError>,
  trust_anchors: &'any [CvTrustAnchor<B>],
  verified_path: &mut VerifiedPath<'any, B>,
) -> bool
where
  B: Lease<[u8]>,
{
  // A `validate_ee_dyn` function is impossible at the current time.
  if IS_EE {
    if !check_common_names(cert, last_err) {
      return false;
    }
    if cert.subject_alternative_name.is_none() {
      *last_err = Some(X509CvError::EeMustHaveSan);
      return false;
    }
    if let Err(err) =
      validate_eku::<IS_EE>(cv_policy.extended_key_usage(), &cert.extended_key_usage)
    {
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
    cert.has_unknown_critical_extension,
    cert.is_self_signed,
    last_err,
    &cert.subject_key_identifier,
    &cert.validity,
  ) {
    return false;
  }

  if !cert.is_self_signed && cert.authority_key_identifier.is_none() {
    *last_err = Some(X509CvError::InvalidAuthorityKeyIdentifier);
    return false;
  }

  if cert.subject.rdn_sequence().is_empty()
    && !cert.subject_alternative_name.as_ref().is_some_and(FlaggedExtension::critical)
  {
    *last_err = Some(X509CvError::SanMustBeCritical);
    return false;
  }

  for trust_anchor in trust_anchors {
    if trust_anchor.subject().lease() != cert.issuer.lease() {
      continue;
    }

    if !validate_ica_dyn(
      trust_anchor.authority_key_identifier(),
      cv_policy,
      trust_anchor.has_unknown_critical_extension(),
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
    if !validate_chain_signature(cert, last_err, trust_anchor.subject_public_key_info()) {
      continue;
    }

    if !check_verified_path(
      last_err,
      trust_anchor.name_constraints(),
      trust_anchor.subject().lease(),
      trust_anchor.subject_public_key_info().subject_public_key.bytes().lease(),
      verified_path,
    ) {
      continue;
    }
    *verified_path.trust_anchor_mut() = trust_anchor;
    return true;
  }

  for intermediate in intermediates {
    let Some(rslt) = validate_intermediate(
      cert,
      cv_policy,
      depth,
      intermediate,
      last_err,
      verified_path,
      intermediates,
      trust_anchors,
    ) else {
      continue;
    };
    return rslt;
  }

  false
}

#[inline]
fn check_common_name<B>(atv: &AttributeTypeAndValue<B>, last_err: &mut Option<X509CvError>) -> bool
where
  B: Lease<[u8]>,
{
  let cn_data = atv.value.data();
  if let [b'0', b'x', ..] = cn_data {
    *last_err = Some(X509CvError::IpCanNotBeHex);
    return false;
  }
  true
}

#[inline]
fn check_common_names<B, const IS_EE: bool>(
  cert: &CvCertificate<B, IS_EE>,
  last_err: &mut Option<X509CvError>,
) -> bool
where
  B: Lease<[u8]>,
{
  for rdn in cert.subject.rdn_sequence() {
    for atv in &rdn.entries {
      if atv.oid != OID_X509_COMMON_NAME {
        continue;
      }
      if !check_common_name(atv, last_err) {
        return false;
      }
    }
  }
  true
}

#[inline]
fn check_gn<B0, B1>(
  gn: &GeneralName<B0>,
  last_err: &mut Option<X509CvError>,
  name_constraints: &NameConstraints<B1>,
) -> bool
where
  B0: Lease<[u8]>,
  B1: Lease<[u8]>,
{
  match gn {
    GeneralName::DnsName(dns) => {
      if dns.lease().len() == 4 || dns.lease().len() == 16 {
        *last_err = Some(X509CvError::InvalidGeneralName);
        return false;
      }
    }
    GeneralName::IpAddress(ip) => {
      if ip.lease().len() != 4 && ip.lease().len() != 16 {
        *last_err = Some(X509CvError::InvalidGeneralName);
        return false;
      }
    }
    GeneralName::Rfc822Name(email) if !email.lease().contains(&b'@') => {
      *last_err = Some(X509CvError::InvalidGeneralName);
      return false;
    }
    GeneralName::DirectoryName(_)
    | GeneralName::EdiPartyName(_)
    | GeneralName::OtherName(_)
    | GeneralName::RegisteredId(_)
    | GeneralName::Rfc822Name(_)
    | GeneralName::UniformResourceIdentifier(_)
    | GeneralName::X400Address(_) => {}
  }
  if let Some(excluded) = &name_constraints.excluded_subtrees {
    for subtree in excluded {
      if matches_name_constraint::<_, _, true>(&subtree.base, gn) {
        *last_err = Some(X509CvError::HasExcludedCerts);
        return false;
      }
    }
  }
  if let Some(permitted) = &name_constraints.permitted_subtrees {
    let mut matched = false;
    let mut same_type_found = false;
    for subtree in permitted {
      if is_same_gn_type(gn, &subtree.base) {
        same_type_found = true;
      }
      if matches_name_constraint::<_, _, false>(&subtree.base, gn) {
        matched = true;
        break;
      }
    }
    if same_type_found && !matched {
      *last_err = Some(X509CvError::DoesNotHaveMatchedConstraints);
      return false;
    }
  }
  true
}

#[inline]
fn check_name_constraint<B, const IS_EE: bool>(
  cert: &CvCertificate<&[u8], IS_EE>,
  last_err: &mut Option<X509CvError>,
  name_constraints: &NameConstraints<B>,
) -> bool
where
  B: Lease<[u8]>,
{
  let mut has_san = false;
  if let Some(subject_alternative_name) = &cert.subject_alternative_name {
    has_san = true;
    for gn in &subject_alternative_name.extension().general_names.entries {
      if !check_gn(gn, last_err, name_constraints) {
        return false;
      }
    }
  }
  if !IS_EE || has_san {
    return true;
  }
  for rdn in cert.subject.rdn_sequence() {
    for atv in &rdn.entries {
      if atv.oid != OID_X509_COMMON_NAME {
        continue;
      }
      if !check_common_name(atv, last_err) {
        return false;
      }
      let cn_gn = GeneralName::DnsName(atv.value.data());
      if !check_gn(&cn_gn, last_err, name_constraints) {
        return false;
      }
    }
  }
  true
}

#[inline]
fn check_revocation<B, const IS_EE: bool>(
  cert: &CvCertificate<&[u8], IS_EE>,
  cv_policy: &CvPolicy<B>,
  depth: u8,
  issuer_ku: Option<KeyUsage>,
  last_err: &mut Option<X509CvError>,
) -> bool
where
  B: Lease<[u8]>,
{
  if cv_policy.evaluation_depth() == CvEvaluationDepth::EndEntity && depth > 0 {
    return true;
  }

  for crl in cv_policy.crls() {
    if cert.issuer.lease() != crl.issuer.lease() {
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
fn check_verified_path<B1, B2>(
  last_err: &mut Option<X509CvError>,
  name_constraints: &Option<NameConstraints<B1>>,
  subject: &[u8],
  subject_public_key: &[u8],
  verified_path: &VerifiedPath<'_, B2>,
) -> bool
where
  B1: Lease<[u8]>,
  B2: Lease<[u8]>,
{
  macro_rules! is_equal {
    ($lhs:expr, $rhs:expr) => {{
      let equal_subject = $lhs.0.subject.bytes().lease() == $lhs.1;
      let equal_spk = $rhs.0.subject_public_key_info.subject_public_key.bytes().lease() == $rhs.1;
      equal_subject && equal_spk
    }};
  }

  let ee = verified_path.end_entity();
  if is_equal!((ee, subject), (ee, subject_public_key)) {
    return false;
  }
  if let Some(elem) = name_constraints {
    if !check_name_constraint(verified_path.end_entity(), last_err, elem) {
      return false;
    }
    for child in verified_path.intermediates() {
      if child.is_self_signed {
        continue;
      }
      if !check_name_constraint(child, last_err, elem) {
        return false;
      }
      if is_equal!((child, subject), (child, subject_public_key)) {
        return false;
      }
    }
  } else {
    for child in verified_path.intermediates() {
      if is_equal!((child, subject), (child, subject_public_key)) {
        return false;
      }
    }
  }
  true
}

#[inline]
const fn is_same_gn_type<B0, B1>(lhs: &GeneralName<B0>, rhs: &GeneralName<B1>) -> bool {
  matches!(
    (lhs, rhs),
    (GeneralName::DirectoryName(_), GeneralName::DirectoryName(_))
      | (GeneralName::DnsName(_), GeneralName::DnsName(_))
      | (GeneralName::IpAddress(_), GeneralName::IpAddress(_))
      | (GeneralName::OtherName(_), GeneralName::OtherName(_))
      | (GeneralName::Rfc822Name(_), GeneralName::Rfc822Name(_))
  )
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
      for ((byte0, byte1), byte2) in lhs.iter().zip(addr).zip(mask) {
        if (byte0 & byte2) != (byte1 & byte2) {
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
      for ((byte0, byte1), byte2) in lhs.iter().zip(addr).zip(mask) {
        if (byte0 & byte2) != (byte1 & byte2) {
          return false;
        }
      }
      true
    }
    _ => false,
  }
}

#[inline]
fn matches_name_constraint<B0, B1, const IS_EXCLUDED: bool>(
  constraint: &GeneralName<B0>,
  name: &GeneralName<B1>,
) -> bool
where
  B0: Lease<[u8]>,
  B1: Lease<[u8]>,
{
  match (name, constraint) {
    (GeneralName::DirectoryName(lhs), GeneralName::DirectoryName(rhs)) => {
      lhs.lease() == rhs.lease()
    }
    (GeneralName::DnsName(lhs), GeneralName::DnsName(rhs)) => {
      matches_name_domain::<IS_EXCLUDED>(lhs.lease(), rhs.lease())
    }
    (GeneralName::Rfc822Name(lhs), GeneralName::Rfc822Name(rhs)) => {
      matches_rfc822(lhs.lease(), rhs.lease())
    }
    (GeneralName::IpAddress(lhs), GeneralName::IpAddress(rhs)) => {
      matches_ip_address(lhs.lease(), rhs.lease())
    }
    (GeneralName::OtherName(lhs), GeneralName::OtherName(rhs)) => lhs.lease() == rhs.lease(),
    _ => false,
  }
}

// `domain` always comes from an ICA and ICAs can't have wildcards.
#[inline]
fn matches_name_domain<const IS_EXCLUDED: bool>(other: &[u8], domain: &[u8]) -> bool {
  #[inline]
  fn matched(slice0: &[u8], slice1: &[u8], slice2: &[u8]) -> bool {
    if let [b'.'] | [.., 0..=45 | 47..=255] = slice0 { false } else { slice1 == slice2 }
  }

  #[inline]
  fn slices<'other>(other: &'other [u8], domain: &[u8]) -> Option<(&'other [u8], &'other [u8])> {
    let idx = other.len().checked_sub(domain.len())?;
    other.split_at_checked(idx)
  }

  if let [b'*', b'.', ..] = domain {
    false
  } else if let [b'*', b'.', rest @ ..] = other {
    if let Some((other_begin, other_domain)) = slices(rest, domain)
      && matched(other_begin, other_domain, domain)
    {
      return true;
    }
    if IS_EXCLUDED
      && let Some((domain_begin, domain_domain)) = slices(domain, rest)
      && matched(domain_begin, domain_domain, rest)
    {
      return true;
    }
    false
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
fn validate_chain_signature<B, const IS_EE: bool>(
  child: &CvCertificate<&[u8], IS_EE>,
  last_err: &mut Option<X509CvError>,
  parent: &SubjectPublicKeyInfo<B>,
) -> bool
where
  B: Lease<[u8]>,
{
  let child_sig_alg = &child.signature_algorithm;
  let par_params_oid = parent.params_oid();
  let len = parent.subject_public_key.bytes().lease().len();
  let tuple = (len, &child_sig_alg.algorithm, par_params_oid.as_ref());
  match SignatureTy::try_from(tuple).and_then(|el| {
    el.validate_signature(
      parent.subject_public_key.bytes().lease(),
      child.signature_msg.lease(),
      child.signature.lease(),
    )
  }) {
    Ok(_) => true,
    Err(_err) => {
      *last_err = Some(X509CvError::HasIncompatibleSignature);
      false
    }
  }
}

#[inline]
fn validate_ee_static<B>(
  basic_constraints: Option<FlaggedExtension<BasicConstraints>>,
  key_usage: Option<KeyUsage>,
  last_err: &mut Option<X509CvError>,
  name_constraints: &Option<NameConstraints<B>>,
) -> bool
where
  B: Lease<[u8]>,
{
  if name_constraints.is_some() {
    *last_err = Some(X509CvError::InvalidNameConstraints);
    return false;
  }
  let is_ca = basic_constraints.as_ref().is_some_and(|el| el.extension().ca());
  let key_cert_sign_set = key_usage.as_ref().is_some_and(KeyUsage::key_cert_sign);
  if !is_ca && key_cert_sign_set {
    *last_err = Some(X509CvError::HasIncompatibleKeyUsage);
    return false;
  }
  true
}

#[inline]
fn validate_eku<const IS_EE: bool>(
  eku_lhs: &ExtendedKeyUsage,
  eku_rhs: &Option<FlaggedExtension<ExtendedKeyUsage>>,
) -> Result<(), X509CvError> {
  if let Some(elem) = eku_rhs {
    if IS_EE && elem.critical() {
      return Err(X509CvError::EeCanNotHaveACriticalEku);
    }
    if elem.extension().len() == 0 {
      return Err(X509CvError::EkuCanNotBeEmpty);
    }
    if elem.extension().any() {
      return Err(X509CvError::EkuCanNotBeAny);
    }
    if eku_lhs.server_auth() && !elem.extension().server_auth() {
      return Err(X509CvError::EkuMismatch);
    }
    if eku_lhs.client_auth() && !elem.extension().client_auth() {
      return Err(X509CvError::EkuMismatch);
    }
    if eku_lhs.code_signing() && !elem.extension().code_signing() {
      return Err(X509CvError::EkuMismatch);
    }
    if eku_lhs.email_protection() && !elem.extension().email_protection() {
      return Err(X509CvError::EkuMismatch);
    }
    if eku_lhs.time_stamping() && !elem.extension().time_stamping() {
      return Err(X509CvError::EkuMismatch);
    }
    if eku_lhs.ocsp_signing() && !elem.extension().ocsp_signing() {
      return Err(X509CvError::EkuMismatch);
    }
  }
  Ok(())
}

#[inline]
fn validate_ica_dyn<B>(
  aki_opt: &Option<AuthorityKeyIdentifier>,
  cv_policy: &CvPolicy<B>,
  has_unknown_critical_extension: bool,
  is_self_signed: bool,
  last_err: &mut Option<X509CvError>,
  ski_opt: &Option<FlaggedExtension<SubjectKeyIdentifier>>,
  validity: &Validity,
) -> bool
where
  B: Lease<[u8]>,
{
  if has_unknown_critical_extension {
    *last_err = Some(X509CvError::CertsMustNotHaveCriticalUnknownExtensions);
    return false;
  }
  let not_before = validity.not_before.date_time();
  let not_after = validity.not_after.date_time();
  if *cv_policy.validation_time() < not_before || *cv_policy.validation_time() > not_after {
    *last_err = Some(X509CvError::HasExpiredCerts);
    return false;
  }
  'ski: {
    if cv_policy.mode().is_lenient() {
      break 'ski;
    }
    match (aki_opt, ski_opt) {
      (None | Some(_), None) => {
        *last_err = Some(X509CvError::IcasMustHaveSki);
        return false;
      }
      (None, Some(ski)) => {
        if ski.critical() {
          *last_err = Some(X509CvError::SubjectKeyIdentifierMustNotBeCritical);
          return false;
        }
      }
      (Some(aki), Some(ski)) => {
        if ski.critical() {
          *last_err = Some(X509CvError::SubjectKeyIdentifierMustNotBeCritical);
          return false;
        }
        let Some(ki) = &aki.key_identifier else {
          *last_err = Some(X509CvError::RootCasMustHaveKeyIdentifiers);
          return false;
        };
        if is_self_signed && ki.bytes() != ski.extension().key_identifier.bytes() {
          *last_err = Some(X509CvError::RootCasMustHaveMatchingAkiAndSki);
          return false;
        }
      }
    }
  }
  true
}

#[inline]
fn validate_ica_static<B>(
  basic_constraints: Option<FlaggedExtension<BasicConstraints>>,
  is_self_signed: bool,
  key_usage: Option<KeyUsage>,
  last_err: &mut Option<X509CvError>,
  subject: &Name<B>,
) -> bool
where
  B: Lease<[u8]>,
{
  if subject.rdn_sequence().is_empty() {
    *last_err = Some(X509CvError::IcasMustHaveASubjectSequence);
    return false;
  }
  if is_self_signed {
    let Some(bc) = basic_constraints else {
      *last_err = Some(X509CvError::IcasMustHaveBasicConstraints);
      return false;
    };
    if !bc.critical() {
      *last_err = Some(X509CvError::IcasMustHaveCriticalBasicConstraints);
      return false;
    }
    if let Some(elem) = key_usage
      && bc.extension().ca() != elem.key_cert_sign()
    {
      *last_err = Some(X509CvError::KeyUsageKeyCertSignMismatch);
      return false;
    }
  }
  true
}

#[inline]
fn validate_intermediate<'any, B, const IS_EE: bool>(
  cert: &'any CvCertificate<&'any [u8], IS_EE>,
  cv_policy: &CvPolicy<B>,
  depth: u8,
  intermediate: &'any CvIntermediate<&'any [u8]>,
  last_err: &mut Option<X509CvError>,
  verified_path: &mut VerifiedPath<'any, B>,
  intermediates: &'any [CvIntermediate<&'any [u8]>],
  trust_anchors: &'any [CvTrustAnchor<B>],
) -> Option<bool>
where
  B: Lease<[u8]>,
{
  if cert.issuer.lease() != intermediate.subject.bytes().lease() {
    return None;
  }

  if let Some(elem) = &cert.extended_key_usage
    && validate_eku::<false>(elem.extension(), &intermediate.extended_key_usage).is_err()
  {
    return None;
  }

  if !validate_chain_signature(cert, last_err, &intermediate.subject_public_key_info) {
    return None;
  }

  if let Some(basic_constraints) = &intermediate.basic_constraints {
    if !basic_constraints.extension().ca() {
      return None;
    }
    if let Some(plc) = basic_constraints.extension().path_len_constraint()
      && u32::from(depth) > plc
    {
      return None;
    }
  } else {
    return None;
  }

  if let Some(key_usage) = &intermediate.key_usage
    && !key_usage.key_cert_sign()
  {
    return None;
  }

  if !check_revocation(cert, cv_policy, depth, intermediate.key_usage, last_err) {
    return None;
  }

  if !check_verified_path(
    last_err,
    &intermediate.name_constraints,
    intermediate.subject.bytes().lease(),
    intermediate.subject_public_key_info.subject_public_key.bytes().lease(),
    verified_path,
  ) {
    return None;
  }

  if verified_path.intermediates_mut().push(intermediate).is_err() {
    *last_err = Some(X509CvError::ExceedDepth);
    return Some(false);
  }

  let next_depth = if intermediate.is_self_signed { depth } else { depth.wrapping_add(1) };

  if validate_chain::<_, false>(
    intermediate,
    cv_policy,
    next_depth,
    intermediates,
    last_err,
    trust_anchors,
    verified_path,
  ) {
    return Some(true);
  }

  let _ = verified_path.intermediates_mut().pop();
  None
}
