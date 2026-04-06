use crate::{
  asn1::{Asn1DecodeWrapper, OID_X509_COMMON_NAME, Oid, decode_asn1_tlv},
  calendar::{DateTime, Utc},
  codec::{Decode, GenericDecodeWrapper},
  crypto::SignatureTy,
  misc::{Lease, bytes_split_once1},
  x509::{
    GeneralName, ServerName, SubjectPublicKeyInfo, TrustAnchorBasic, X509Error,
    certificate_basic::CertificateBasic, extensions::SubjectAlternativeName,
  },
};

/// Final leaf of a PKI chain and also an entry-point where certificates can be validated.
///
/// Servers should concurrently or sequentially call [`Self::validate_chain`],
/// [`Self::validate_signature`] and [`Self::validate_subject_name`] to fully validate
/// certificates. It is also possible to call [`Self::validate_server`] to sequentially invoke
/// all three methods at once.
///
/// Clients should concurrently or sequentially call [`Self::validate_chain`] and
/// [`Self::validate_signature`] to fully validate certificates. It is also possible to call
/// [`Self::validate_client`] to sequentially invoke all two methods at once.
#[derive(Debug, PartialEq)]
pub struct EndEntityCert<C>(
  /// Generic Certificate
  pub C,
);

impl<'any, 'bytes, C> EndEntityCert<C>
where
  C: Lease<CertificateBasic<'any, 'bytes>>,
  'bytes: 'any,
{
  /// Sequentially calls [`Self::validate_chain`], [`Self::validate_signature`].
  #[inline]
  pub fn validate_client(
    &self,
    intermediate_certs: &[CertificateBasic<'_, '_>],
    msg: &[u8],
    signature: &[u8],
    time: DateTime<Utc>,
    trust_anchors: &[TrustAnchorBasic<'_, '_>],
  ) -> crate::Result<()> {
    self.validate_chain(intermediate_certs, time, trust_anchors)?;
    self.validate_signature(msg, signature)?;
    Ok(())
  }

  /// Checks that a valid path exists when walking through the provided intermediate certificates.
  /// A valid path is constructed when it hits one of the trust anchors and the associated
  /// constraints like expirations times are satisfied.
  ///
  /// It is worth noting that this method is not cheap and the number of intermediates is a
  /// considerable factor.
  #[inline]
  pub fn validate_chain(
    &self,
    intermediate_certs: &[CertificateBasic<'_, '_>],
    time: DateTime<Utc>,
    trust_anchors: &[TrustAnchorBasic<'_, '_>],
  ) -> crate::Result<()> {
    if !do_validate_chain(self.0.lease(), 0, intermediate_certs, time, trust_anchors)? {
      return Err(X509Error::ChainValidationDidNotFindPath.into());
    }
    Ok(())
  }

  /// Verifies `signature` over `msg` using the public key contained in this certificate.
  #[inline]
  pub fn validate_signature(&self, msg: &[u8], signature: &[u8]) -> crate::Result<()> {
    let spki = &self.0.lease().spki.lease();
    let alg_oid = &spki.algorithm.algorithm;
    let pk = spki.subject_public_key.bytes();
    let signature_ty = SignatureTy::try_from((alg_oid, params_oid(spki).as_ref()))?;
    signature_ty.validate_signature(pk, msg, signature)?;
    Ok(())
  }

  /// Sequentially calls [`Self::validate_chain`], [`Self::validate_signature`] and
  /// [`Self::validate_subject_name`].
  #[inline]
  pub fn validate_server(
    &self,
    intermediate_certs: &[CertificateBasic<'_, '_>],
    msg: &[u8],
    signature: &[u8],
    sn: ServerName<&[u8]>,
    time: DateTime<Utc>,
    trust_anchors: &[TrustAnchorBasic<'_, '_>],
  ) -> crate::Result<()> {
    self.validate_chain(intermediate_certs, time, trust_anchors)?;
    self.validate_signature(msg, signature)?;
    self.validate_subject_name(sn)?;
    Ok(())
  }

  /// Matches `sn` with [`SubjectAlternativeName`] if it exists, otherwise tries to match `sn`
  /// with the common name.
  #[inline]
  pub fn validate_subject_name(&self, sn: ServerName<&[u8]>) -> crate::Result<()> {
    let mut ip_buffer = &mut [0; 16];
    let sn_bytes = sn.bytes(&mut ip_buffer);
    let cert = self.0.lease();
    if validate_sn(sn_bytes, &cert.subject_alt_name)? {
      return Ok(());
    }
    if validate_sn_from_subject(cert, sn_bytes)? {
      return Ok(());
    }
    Err(X509Error::UnknownSubjectName.into())
  }
}

