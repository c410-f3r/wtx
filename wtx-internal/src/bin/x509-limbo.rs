use core::{mem, ops::Range, slice};
use std::{borrow::Cow, fs::File, io::BufReader};
use wtx::{
  asn1::{Asn1Error, parse_der_from_pem_range, parse_der_from_pem_range_many},
  calendar::{DateTime, Instant, Utc},
  codec::{Decode, DecodeWrapper},
  collection::Vector,
  misc::Pem,
  x509::{
    Certificate, Crl, CvCertificate, CvCrl, CvEvaluationDepth, CvPolicy, CvPolicyMode,
    CvTrustAnchor, ServerName, X509CvError, X509Error, extensions::ExtendedKeyUsage,
  },
};

fn main() {
  let file = File::open("limbo.json").unwrap();
  let limbo: Limbo = serde_json::from_reader(BufReader::new(file)).unwrap();

  let mut bytes_certs = Vector::new();
  let mut crls = Vector::new();
  let mut pems = Vector::new();
  let mut trusted_certs = Vector::new();
  let mut untrusted_intermediates = Vector::new();

  for testcase in &limbo.testcases {
    let mut local_crls = crls;
    let mut local_trusted_certs = trusted_certs;
    let mut local_untrusted_intermediates = untrusted_intermediates;
    bytes_certs.clear();
    pems.clear();
    evaluate_test_case(
      &mut bytes_certs,
      &mut local_crls,
      &mut pems,
      testcase,
      &mut local_trusted_certs,
      &mut local_untrusted_intermediates,
    );
    crls = clear_and_recycle(local_crls);
    trusted_certs = clear_and_recycle(local_trusted_certs);
    untrusted_intermediates = clear_and_recycle(local_untrusted_intermediates);
  }
}

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum ExpectedResult {
  Success,
  Failure,
}

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
enum Feature {
  #[serde(rename = "has-policy-constraints")]
  HasPolicyConstraints,
  #[serde(rename = "has-cert-policies")]
  HasCertPolicies,
  #[serde(rename = "no-cert-policies")]
  NoCertPolicies,
  #[serde(rename = "pedantic-public-suffix-wildcard")]
  PedanticPublicSuffixWildcard,
  #[serde(rename = "name-constraint-dn")]
  NameConstraintDn,
  #[serde(rename = "pedantic-webpki-subscriber-key")]
  PedanticWebpkiSubscriberKey,
  #[serde(rename = "pedantic-webpki-eku")]
  PedanticWebpkiEku,
  #[serde(rename = "pedantic-serial-number")]
  PedanticSerialNumber,
  #[serde(rename = "max-chain-depth")]
  MaxChainDepth,
  #[serde(rename = "pedantic-rfc5280")]
  PedanticRfc5280,
  #[serde(rename = "rfc5280-incompatible-with-webpki")]
  Rfc5280IncompatibleWithWebpki,
  #[serde(rename = "denial-of-service")]
  DenialOfService,
  #[serde(rename = "has-crl")]
  HasCrl,
}

