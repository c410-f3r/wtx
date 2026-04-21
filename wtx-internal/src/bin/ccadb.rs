//! CCADB

use std::{
  fmt::{Debug, Display, Formatter, Write},
  fs,
  io::BufReader,
};
use wtx::{
  calendar::{Date, DateTime, Duration, Instant, Time, Utc, parse_bytes_into_tokens},
  codec::Csv,
  collection::{ArrayVectorU8, HashSet, Vector},
  http::{HttpClient, ReqBuilder, ReqResBuffer, client_pool::ClientPoolBuilder},
  misc::{Lease, UriRef},
  x509::{
    Certificate, CvTrustAnchor, RelativeDistinguishedName, SubjectPublicKeyInfo, X509Error,
    extensions::NameConstraints,
  },
};

static EXCLUDED_FINGERPRINTS: &[&str] =
  &["9A296A5182D1D451A2E37F439B74DAAFA267523329F90F9A0D2007C334E23C9A"];

#[tokio::main]
async fn main() {
  let csv = {
    let uri = "https://ccadb.my.salesforce-sites.com/mozilla/IncludedCACertificateReportPEMCSV";
    let rrb = ReqResBuffer::empty();
    let pool = ClientPoolBuilder::tokio_rustls(1).build();
    pool.send_req_recv_res(ReqBuilder::get(UriRef::new(uri)), rrb).await.unwrap().rrd.body
  };

  let mut counters = Counters::default();
  let mut csv = Csv::from_buf_read(BufReader::new(&*csv));
  let mut file_buffer = Vector::new();
  let mut line_buffer = Vector::new();
  let mut pem_buffer = Vector::new();
  let mut unique_certs = HashSet::new();

  file_buffer
    .extend_from_copyable_slice(
      b"/// Set of filtered certificates from CCADB suitable for scenarios related to TLS \
      chain verification.\n",
    )
    .unwrap();
  file_buffer.extend_from_copyable_slice(b"#[rustfmt::skip]\n").unwrap();
  file_buffer
    .extend_from_copyable_slice(
      b"pub static CCADB_TLS: &[(&[(&str,u8,&[u8])],(&str,u8,&[u8],&[u8]))] = &[\n",
    )
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
    let tav = CvTrustAnchor::try_from(cert).unwrap();
    let rdn_sequence = &tav.subject().lease().rdn_sequence;
    let subject_public_key_info = &tav.subject_public_key_info();
    counters.increment(&tav);
    file_buffer.extend_from_copyable_slice(b"  (").unwrap();
    write_subject(&mut file_buffer, rdn_sequence);
    write_subject_public_key_info(&mut file_buffer, subject_public_key_info);
    write_name_constraints(tav.name_constraints());
    file_buffer.extend_from_copyable_slice(b"),\n").unwrap();
  }

  file_buffer.extend_from_copyable_slice(b"];\n").unwrap();

  fs::write("wtx/src/x509/ccadb.rs", &file_buffer).unwrap();
  counters.print();
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
    f.write_str("&[")?;
    let mut iter = self.0.iter();
    if let Some(elem) = iter.next() {
      f.write_fmt(format_args!("{elem}"))?;
    }
    for elem in iter {
      f.write_fmt(format_args!(",{elem}"))?;
    }
    f.write_str("]")
  }
}

impl Display for Bytes<'_> {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    <Bytes as Debug>::fmt(self, f)
  }
}

#[derive(Debug, Default)]
struct Counters {
  max_algo_any: usize,
  max_es: usize,
  max_name_tv: usize,
  max_ps: usize,
  max_rdn: usize,
  max_rdn_fields: u8,
}

impl Counters {
  fn increment(&mut self, tav: &CvTrustAnchor<'_, '_>) {
    let rdn_sequence = &tav.subject().lease().rdn_sequence;
    let subject_public_key_info = &tav.subject_public_key_info();
    self.max_rdn = self.max_rdn.max(rdn_sequence.len());
    for fields in rdn_sequence {
      self.max_rdn_fields = self.max_rdn_fields.max(fields.entries.len());
      for field in &fields.entries {
        self.max_name_tv = self.max_name_tv.max(field.value.data().len());
      }
    }
    if let Some(parameters) = &subject_public_key_info.algorithm.parameters {
      self.max_algo_any = self.max_algo_any.max(parameters.data().len());
    }
    if let Some(nc) = tav.name_constraints() {
      self.max_es = self
        .max_es
        .max(nc.excluded_subtrees.as_ref().map(|el| el.len()).unwrap_or_default().into());
      self.max_ps = self
        .max_ps
        .max(nc.permitted_subtrees.as_ref().map(|el| el.len()).unwrap_or_default().into());
    }
  }

  fn print(&self) {
    let Self { max_algo_any, max_es, max_name_tv, max_ps, max_rdn, max_rdn_fields } = self;
    println!("Max ALGORITHM_ANY = {max_algo_any}");
    println!("Max NAME_TYPE_AND_VALUE = {max_name_tv}");
    println!("Max RDN_SEQUENCE = {max_rdn}");
    println!("Max RDN_SEQUENCE_FIELDS = {max_rdn_fields}");
    println!("Max EXCLUDED_SUBTREES = {max_es}");
    println!("Max INCLUDED_SUBTREES = {max_ps}");
  }
}

fn write_name_constraints(name_constraints: &Option<NameConstraints<'_>>) {
  let Some(elem) = name_constraints else {
    return;
  };
  assert_eq!(elem.excluded_subtrees.iter().count(), 0);
  assert_eq!(elem.permitted_subtrees.iter().count(), 0);
}

fn write_subject(file_buffer: &mut Vector<u8>, rdn_sequence: &[RelativeDistinguishedName<'_>]) {
  file_buffer.extend_from_copyable_slice(b"&[").unwrap();
  assert!(rdn_sequence.len() <= 1);
  if let Some(first_rdn) = rdn_sequence.first() {
    assert!(first_rdn.entries.len() <= 1);
    if let Some(first_name) = first_rdn.entries.first() {
      let name_oid = &*first_name.oid;
      let tag = first_name.value.tag();
      let any = Bytes(first_name.value.data());
      file_buffer.write_fmt(format_args!("(\"{name_oid}\",{tag},{any})")).unwrap();
    }
  }
  file_buffer.extend_from_copyable_slice(b"],").unwrap();
}

fn write_subject_public_key_info(
  file_buffer: &mut Vector<u8>,
  subject_public_key_info: &SubjectPublicKeyInfo<'_>,
) {
  let (params_bytes, params_tag) =
    if let Some(params) = &subject_public_key_info.algorithm.parameters {
      (Bytes(params.data()), params.tag())
    } else {
      (Bytes(&[]), 0)
    };
  let algorithm_oid = &subject_public_key_info.algorithm.algorithm;
  file_buffer.write_fmt(format_args!("(\"{algorithm_oid}\",{params_tag},{params_bytes},")).unwrap();
  let pk = subject_public_key_info.subject_public_key.bytes();
  file_buffer.write_fmt(format_args!("{})", Bytes(pk))).unwrap();
}
