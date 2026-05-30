//! CCADB

use std::{
  fmt::{Debug, Display, Formatter, Write},
  fs,
  io::BufReader,
};
use wtx::{
  calendar::{Date, DateTime, Duration, Instant, Time, Utc, parse_bytes_into_tokens},
  codec::{Csv, HexDisplay, HexEncMode},
  collection::{ArrayVectorU8, HashSet, Vector},
  http::{HttpClient, ReqBuilder, client_pool::ClientPoolBuilder},
  misc::UriRef,
  x509::{Certificate, CvTrustAnchor, X509Error},
};

static EXCLUDED_FINGERPRINTS: &[&str] =
  &["9A296A5182D1D451A2E37F439B74DAAFA267523329F90F9A0D2007C334E23C9A"];

#[tokio::main]
async fn main() {
  let csv = {
    let uri = "https://ccadb.my.salesforce-sites.com/mozilla/IncludedCACertificateReportPEMCSV";
    let pool = ClientPoolBuilder::tokio_rustls(1).build();
    pool
      .send_req_recv_res(ReqBuilder::get(UriRef::new(uri)).into_request())
      .await
      .unwrap()
      .msg_data
      .body
  };

  let mut csv = Csv::from_buf_read(BufReader::new(&*csv));
  let mut file_buffer = Vector::new();
  let mut line_buffer = Vector::new();
  let mut pem_buffer = Vector::new();
  let mut unique_certs = HashSet::new();

  file_buffer
    .extend_from_copyable_slice(
      b"use crate::x509::cv::cv_trust_anchor::CvTrustAnchorRaw;\n\n\
      /// Set of filtered certificates from CCADB suitable for scenarios related to TLS chain verification.\n",
    )
    .unwrap();
  file_buffer.extend_from_copyable_slice(b"#[rustfmt::skip]\n").unwrap();
  file_buffer
    .extend_from_copyable_slice(b"pub static CCADB: &[CvTrustAnchorRaw<'_,>] = &[\n")
    .unwrap();

  let _ = csv.next_elements(&mut line_buffer).unwrap().unwrap();
  while let Some(line) = csv.next_elements(&mut line_buffer).unwrap() {
    let cm = CertificateMetadata::from_line(line);
    if !cm.is_suitable_for_tls() {
      continue;
    }
    let hash: [u8; 64] = cm.sha256_fingerprint.try_into().unwrap();
    if !unique_certs.insert(hash) {
      panic!();
    }
    let cert = match Certificate::from_pem(&mut pem_buffer, cm.pem_info) {
      Ok(cert) => cert,
      Err(wtx::Error::X509Error(X509Error::InvalidSerialNumberBytes)) => continue,
      Err(err) => panic!("{err}"),
    };
    let ta = CvTrustAnchor::try_from(cert).unwrap();
    file_buffer.extend_from_copyable_slice(b"  (").unwrap();
    write_authority_key_identifier(&mut file_buffer, &ta);
    write_has_unknown_critical_extension(&mut file_buffer, &ta);
    write_is_self_signed(&mut file_buffer, &ta);
    write_key_usage(&mut file_buffer, &ta);
    write_name_constraints(&mut file_buffer, &ta);
    write_subject(&mut file_buffer, &ta);
    write_subject_key_identifier(&mut file_buffer, &ta);
    write_subject_public_key_info(&mut file_buffer, &ta);
    write_validity(&mut file_buffer, &ta);
    file_buffer.extend_from_copyable_slice(b"),\n").unwrap();
  }

  file_buffer.extend_from_copyable_slice(b"];\n").unwrap();

  fs::write("wtx/src/x509/ccadb.rs", &file_buffer).unwrap();
}

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
enum TrustBits {
  Websites,
  Email,
  Code,
  AllTrustBitsTurnedOff,
}

impl From<&[u8]> for TrustBits {
  fn from(value: &[u8]) -> Self {
    match value {
      b"Websites" => TrustBits::Websites,
      b"Email" => TrustBits::Email,
      b"Code" => TrustBits::Code,
      b"All Trust Bits Turned Off" => TrustBits::AllTrustBitsTurnedOff,
      _ => panic!(),
    }
  }
}