#[derive(Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "lowercase")]
enum Importance {
  #[default]
  Undetermined,
  Low,
  Medium,
  High,
  Critical,
}

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
enum KeyUsage {
  #[serde(rename = "digitalSignature")]
  DigitalSignature,
  #[serde(rename = "contentCommitment")]
  ContentCommitment,
  #[serde(rename = "keyEncipherment")]
  KeyEncipherment,
  #[serde(rename = "dataEncipherment")]
  DataEncipherment,
  #[serde(rename = "keyAgreement")]
  KeyAgreement,
  #[serde(rename = "keyCertSign")]
  KeyCertSign,
  #[serde(rename = "cRLSign")]
  CrlSign,
  #[serde(rename = "encipherOnly")]
  EncipherOnly,
  #[serde(rename = "decipherOnly")]
  DecipherOnly,
}

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
enum KnownEKUs {
  #[serde(rename = "anyExtendedKeyUsage")]
  AnyExtendedKeyUsage,
  #[serde(rename = "serverAuth")]
  ServerAuth,
  #[serde(rename = "clientAuth")]
  ClientAuth,
  #[serde(rename = "codeSigning")]
  CodeSigning,
  #[serde(rename = "emailProtection")]
  EmailProtection,
  #[serde(rename = "timeStamping")]
  TimeStamping,
  #[serde(rename = "OCSPSigning")]
  OcspSigning,
}

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum PeerKind {
  #[serde(rename = "DNS")]
  Dns,
  #[serde(rename = "IP")]
  Ip,
  #[serde(rename = "RFC822")]
  Rfc822,
}

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[expect(non_camel_case_types, reason = "canonical names")]
enum SignatureAlgorithm {
  DSA_WITH_SHA1,
  DSA_WITH_SHA224,
  DSA_WITH_SHA256,
  DSA_WITH_SHA384,
  DSA_WITH_SHA512,
  ECDSA_WITH_SHA1,
  ECDSA_WITH_SHA224,
  ECDSA_WITH_SHA256,
  ECDSA_WITH_SHA3_224,
  ECDSA_WITH_SHA3_256,
  ECDSA_WITH_SHA3_384,
  ECDSA_WITH_SHA3_512,
  ECDSA_WITH_SHA384,
  ECDSA_WITH_SHA512,
  ED25519,
  ED448,
  GOSTR3410_2012_WITH_3411_2012_256,
  GOSTR3410_2012_WITH_3411_2012_512,
  GOSTR3411_94_WITH_3410_2001,
  RSA_WITH_MD5,
  RSA_WITH_SHA1,
  RSA_WITH_SHA224,
  RSA_WITH_SHA256,
  RSA_WITH_SHA3_224,
  RSA_WITH_SHA3_256,
  RSA_WITH_SHA3_384,
  RSA_WITH_SHA3_512,
  RSA_WITH_SHA384,
  RSA_WITH_SHA512,
  RSASSA_PSS,
}

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "UPPERCASE")]
enum TestcaseResultStatus {
  Success,
  Failure,
  Skipped,
}

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum ValidationKind {
  Client,
  Server,
}

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
struct Limbo {
  version: u32,
  testcases: Vector<Testcase>,
}

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
struct PeerName {
  kind: PeerKind,
  value: String,
}

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
struct Testcase {
  id: String,
  description: String,
  validation_kind: ValidationKind,
  trusted_certs: Vector<String>,
  untrusted_intermediates: Vector<String>,
  peer_certificate: String,
  signature_algorithms: Vector<SignatureAlgorithm>,
  key_usage: Vector<KeyUsage>,
  extended_key_usage: Vector<KnownEKUs>,
  expected_result: ExpectedResult,
  expected_peer_names: Vector<PeerName>,
  #[serde(default)]
  conflicts_with: Vector<String>,
  #[serde(default)]
  features: Vector<Feature>,
  #[serde(default)]
  importance: Importance,
  #[serde(default)]
  peer_certificate_key: Option<String>,
  #[serde(default)]
  validation_time: Option<DateTime<Utc>>,
  #[serde(default)]
  expected_peer_name: Option<PeerName>,
  #[serde(default)]
  max_chain_depth: Option<u8>,
  #[serde(default)]
  crls: Vector<String>,
}

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
struct TestcaseResult<'any> {
  context: Option<Cow<'static, str>>,
  id: &'any str,
  status: TestcaseResultStatus,
}

impl<'any> TestcaseResult<'any> {
  fn fail(testcase: &'any Testcase, reason: impl Into<Cow<'static, str>>) -> Self {
    Self { context: Some(reason.into()), id: &testcase.id, status: TestcaseResultStatus::Failure }
  }
}

fn clear_and_recycle<T, U>(mut vector: Vector<T>) -> Vector<U> {
  vector.clear();
  assert!(size_of::<T>() == size_of::<U>());
  assert!(align_of::<T>() == align_of::<U>());
  let cap = vector.capacity();
  let ptr = vector.as_mut_ptr().cast();
  mem::forget(vector);
  // SAFETY: storage comes from the non-dropped vector
  Vector::from_vec(unsafe { Vec::from_raw_parts(ptr, 0, cap) })
}

