use crate::{
  asn1::{
    Asn1DecodeWrapper, OID_X509_COMMON_NAME, OID_X509_EXT_BASIC_CONSTRAINTS,
    OID_X509_EXT_SUBJECT_ALT_NAME, Oid, decode_asn1_tlv,
  },
  calendar::{DateTime, Utc},
  codec::{Decode, GenericDecodeWrapper},
  crypto::SignatureTy,
  misc::{Either, Lease, bytes_split_once1},
  x509::{
    Certificate, Extension, GeneralName, SubjectPublicKeyInfo, X509Error,
    extensions::{BasicConstraints, SubjectAlternativeName},
  },
};

/// Semantic entry-point where certificates can be validated.
///
/// Servers should concurrently or sequentially call `validate_chain`, `validate_signature` and
/// `validate_subject_name` to fully validate certificates. It is also possible to call
/// `validate_server` to sequentially invoke all three methods at once.
///
/// Clients should concurrently or sequentially call `validate_chain` and `validate_signature` to
/// to fully validate certificates. It is also possible to call `validate_client` to sequentially
/// invoke all two methods at once.
#[derive(Debug, PartialEq)]
pub struct EndEntityCert<C>(
  /// Generic Certificate
  pub C,
);

impl<'bytes, C> EndEntityCert<C>
where
  C: Lease<Certificate<'bytes>>,
{
  /// Sequentially calls [`Self::validate_chain`], [`Self::validate_signature`].
  #[inline]
  pub fn validate_client(
    &self,
    intermediate_certs: &[Certificate<'_>],
    msg: &[u8],
    signature: &[u8],
    time: DateTime<Utc>,
    trust_anchors: &[Certificate<'_>],
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
    intermediate_certs: &[Certificate<'_>],
    time: DateTime<Utc>,
    trust_anchors: &[Certificate<'_>],
  ) -> crate::Result<()> {
    if !do_validate_chain(self.0.lease(), 0, intermediate_certs, time, trust_anchors)? {
      return Err(X509Error::ChainValidationDidNotFindPath.into());
    }
    Ok(())
  }

  /// Verifies `signature` over `msg` using the public key contained in this certificate.
  #[inline]
  pub fn validate_signature(&self, msg: &[u8], signature: &[u8]) -> crate::Result<()> {
    let spki = &self.0.lease().tbs_certificate.subject_public_key_info;
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
    intermediate_certs: &[Certificate<'_>],
    msg: &[u8],
    signature: &[u8],
    sn: &[u8],
    time: DateTime<Utc>,
    trust_anchors: &[Certificate<'_>],
  ) -> crate::Result<()> {
    self.validate_chain(intermediate_certs, time, trust_anchors)?;
    self.validate_signature(msg, signature)?;
    self.validate_subject_name(sn)?;
    Ok(())
  }

  /// Matches `sn` with [`SubjectAlternativeName`] if it exists, otherwise tries to match `sn`
  /// with the common name.
  #[inline]
  pub fn validate_subject_name(&self, sn: &[u8]) -> crate::Result<()> {
    let cert = self.0.lease();
    let mut has_san = false;
    if let Some(exts) = &cert.tbs_certificate.extensions {
      for ext in exts.0.iter() {
        has_san |= match validate_sn_from_extension(ext, sn)? {
          Either::Left(_) => return Ok(()),
          Either::Right(has_san) => has_san,
        };
      }
    }
    if !has_san && validate_sn_from_subject(cert, sn)? {
      return Ok(());
    }
    Err(X509Error::UnknownSubjectName.into())
  }
}

#[inline]
fn do_validate_chain(
  cert: &Certificate<'_>,
  depth: u8,
  intermediate_certs: &[Certificate<'_>],
  time: DateTime<Utc>,
  trust_anchors: &[Certificate<'_>],
) -> crate::Result<bool> {
  if depth > 10 {
    return Err(X509Error::ChainValidationExceedDepth.into());
  }

  let not_before = cert.tbs_certificate.validity.not_before.date_time();
  let not_after = cert.tbs_certificate.validity.not_after.date_time();
  if time < not_before || time > not_after {
    return Err(X509Error::ChainValidationHasExpiredCerts.into());
  }

  for ta_cert in trust_anchors {
    if cert.tbs_certificate.issuer != ta_cert.tbs_certificate.subject {
      continue;
    }
    validate_chain_signature(cert, ta_cert)?;
    return Ok(true);
  }

  'outer: for inter in intermediate_certs {
    if cert.tbs_certificate.issuer != inter.tbs_certificate.subject {
      continue 'outer;
    }
    if validate_chain_signature(cert, inter).is_err() {
      continue 'outer;
    }
    let Some(exts) = &inter.tbs_certificate.extensions else {
      continue 'outer;
    };
    let mut is_ca = false;
    'inner: for ext in exts.0.iter() {
      if ext.extn_id != OID_X509_EXT_BASIC_CONSTRAINTS {
        continue 'inner;
      }
      let bc = BasicConstraints::decode(&mut GenericDecodeWrapper::new(
        ext.extn_value.bytes(),
        Asn1DecodeWrapper::default(),
      ))?;
      is_ca = bc.ca;
      let Some(plc) = bc.path_len_constraint else {
        continue 'inner;
      };
      if u32::from(depth) > plc.u32() {
        return Err(X509Error::ChainValidationExceedDepth.into());
      }
    }
    if is_ca {
      let local_depth = depth.wrapping_add(1);
      if do_validate_chain(inter, local_depth, intermediate_certs, time, trust_anchors)? {
        return Ok(true);
      }
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
  child: &Certificate<'_>,
  parent: &Certificate<'_>,
) -> crate::Result<()> {
  if child.signature_algorithm != child.tbs_certificate.signature {
    return Err(X509Error::ChainValidationDidNotFindPath.into());
  }
  let par_spki = &parent.tbs_certificate.subject_public_key_info;
  let child_alg_oid = &child.signature_algorithm.algorithm;
  let par_params_oid = params_oid(par_spki);
  let signature_ty = SignatureTy::try_from((child_alg_oid, par_params_oid.as_ref()))?;
  signature_ty.validate_signature(
    par_spki.subject_public_key.bytes(),
    child.tbs_certificate.bytes,
    child.signature_value.bytes(),
  )?;
  Ok(())
}

#[inline]
fn validate_sn_from_extension(ext: &Extension<'_>, sn: &[u8]) -> crate::Result<Either<(), bool>> {
  if ext.extn_id != OID_X509_EXT_SUBJECT_ALT_NAME {
    return Ok(Either::Right(false));
  }
  let san = SubjectAlternativeName::decode(&mut GenericDecodeWrapper::new(
    ext.extn_value.bytes(),
    Asn1DecodeWrapper::default(),
  ))?;
  for gn in san.0.0.iter() {
    match gn {
      GeneralName::DnsName(dns) if matches_domain(dns, sn) => {
        return Ok(Either::Left(()));
      }
      GeneralName::IpAddress(ip) if *ip == sn => {
        return Ok(Either::Left(()));
      }
      _ => {}
    }
  }
  Ok(Either::Right(true))
}

#[inline]
fn validate_sn_from_subject(cert: &Certificate<'_>, sn: &[u8]) -> crate::Result<bool> {
  for rdn in cert.tbs_certificate.subject.rdn_sequence.0.iter() {
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