#[derive(Debug)]
struct CertificateMetadata<'any> {
  distrust_for_tls_after_date: &'any [u8],
  pem_info: &'any [u8],
  sha256_fingerprint: &'any [u8],
  trust_bits: &'any [u8],
}

impl<'any> CertificateMetadata<'any> {
  fn from_line(mut line: impl Iterator<Item = &'any [u8]>) -> Self {
    let _owner = line.next().unwrap();
    let _certificate_issuer_organization = line.next().unwrap();
    let _certificate_issuer_organizational_unit = line.next().unwrap();
    let _common_name_or_certificate_name = line.next().unwrap();
    let _certificate_serial_number = line.next().unwrap();
    let sha256_fingerprint = line.next().unwrap();
    let _subject_spki_sha256 = line.next().unwrap();
    let _valid_from_gmt = line.next().unwrap();
    let _valid_to_gmt = line.next().unwrap();
    let _public_key_algorithm = line.next().unwrap();
    let _signature_hash_algorithm = line.next().unwrap();
    let trust_bits = line.next().unwrap();
    let distrust_for_tls_after_date = line.next().unwrap();
    let _distrust_for_s_mime_after_date = line.next().unwrap();
    let _ev_policy_oid_s = line.next().unwrap();
    let _approval_bug = line.next().unwrap();
    let _nss_release_when_first_included = line.next().unwrap();
    let _firefox_release_when_first_included = line.next().unwrap();
    let _test_website_valid = line.next().unwrap();
    let _test_website_expired = line.next().unwrap();
    let _test_website_revoked = line.next().unwrap();
    let _mozilla_applied_constraints = line.next().unwrap();
    let _company_website = line.next().unwrap();
    let _geographic_focus = line.next().unwrap();
    let _certificate_policy_cp = line.next().unwrap();
    let _certification_practice_statement_cps = line.next().unwrap();
    let _certificate_practice_policy_statement_cp_cps = line.next().unwrap();
    let _markdown_asciidoc_cp_cps = line.next().unwrap();
    let _standard_audit = line.next().unwrap();
    let _netsec_audit = line.next().unwrap();
    let _tls_br_audit = line.next().unwrap();
    let _tls_evg_audit = line.next().unwrap();
    let _s_mime_br_audit = line.next().unwrap();
    let _audit_firm = line.next().unwrap();
    let _standard_audit_type = line.next().unwrap();
    let _standard_audit_statement_dt = line.next().unwrap();
    let pem_info = line.next().unwrap();
    Self {
      distrust_for_tls_after_date,
      pem_info: if let [b'\'', rest @ .., b'\''] | [b'\'', rest @ .., b'\'', b'\n'] = pem_info {
        rest
      } else {
        pem_info
      },
      sha256_fingerprint,
      trust_bits,
    }
  }

  fn distrust_for_tls_after_date(&self) -> Option<DateTime<Utc>> {
    if self.distrust_for_tls_after_date.is_empty() {
      return None;
    }
    let tokens = parse_bytes_into_tokens(b"%Y.%m.%d").unwrap();
    let date = Date::parse(self.distrust_for_tls_after_date, tokens).unwrap();
    Some(DateTime::new(date, Time::default(), Utc))
  }

  fn is_suitable_for_tls(&self) -> bool {
    if EXCLUDED_FINGERPRINTS.iter().any(|el| el.as_bytes() == self.sha256_fingerprint) {
      return false;
    }
    if !self.trust_bits().contains(&TrustBits::Websites) {
      return false;
    }
    let Some(distrust_for_tls_after_date) = self.distrust_for_tls_after_date() else {
      return true;
    };
    let days = Duration::from_days(398).unwrap();
    Instant::now_date_time(0).unwrap() < distrust_for_tls_after_date.add(days).unwrap()
  }