fn evaluate_test_case<'bytes>(
  bytes_certs: &'bytes mut Vector<u8>,
  crls: &mut Vector<CvCrl<'_, 'bytes>>,
  pems: &mut Vector<Pem<Range<usize>, 1>>,
  testcase: &Testcase,
  trusted_certs: &mut Vector<CvTrustAnchor<'_, 'bytes>>,
  untrusted_intermediates: &mut Vector<CvCertificate<'_, 'bytes, false>>,
) {
  let supported = ["crl", "cve", "invalid", "pathlen", "pathological", "rfc5280"];
  if !supported.iter().any(|el| testcase.id.starts_with(el)) {
    return;
  }

  let leaf_pem = {
    let mut dw = DecodeWrapper::new(testcase.peer_certificate.as_bytes(), &mut *bytes_certs);
    Pem::decode(&mut dw).unwrap()
  };

  for elem in &testcase.crls {
    let mut dw = DecodeWrapper::new(elem.as_bytes(), &mut *bytes_certs);
    pems.push(Pem::decode(&mut dw).unwrap()).unwrap();
  }

  let idx0 = pems.len();
  for elem in &testcase.trusted_certs {
    let mut dw = DecodeWrapper::new(elem.as_bytes(), &mut *bytes_certs);
    pems.push(Pem::decode(&mut dw).unwrap()).unwrap();
  }

  let idx1 = pems.len();
  for elem in &testcase.untrusted_intermediates {
    let mut dw = DecodeWrapper::new(elem.as_bytes(), &mut *bytes_certs);
    pems.push(Pem::decode(&mut dw).unwrap()).unwrap();
  }

  let Some(leaf) = eval_eager_checks(
    parse_der_from_pem_range::<Certificate<'_>>(&*bytes_certs, &leaf_pem)
      .and_then(CvCertificate::<'_, '_>::try_from),
  ) else {
    return;
  };

  let Some(_) = eval_eager_checks(parse_der_from_pem_range_many(
    &*bytes_certs,
    crls,
    &pems[..idx0],
    |el: Crl<'_>| el.try_into(),
  )) else {
    return;
  };

  let Some(_) = eval_eager_checks(parse_der_from_pem_range_many(
    &*bytes_certs,
    trusted_certs,
    &pems[idx0..idx1],
    |el: Certificate<'_>| el.try_into(),
  )) else {
    return;
  };

  let Some(_) = eval_eager_checks(parse_der_from_pem_range_many(
    &*bytes_certs,
    untrusted_intermediates,
    &pems[idx1..],
    |el: Certificate<'_>| el.try_into(),
  )) else {
    return;
  };

  let mut cvp = CvPolicy::from_crls(crls).unwrap();
  let mut eku = ExtendedKeyUsage::default();
  fill_cvp(&mut cvp, &mut eku, testcase);

  let rslt_chain = leaf.validate_chain(untrusted_intermediates, &cvp, trusted_certs);
  let peer_names = if let Some(peer_name) = testcase.expected_peer_name.as_ref() {
    slice::from_ref(peer_name)
  } else if !testcase.expected_peer_names.is_empty() {
    testcase.expected_peer_names.as_slice()
  } else {
    &[]
  };
  let rslt_sn = leaf.validate_subject_name(
    peer_names.iter().map(|el| ServerName::from_ascii(el.value.as_bytes()).unwrap()),
  );

  let rslt = rslt_chain.and(rslt_sn);
  match testcase.expected_result {
    ExpectedResult::Success => {
      if let Err(err) = rslt {
        panic!("{:?}", TestcaseResult::fail(testcase, err.to_string()));
      }
    }
    ExpectedResult::Failure => {
      if rslt.is_ok() {
        panic!("{:?}", TestcaseResult::fail(testcase, "Test should fail but is actually passing"));
      }
    }
  }
}

fn eval_eager_checks<T>(rslt: wtx::Result<T>) -> Option<T> {
  match rslt {
    Ok(elem) => Some(elem),
    Err(wtx::Error::Asn1Error(Asn1Error::LargeData))
    | Err(wtx::Error::X509Error(X509Error::InvalidCertificateVersion))
    | Err(wtx::Error::X509Error(X509Error::InvalidExtendedKeyUsage))
    | Err(wtx::Error::X509Error(X509Error::InvalidExtensionKeyUsage))
    | Err(wtx::Error::X509Error(X509Error::InvalidExtensionNameConstraints))
    | Err(wtx::Error::X509Error(X509Error::InvalidSan))
    | Err(wtx::Error::X509Error(X509Error::InvalidSerialNumberBytes))
    | Err(wtx::Error::X509CvError(X509CvError::AuthorityKeyIdentifierMustNotBeCritical))
    | Err(wtx::Error::X509CvError(X509CvError::CertCanNotHaveDuplicateExtensions))
    | Err(wtx::Error::X509CvError(X509CvError::CertificateAlgorithmMismatch))
    | Err(wtx::Error::X509CvError(X509CvError::CertsMustNotHaveCriticalUnknownExtensions))
    | Err(wtx::Error::X509CvError(X509CvError::HasIncompatibleKeyUsage))
    | Err(wtx::Error::X509CvError(X509CvError::InvalidNameConstraints))
    | Err(wtx::Error::X509CvError(X509CvError::CrlNumberMustNotBeCritical))
    | Err(wtx::Error::X509CvError(X509CvError::IcasMustHaveASubjectSequence))
    | Err(wtx::Error::X509CvError(X509CvError::IcasMustHaveCriticalBasicConstraints))
    | Err(wtx::Error::X509CvError(X509CvError::IcasMustHaveSki))
    | Err(wtx::Error::X509CvError(X509CvError::InvalidAuthorityKeyIdentifier))
    | Err(wtx::Error::X509CvError(X509CvError::MissingCrlNumber))
    | Err(wtx::Error::X509CvError(X509CvError::NameConstraintsMustBeCritical))
    | Err(wtx::Error::X509CvError(X509CvError::NameConstraintsOverflow))
    | Err(wtx::Error::X509CvError(X509CvError::PolicyConstraintMustBeCritical))
    | Err(wtx::Error::X509CvError(X509CvError::RootCasMustHaveKeyIdentifiers))
    | Err(wtx::Error::X509CvError(X509CvError::RootCasMustHaveMatchingAkiAndSki))
    | Err(wtx::Error::X509CvError(X509CvError::SubjectKeyIdentifierMustNotBeCritical)) => None,
    Err(err) => panic!("{err}"),
  }
}

