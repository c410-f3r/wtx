// Based on the code available at the x509-limbo repository.

use core::ops::Range;
use std::process::ExitCode;
use wtx::{
  calendar::{DateTime, Instant, Utc},
  collection::Vector,
  misc::Pem,
};

fn main() -> ExitCode {
  let limbo: Limbo = serde_json::from_reader(std::io::stdin()).unwrap();
  let mut bytes = Vector::new();
  let mut pems = Vector::new();
  let mut _is_success = true;
  let mut results = Vector::new();
  for testcase in limbo.testcases {
    let result = evaluate_test_case(&mut bytes, &mut pems, &testcase);
    if matches!(result.status, TestcaseResultStatus::Failure) {
      _is_success = false;
    }
    results.push(result).unwrap();
  }
  let limbo_result = LimboResult { version: 1, harness: "wtx".into(), results };
  serde_json::to_writer_pretty(std::io::stdout(), &limbo_result).unwrap();
  ExitCode::SUCCESS
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
  #[serde(rename = "RFC822")]
  Rfc822,
  #[serde(rename = "DNS")]
  Dns,
  #[serde(rename = "IP")]
  Ip,
}

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[expect(non_camel_case_types, reason = "canonical names")]
enum SignatureAlgorithm {
  RSA_WITH_MD5,
  RSA_WITH_SHA1,
  RSA_WITH_SHA224,
  RSA_WITH_SHA256,
  RSA_WITH_SHA384,
  RSA_WITH_SHA512,
  RSA_WITH_SHA3_224,
  RSA_WITH_SHA3_256,
  RSA_WITH_SHA3_384,
  RSA_WITH_SHA3_512,
  RSASSA_PSS,
  ECDSA_WITH_SHA1,
  ECDSA_WITH_SHA224,
  ECDSA_WITH_SHA256,
  ECDSA_WITH_SHA384,
  ECDSA_WITH_SHA512,
  ECDSA_WITH_SHA3_224,
  ECDSA_WITH_SHA3_256,
  ECDSA_WITH_SHA3_384,
  ECDSA_WITH_SHA3_512,
  DSA_WITH_SHA1,
  DSA_WITH_SHA224,
  DSA_WITH_SHA256,
  DSA_WITH_SHA384,
  DSA_WITH_SHA512,
  ED25519,
  ED448,
  GOSTR3411_94_WITH_3410_2001,
  GOSTR3410_2012_WITH_3411_2012_256,
  GOSTR3410_2012_WITH_3411_2012_512,
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
struct LimboResult {
  version: u8,
  harness: String,
  results: Vector<TestcaseResult>,
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
struct TestcaseResult {
  context: Option<String>,
  id: String,
  status: TestcaseResultStatus,
}

impl TestcaseResult {
  fn fail(tc: &Testcase, reason: &str) -> Self {
    Self {
      context: Some(reason.into()),
      id: tc.id.to_string(),
      status: TestcaseResultStatus::Failure,
    }
  }

  fn skip(tc: &Testcase, reason: &str) -> Self {
    Self {
      context: Some(reason.into()),
      id: tc.id.to_string(),
      status: TestcaseResultStatus::Skipped,
    }
  }

  fn success(tc: &Testcase) -> Self {
    Self { id: tc.id.to_string(), status: TestcaseResultStatus::Success, context: None }
  }
}

fn evaluate_test_case(
  bytes: &mut Vector<u8>,
  pems: &mut Vector<Pem<Range<usize>, 1>>,
  test_case: &Testcase,
) -> TestcaseResult {
  if test_case.features.contains(&Feature::MaxChainDepth) {
    return TestcaseResult::skip(
      test_case,
      "max-chain-depth testcases are not supported by this API",
    );
  }
  if !matches!(test_case.validation_kind, ValidationKind::Server) {
    return TestcaseResult::skip(test_case, "non-SERVER testcases not supported yet");
  }
  if !test_case.signature_algorithms.is_empty() {
    return TestcaseResult::skip(test_case, "signature_algorithms not supported yet");
  }
  if !test_case.key_usage.is_empty() {
    return TestcaseResult::skip(test_case, "key_usage not supported yet");
  }

  bytes.clear();
  pems.clear();
  //  let leaf_pem = {
  //    let mut dw = GenericDecodeWrapper::new(test_case.peer_certificate.as_bytes(), &mut *bytes);
  //    Pem::decode(&mut dw).unwrap()
  //  };
  //  for elem in &test_case.trusted_certs {
  //    let mut dw = GenericDecodeWrapper::new(elem.as_bytes(), &mut *bytes);
  //    pems.push(Pem::decode(&mut dw).unwrap()).unwrap();
  //  }
  //  let pems_idx = pems.len();
  //  for elem in &test_case.untrusted_intermediates {
  //    let mut dw = GenericDecodeWrapper::new(elem.as_bytes(), &mut *bytes);
  //    pems.push(Pem::decode(&mut dw).unwrap()).unwrap();
  //  }
  //
  //  let _leaf = EndEntityCert(Certificate::from_pem(&*bytes, &leaf_pem));
  //  let mut intermediates = Vector::new();
  //  Certificate::from_pems(&*bytes, &mut intermediates, &pems[..pems_idx]).unwrap();
  //  let mut trust_anchors = Vector::new();
  //  Certificate::from_pems(&*bytes, &mut trust_anchors, &pems[pems_idx..]).unwrap();

  let _validation_time = test_case.validation_time.unwrap_or(Instant::now_date_time(0).unwrap());

  let Some(_peer_name) = test_case.expected_peer_name.as_ref() else {
    return TestcaseResult::skip(test_case, "implementation requires peer names");
  };
  if true {
    TestcaseResult::fail(test_case, "subject name validation failed")
  } else {
    TestcaseResult::success(test_case)
  }
}