impl<C> From<C> for EndEntityCert<C> {
  #[inline]
  fn from(value: C) -> Self {
    Self(value)
  }
}

#[inline]
fn do_validate_chain(
  cert: &CertificateBasic<'_, '_>,
  depth: u8,
  intermediate_certs: &[CertificateBasic<'_, '_>],
  time: DateTime<Utc>,
  trust_anchors: &[TrustAnchorBasic<'_, '_>],
) -> crate::Result<bool> {
  if depth > 10 {
    return Err(X509Error::ChainValidationExceedDepth.into());
  }

  let not_before = cert.validity.not_before.date_time();
  let not_after = cert.validity.not_after.date_time();
  if time < not_before || time > not_after {
    return Err(X509Error::ChainValidationHasExpiredCerts.into());
  }

  for ta_cert in trust_anchors {
    if cert.issuer != ta_cert.subject {
      continue;
    }
    validate_chain_signature(cert, ta_cert.subject_public_key_info.lease())?;
    return Ok(true);
  }

  'outer: for inter in intermediate_certs {
    if cert.issuer != inter.subject {
      continue 'outer;
    }
    if validate_chain_signature(cert, inter.spki.lease()).is_err() {
      continue 'outer;
    }
    let Some(bc) = &inter.basic_constraints else {
      continue 'outer;
    };
    let Some(plc) = &bc.path_len_constraint() else {
      continue 'outer;
    };
    if u32::from(depth) > plc.u32() {
      return Err(X509Error::ChainValidationExceedDepth.into());
    }
    if do_validate_chain(inter, depth.wrapping_add(1), intermediate_certs, time, trust_anchors)? {
      return Ok(true);
    }
  }

  Ok(false)
}

#[inline]
fn matches_domain(cert_name: &[u8], sn: &[u8]) -> bool {
  let [b'*', b'.', rest @ ..] = cert_name else {
    return cert_name.eq_ignore_ascii_case(sn);
  };
  let Some((_, rhs)) = bytes_split_once1(sn, b'.') else {
    return false;
  };
  rest.eq_ignore_ascii_case(rhs)
}

#[inline]
fn params_oid(spki: &SubjectPublicKeyInfo<'_>) -> Option<Oid> {
  spki.algorithm.parameters.as_ref().and_then(|el| {
    Oid::decode(&mut GenericDecodeWrapper::new(el.data(), Asn1DecodeWrapper::default())).ok()
  })
}

#[inline]
fn validate_chain_signature(
  child: &CertificateBasic<'_, '_>,
  parent: &SubjectPublicKeyInfo<'_>,
) -> crate::Result<()> {
  let child_sig_alg = child.signature_algorithm.lease();
  if child_sig_alg != &parent.algorithm {
    return Err(X509Error::ChainValidationDidNotFindPath.into());
  }
  let par_params_oid = params_oid(parent);
  let signature_ty = SignatureTy::try_from((&child_sig_alg.algorithm, par_params_oid.as_ref()))?;
  signature_ty.validate_signature(
    parent.subject_public_key.bytes(),
    child.signature_msg,
    child.signature,
  )?;
  Ok(())
}

#[inline]
fn validate_sn(sn: &[u8], san_opt: &Option<SubjectAlternativeName<'_>>) -> crate::Result<bool> {
  let Some(san) = san_opt else {
    return Ok(false);
  };
  for gn in san.0.0.iter() {
    match gn {
      GeneralName::DnsName(dns) if matches_domain(dns, sn) => {
        return Ok(true);
      }
      GeneralName::IpAddress(ip) if *ip == sn => {
        return Ok(true);
      }
      _ => {}
    }
  }
  Ok(false)
}

#[inline]
fn validate_sn_from_subject(cert: &CertificateBasic<'_, '_>, sn: &[u8]) -> crate::Result<bool> {
  for rdn in cert.subject.lease().rdn_sequence.0.iter() {
    for atv in rdn.0.iter() {
      if atv.oid != OID_X509_COMMON_NAME {
        continue;
      }
      let (_, _, val, _) = decode_asn1_tlv(atv.value.data())?;
      if matches_domain(val, sn) {
        return Ok(true);
      }
    }
  }
  Ok(false)
}