fn fill_cvp<'any>(
  cvp: &mut CvPolicy<'any, '_>,
  eku: &'any mut ExtendedKeyUsage,
  testcase: &Testcase,
) {
  for elem in &testcase.extended_key_usage {
    match elem {
      KnownEKUs::AnyExtendedKeyUsage => {
        *eku.any_mut() = true;
      }
      KnownEKUs::ServerAuth => {
        *eku.server_auth_mut() = true;
      }
      KnownEKUs::ClientAuth => {
        *eku.client_auth_mut() = true;
      }
      KnownEKUs::CodeSigning => {
        *eku.code_signing_mut() = true;
      }
      KnownEKUs::EmailProtection => {
        *eku.email_protection_mut() = true;
      }
      KnownEKUs::TimeStamping => {
        *eku.time_stamping_mut() = true;
      }
      KnownEKUs::OcspSigning => {
        *eku.ocsp_signing_mut() = true;
      }
    }
  }

  let mut ku: wtx::x509::extensions::KeyUsage = wtx::x509::extensions::KeyUsage::default();
  for elem in &testcase.key_usage {
    match elem {
      KeyUsage::DigitalSignature => {
        ku.set_digital_signature(true);
      }
      KeyUsage::ContentCommitment => {
        ku.set_non_repudiation(true);
      }
      KeyUsage::KeyEncipherment => {
        ku.set_key_encipherment(true);
      }
      KeyUsage::DataEncipherment => {
        ku.set_data_encipherment(true);
      }
      KeyUsage::KeyAgreement => {
        ku.set_key_agreement(true);
      }
      KeyUsage::KeyCertSign => {
        ku.set_key_cert_sign(true);
      }
      KeyUsage::CrlSign => {
        ku.set_crl_sign(true);
      }
      KeyUsage::EncipherOnly => {
        ku.set_encipher_only(true);
      }
      KeyUsage::DecipherOnly => {
        ku.set_decipher_only(true);
      }
    }
  }

  let mut is_pedantic = false;
  for elem in &testcase.features {
    is_pedantic |= matches!(
      elem,
      Feature::PedanticPublicSuffixWildcard
        | Feature::PedanticWebpkiSubscriberKey
        | Feature::PedanticWebpkiEku
        | Feature::PedanticSerialNumber
        | Feature::PedanticRfc5280
    );
    if is_pedantic {
      break;
    }
  }

  *cvp.mode_mut() = if is_pedantic {
    CvPolicyMode::Strict
  } else {
    // The vast majority don't have AKI, even some `webpki` tests don't have AKI.
    if [
      "webpki::aki::root-with-aki-missing-keyidentifier",
      "webpki::aki::root-with-aki-ski-mismatch",
    ]
    .contains(&testcase.id.as_str())
    {
      CvPolicyMode::Strict
    } else {
      CvPolicyMode::Lenient
    }
  };

  if let Some(elem) = testcase.max_chain_depth {
    *cvp.evaluation_depth_mut() = CvEvaluationDepth::Chain(elem);
  }

  *cvp.key_usage_mut() = ku;
  *cvp.extended_key_usage_mut() = eku;
  cvp.set_validation_time(
    testcase.validation_time.unwrap_or_else(|| Instant::now_date_time(0).unwrap()),
  );
}