  fn trust_bits(&self) -> ArrayVectorU8<TrustBits, 4> {
    let iter = self.trust_bits.split(|el| *el == b';').map(TrustBits::from);
    let mut trust_bits = ArrayVectorU8::from_iterator(iter).unwrap();
    trust_bits.sort_unstable();
    let mut iter = trust_bits.windows(2);
    while let Some([a, b]) = iter.next() {
      assert!(a != b);
    }
    if trust_bits.contains(&TrustBits::AllTrustBitsTurnedOff) && trust_bits.len() > 1 {
      panic!();
    }
    trust_bits
  }
}

struct Bytes<'any>(&'any [u8]);

impl Debug for Bytes<'_> {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    if self.0.is_empty() {
      f.write_str("[]")
    } else {
      f.write_fmt(format_args!(
        "hexd!(b\"{}\")",
        HexDisplay(self.0, Some(HexEncMode::WithPrefixLower))
      ))
    }
  }
}

impl Display for Bytes<'_> {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    <Bytes as Debug>::fmt(self, f)
  }
}

fn write_authority_key_identifier(file_buffer: &mut Vector<u8>, ta: &CvTrustAnchor<'_>) {
  let Some(ki) = ta.authority_key_identifier().as_ref().and_then(|aki| aki.key_identifier.as_ref())
  else {
    file_buffer.extend_from_copyable_slice(b"None,").unwrap();
    return;
  };
  file_buffer.write_fmt(format_args!("Some({}),", Bytes(ki.bytes().as_inner().unwrap()))).unwrap();
}

fn write_has_unknown_critical_extension(file_buffer: &mut Vector<u8>, ta: &CvTrustAnchor<'_>) {
  file_buffer.write_fmt(format_args!("{},", ta.has_unknown_critical_extension())).unwrap();
}

fn write_is_self_signed(file_buffer: &mut Vector<u8>, ta: &CvTrustAnchor<'_>) {
  file_buffer.write_fmt(format_args!("{},", ta.is_self_signed())).unwrap();
}

fn write_key_usage(file_buffer: &mut Vector<u8>, ta: &CvTrustAnchor<'_>) {
  let Some(ku) = ta.key_usage() else {
    file_buffer.extend_from_copyable_slice(b"None,").unwrap();
    return;
  };
  let bytes = ku.bytes();
  file_buffer.write_fmt(format_args!("Some(({},{})),", bytes.0, bytes.1)).unwrap();
}

fn write_name_constraints(_: &mut Vector<u8>, ta: &CvTrustAnchor<'_>) {
  assert!(ta.name_constraints().is_none());
}

fn write_subject(file_buffer: &mut Vector<u8>, ta: &CvTrustAnchor<'_>) {
  file_buffer.write_fmt(format_args!("&{},", Bytes(ta.subject()))).unwrap();
}

fn write_subject_public_key_info(file_buffer: &mut Vector<u8>, ta: &CvTrustAnchor<'_>) {
  let spki = ta.subject_public_key_info();
  file_buffer.write_fmt(format_args!("(b\"{}\",", &spki.algorithm.algorithm)).unwrap();
  if let Some(params) = &spki.algorithm.parameters {
    file_buffer
      .write_fmt(format_args!("Some((&{},{})),", Bytes(params.data()), params.tag()))
      .unwrap();
  } else {
    file_buffer.extend_from_copyable_slice(b"None,").unwrap();
  }
  file_buffer.write_fmt(format_args!("&{}),", Bytes(spki.subject_public_key.bytes()))).unwrap();
}

fn write_subject_key_identifier(file_buffer: &mut Vector<u8>, ta: &CvTrustAnchor<'_>) {
  let Some(el) = ta.subject_key_identifier() else {
    file_buffer.extend_from_copyable_slice(b"None,").unwrap();
    return;
  };
  file_buffer
    .write_fmt(format_args!(
      "Some(({},{})),",
      Bytes(el.extension().key_identifier.bytes()),
      el.critical()
    ))
    .unwrap();
}

fn write_validity(file_buffer: &mut Vector<u8>, ta: &CvTrustAnchor<'_>) {
  let not_before = ta.validity().not_before.date_time().timestamp_secs_and_ns().0;
  let not_after = ta.validity().not_after.date_time().timestamp_secs_and_ns().0;
  file_buffer.write_fmt(format_args!("({},{})", not_before, not_after)).unwrap();
}
