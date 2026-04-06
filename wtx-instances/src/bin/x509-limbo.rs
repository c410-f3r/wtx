// Based on the code available at the x509-limbo repository.

use core::{mem, ops::Range};
use std::{borrow::Cow, process::ExitCode};
use wtx::{
  asn1::{parse_der_from_pem_range, parse_der_from_pem_range_many},
  calendar::{DateTime, Instant, Utc},
  codec::{Decode, GenericDecodeWrapper},
  collection::Vector,
  misc::Pem,
  x509::{Certificate, CertificateBasic, EndEntityCert, ServerName, TrustAnchorBasic},
};

fn main() -> ExitCode {
  let limbo: Limbo = serde_json::from_reader(std::io::stdin()).unwrap();

  let mut bytes_certs = Vector::new();
  let mut crls = Vector::new();
  let mut is_success = true;
  let mut pems = Vector::new();
  let mut results = Vector::new();
  let mut trusted_certs = Vector::new();
  let mut untrusted_intermediates = Vector::new();

  for testcase in &limbo.testcases {
    let mut local_crls = crls;
    let mut local_trusted_certs = trusted_certs;
    let mut local_untrusted_intermediates = untrusted_intermediates;
    bytes_certs.clear();
    pems.clear();
    let result = evaluate_test_case(
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

    if matches!(result.status, TestcaseResultStatus::Failure) {
      is_success = false;
    }
    results.push(result).unwrap();
  }

  let limbo_result = LimboResult { version: 1, harness: "wtx", results };
  serde_json::to_writer_pretty(std::io::stdout(), &limbo_result).unwrap();
  if is_success { ExitCode::SUCCESS } else { ExitCode::FAILURE }
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
struct LimboResult<'any> {
  version: u8,
  harness: &'any str,
  results: Vector<TestcaseResult<'any>>,
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
  max_chain_depth: Option<u64>,
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

  fn skip(testcase: &'any Testcase, reason: impl Into<Cow<'static, str>>) -> Self {
    Self { context: Some(reason.into()), id: &testcase.id, status: TestcaseResultStatus::Skipped }
  }

  fn success(testcase: &'any Testcase) -> Self {
    Self { context: None, id: &testcase.id, status: TestcaseResultStatus::Success }
  }
}

fn evaluate_test_case<'bytes, 'tc>(
  bytes_certs: &'bytes mut Vector<u8>,
  crls: &mut Vector<CertificateBasic<'_, 'bytes>>,
  pems: &mut Vector<Pem<Range<usize>, 1>>,
  testcase: &'tc Testcase,
  trusted_certs: &mut Vector<TrustAnchorBasic<'_, 'bytes>>,
  untrusted_intermediates: &mut Vector<CertificateBasic<'_, 'bytes>>,
) -> TestcaseResult<'tc> {
  if testcase.features.contains(&Feature::MaxChainDepth) {
    return TestcaseResult::skip(
      testcase,
      "max-chain-depth testcases are not supported by this API",
    );
  }
  if !matches!(testcase.validation_kind, ValidationKind::Server) {
    return TestcaseResult::skip(testcase, "non-SERVER testcases not supported yet");
  }
  if !testcase.signature_algorithms.is_empty() {
    return TestcaseResult::skip(testcase, "signature_algorithms not supported yet");
  }
  if !testcase.key_usage.is_empty() {
    return TestcaseResult::skip(testcase, "key_usage not supported yet");
  }

  let leaf_pem = {
    let mut dw = GenericDecodeWrapper::new(testcase.peer_certificate.as_bytes(), &mut *bytes_certs);
    Pem::decode(&mut dw).unwrap()
  };

  for elem in &testcase.crls {
    let mut dw = GenericDecodeWrapper::new(elem.as_bytes(), &mut *bytes_certs);
    pems.push(Pem::decode(&mut dw).unwrap()).unwrap();
  }

  let idx0 = pems.len();
  for elem in &testcase.trusted_certs {
    let mut dw = GenericDecodeWrapper::new(elem.as_bytes(), &mut *bytes_certs);
    pems.push(Pem::decode(&mut dw).unwrap()).unwrap();
  }

  let idx1 = pems.len();
  for elem in &testcase.untrusted_intermediates {
    let mut dw = GenericDecodeWrapper::new(elem.as_bytes(), &mut *bytes_certs);
    pems.push(Pem::decode(&mut dw).unwrap()).unwrap();
  }

  let leaf = EndEntityCert::<CertificateBasic<'_, '_>>(
    parse_der_from_pem_range::<Certificate<'_>>(&*bytes_certs, &leaf_pem)
      .unwrap()
      .try_into()
      .unwrap(),
  );
  parse_der_from_pem_range_many(&*bytes_certs, crls, &pems[..idx0], |el: Certificate<'_>| {
    Ok(el.try_into()?)
  })
  .unwrap();
  parse_der_from_pem_range_many(
    &*bytes_certs,
    trusted_certs,
    &pems[idx0..idx1],
    |el: Certificate<'_>| Ok(el.try_into()?),
  )
  .unwrap();
  parse_der_from_pem_range_many(
    &*bytes_certs,
    untrusted_intermediates,
    &pems[idx1..],
    |el: Certificate<'_>| Ok(el.try_into()?),
  )
  .unwrap();

  if let Err(err) = leaf.validate_chain(
    untrusted_intermediates,
    testcase.validation_time.unwrap_or(Instant::now_date_time(0).unwrap()),
    trusted_certs,
  ) {
    return TestcaseResult::fail(testcase, err.to_string());
  }

  let Some(peer_name) = testcase.expected_peer_name.as_ref() else {
    return TestcaseResult::skip(testcase, "implementation requires peer names");
  };
  if leaf
    .validate_subject_name(ServerName::from_arbitrary_bytes(peer_name.value.as_bytes()).unwrap())
    .is_err()
  {
    return TestcaseResult::fail(testcase, "subject name validation failed");
  }

  TestcaseResult::success(testcase)
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
