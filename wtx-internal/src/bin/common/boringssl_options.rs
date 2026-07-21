use core::fmt::Debug;
use std::{fs, process};
use wtx::{
  asn1::Pkcs8,
  codec::decode_base64_into_buffer,
  collections::Vector,
  crypto::SignatureTy,
  tls::{HandshakePath, MaxFragmentLength, NamedGroup, ProtocolVersion},
  x509::Certificate,
};

const BOGO_NACK: i32 = 89;

#[derive(Debug)]
pub struct Options {
  pub cert_der: Vector<u8>,
  pub check_close_notify: bool,
  pub expect_curve_id: Option<NamedGroup>,
  pub expect_handshake_kind: Option<Vector<HandshakePath>>,
  pub expect_quic_transport_params: Vector<u8>,
  pub expect_selected_credential: Option<isize>,
  pub export_keying_material: usize,
  pub export_keying_material_context_used: bool,
  pub export_keying_material_context: String,
  pub export_keying_material_label: String,
  pub export_traffic_secrets: bool,
  pub groups: Option<Vector<NamedGroup>>,
  pub host_name: String,
  pub is_client: bool,
  pub key_der: Vector<u8>,
  pub max_fragment: Option<MaxFragmentLength>,
  pub must_match_issuer: bool,
  pub offer_no_client_cas: bool,
  pub on_initial_expect_curve_id: Option<NamedGroup>,
  pub only_write_one_byte_after_handshake: bool,
  pub port: u16,
  pub protocols: Vector<String>,
  pub queue_data: bool,
  pub read_size: usize,
  pub reject_alpn: bool,
  pub resume_count: usize,
  pub select_empty_alpn: bool,
  pub send_key_update: bool,
  pub server_preference: bool,
  pub server_supported_group_hint: Option<NamedGroup>,
  pub shim_id: u64,
  pub shut_down_after_handshake: bool,
  pub signing_prefs: Option<u16>,
  pub trusted_cert_file: String,
  pub use_sni: bool,
  pub verify_peer: bool,
  pub wait_for_debugger: bool,
}

impl Default for Options {
  fn default() -> Self {
    Self {
      cert_der: Vector::default(),
      check_close_notify: false,
      expect_curve_id: None,
      expect_handshake_kind: None,
      expect_quic_transport_params: Vector::new(),
      export_keying_material: 0,
      expect_selected_credential: None,
      export_keying_material_context_used: false,
      export_keying_material_context: String::new(),
      export_keying_material_label: String::new(),
      export_traffic_secrets: false,
      groups: None,
      host_name: "example.com".into(),
      is_client: true,
      key_der: Vector::default(),
      max_fragment: None,
      must_match_issuer: false,
      offer_no_client_cas: false,
      on_initial_expect_curve_id: None,
      only_write_one_byte_after_handshake: false,
      port: 0,
      protocols: Vector::new(),
      queue_data: false,
      read_size: 512,
      reject_alpn: false,
      resume_count: 0,
      select_empty_alpn: false,
      send_key_update: false,
      server_preference: false,
      server_supported_group_hint: None,
      shim_id: 0,
      shut_down_after_handshake: false,
      signing_prefs: None,
      trusted_cert_file: String::new(),
      use_sni: false,
      verify_peer: false,
      wait_for_debugger: false,
    }
  }
}

pub struct OptionsIter<'any, A> {
  args: A,
  options: &'any mut Options,
}

impl<'any, A> OptionsIter<'any, A> {
  pub fn new(args: A, options: &'any mut Options) -> Self {
    Self { args, options }
  }
}

impl<'any, A> Iterator for OptionsIter<'any, A>
where
  A: Iterator<Item = String>,
{
  type Item = ();

  fn next(&mut self) -> Option<Self::Item> {
    let arg = self.args.next()?;
    check_unimplemented_arguments(&arg);
    let has_arg = check_implemented_arguments(&arg, &mut self.args, &mut self.options)
      || check_ignored_arguments(&arg, &mut self.args)
      || check_irrelevant_arguments(&arg);
    if has_arg {
      return Some(());
    }
    if &arg == "-is-handshaker-supported" {
      println!("No");
      process::exit(0);
    } else {
      println!("Unknown: {arg:?}");
      process::exit(1);
    }
  }
}

impl<A> Drop for OptionsIter<'_, A> {
  fn drop(&mut self) {
    if !self.options.is_client && verify_cert(self.options) {
      process::exit(BOGO_NACK);
    }
  }
}

pub fn cert_der_from_pem_file(path: &str) -> (Certificate<Vector<u8>>, Vector<u8>) {
  if path.is_empty() {
    return (Certificate::default(), Vector::new());
  }
  let mut buffer = Vector::new();
  let data = fs::read_to_string(path).unwrap();
  let params = Certificate::<Vector<u8>>::from_pem(&mut buffer, data.as_bytes()).unwrap();
  (params.0, params.1.try_into().unwrap())
}

fn check_ignored_arguments(arg: &str, args: &mut impl Iterator<Item = String>) -> bool {
  match arg {
    "-enable-ed25519"
    | "-enable-signed-cert-timestamps"
    | "-expect-no-session-id"
    | "-expect-secure-renegotiation"
    | "-expect-session-id"
    | "-expect-tls13-downgrade"
    | "-on-resume-expect-no-offer-early-data" => {
      println!("Ignored: {arg}");
    }
    "-application-settings"
    | "-expect-advertised-alpn"
    | "-expect-alpn"
    | "-expect-certificate-types"
    | "-expect-msg-callback"
    | "-expect-peer-signature-algorithm"
    | "-expect-peer-verify-pref"
    | "-expect-server-name"
    | "-expect-signed-cert-timestamps"
    | "-expect-ticket-age-skew"
    | "-handshaker-path"
    | "-max-cert-list"
    | "-on-initial-expect-alpn"
    | "-on-initial-expect-cipher"
    | "-on-initial-expect-early-data-reason"
    | "-on-resume-expect-alpn"
    | "-on-resume-expect-cipher"
    | "-on-retry-expect-alpn"
    | "-on-retry-expect-cipher" => {
      println!("Ignored: {arg} with {:?}", args.next());
    }
    _ => return false,
  }
  true
}

fn check_implemented_arguments(
  arg: &str,
  args: &mut impl Iterator<Item = String>,
  options: &mut Options,
) -> bool {
  match arg {
    "-advertise-alpn" => {
      options.protocols = split_protocols(&args.next().unwrap());
    }
    "-cert-file" => {
      options.cert_der = cert_der_from_pem_file(&args.next().unwrap()).1;
    }
    "-check-close-notify" => {
      options.check_close_notify = true;
    }
    "-curves" => {
      let group = NamedGroup::try_from(args.next().unwrap().parse::<u16>().unwrap()).unwrap();
      options.groups.get_or_insert(Vector::new()).push(group).unwrap();
    }
    "-expect-curve-id" => {
      options.expect_curve_id =
        Some(NamedGroup::try_from(args.next().unwrap().parse::<u16>().unwrap()).unwrap());
    }
    "-expect-no-hrr" => {
      options.expect_handshake_kind = Some(Vector::from_iterator([HandshakePath::Full]).unwrap());
    }
    "-expect-quic-transport-params" => {
      options.expect_quic_transport_params = decode_base64(args.next().unwrap().as_bytes());
    }
    "-expect-selected-credential" => {
      options.expect_selected_credential = Some(args.next().unwrap().parse().unwrap());
    }
    "-expect-version" => {
      let expect_version = args.next().unwrap().parse::<u16>().unwrap();
      if expect_version != 0 && expect_version < ProtocolVersion::Tls13.into() {
        process::exit(BOGO_NACK);
      }
    }
    "-export-context" => {
      options.export_keying_material_context = args.next().unwrap();
    }
    "-export-keying-material" => {
      options.export_keying_material = args.next().unwrap().parse::<usize>().unwrap();
    }
    "-export-label" => {
      options.export_keying_material_label = args.next().unwrap();
    }
    "-export-traffic-secrets" => {
      options.export_traffic_secrets = true;
    }
    "-host-name" => {
      options.host_name = args.next().unwrap();
      options.use_sni = true;
    }
    "-key-file" => {
      options.key_der = pkc8_from_pem_file(&args.next().unwrap());
    }
    "-key-update" => {
      options.send_key_update = true;
    }
    "-max-send-fragment" => {
      let max_fragment = args.next().unwrap().parse::<u16>().unwrap();
      options.max_fragment = Some(MaxFragmentLength::from_num(max_fragment).unwrap());
    }
    "-max-version" => {
      let value = args.next().unwrap().parse::<u16>().unwrap();
      if value < ProtocolVersion::Tls13.into() {
        process::exit(BOGO_NACK);
      }
    }
    "-min-version" => {
      let value = args.next().unwrap().parse::<u16>().unwrap();
      if value != u16::from(ProtocolVersion::Tls13) {
        process::exit(BOGO_NACK);
      }
    }
    "-must-match-issuer" => {
      options.must_match_issuer = true;
    }
    "-on-initial-expect-curve-id" => {
      options.on_initial_expect_curve_id =
        Some(NamedGroup::try_from(args.next().unwrap().parse::<u16>().unwrap()).unwrap());
    }
    "-port" => {
      options.port = args.next().unwrap().parse::<u16>().unwrap();
    }
    "-read-size" => {
      let rdsz = args.next().unwrap().parse::<usize>().unwrap();
      options.read_size = rdsz;
    }
    "-read-with-unfinished-write" => {
      options.queue_data = true;
      options.only_write_one_byte_after_handshake = true;
    }
    "-reject-alpn" => {
      options.reject_alpn = true;
    }
    "-resume-count" => {
      options.resume_count = args.next().unwrap().parse::<usize>().unwrap();
    }
    "-select-alpn" => {
      options.protocols.push(args.next().unwrap()).unwrap();
    }
    "-select-empty-alpn" => {
      options.select_empty_alpn = true;
    }
    "-server-preference" => {
      options.server_preference = true;
    }
    "-server-supported-groups-hint" => {
      let group = NamedGroup::try_from(args.next().unwrap().parse::<u16>().unwrap()).unwrap();
      options.server_supported_group_hint = Some(group);
    }
    "-server" => {
      options.is_client = false;
    }
    "-shim-id" => {
      options.shim_id = args.next().unwrap().parse::<u64>().unwrap();
    }
    "-shim-shuts-down" => {
      options.shut_down_after_handshake = true;
    }
    "-shim-writes-first" => {
      options.queue_data = true;
    }
    "-signing-prefs" => {
      options.signing_prefs = Some(args.next().unwrap().parse().unwrap());
    }
    "-tls13-variant" => {
      let variant = args.next().unwrap().parse::<u16>().unwrap();
      if variant != 1 {
        process::exit(BOGO_NACK);
      }
    }
    "-trust-cert" => {
      options.trusted_cert_file = args.next().unwrap();
    }
    "-use-export-context" => {
      options.export_keying_material_context_used = true;
    }
    "-verify-peer" => {
      options.verify_peer = true;
    }
    "-verify-prefs" => {
      let Ok(_el) = SignatureTy::try_from(args.next().unwrap().parse::<u16>().unwrap()) else {
        process::exit(BOGO_NACK);
      };
    }
    "-wait-for-debugger" => {
      if cfg!(unix) {
        options.wait_for_debugger = true;
      } else {
        panic!("-wait-for-debugger is not supported");
      }
    }
    _ => return false,
  }
  true
}

// Does not matter or it is already a default behavior
fn check_irrelevant_arguments(arg: &str) -> bool {
  match arg {
    "-async" // openssl
    | "-decline-alpn"
    | "-enable-all-curves"
    | "-expect-no-session"
    | "-expect-ticket-renewal"
    | "-forbid-renegotiation-after-handshake"
    | "-handoff"
    | "-implicit-handshake" // openssl
    | "-ipv6"
    | "-no-ssl3"
    | "-no-tls1"
    | "-no-tls11"
    | "-no-tls12"
    | "-permute-extensions"
    | "-renegotiate-ignore"
    | "-use-early-callback" // openssl
    | "-use-old-client-cert-callback" // openssl
    => true,
    _ => false
  }
}

fn check_unimplemented_arguments(arg: &str) {
  match arg {
    "-advertise-empty-npn"
    | "-advertise-npn"
    | "-allow-hint-mismatch"
    | "-allow-unknown-alpn-protos"
    | "-cipher"
    | "-cnsa-202407"
    | "-digest-prefs"
    | "-dtls"
    | "-enable-channel-id"
    | "-enable-client-custom-extension"
    | "-enable-early-data"
    | "-enable-grease"
    | "-enable-server-custom-extension"
    | "-expect-accept-early-data"
    | "-expect-channel-id"
    | "-expect-cipher-aes"
    | "-expect-client-ca-list"
    | "-expect-dhe-group-size"
    | "-expect-draft-downgrade"
    | "-expect-early-data-info"
    | "-expect-early-data-reason"
    | "-expect-hrr"
    | "-expect-not-resumable-across-names"
    | "-expect-peer-cert-file"
    | "-expect-reject-early-data"
    | "-expect-resumable-across-names"
    | "-expect-ticket-supports-early-data"
    | "-expect-verify-result"
    | "-export-early-keying-material"
    | "-fail-cert-callback"
    | "-fail-early-callback"
    | "-fallback-scsv"
    | "-false-start"
    | "-fips-202205"
    | "-handshake-twice"
    | "-ignore-tls13-downgrade"
    | "-install-ddos-callback"
    | "-key-shares"
    | "-no-key-shares"
    | "-no-op-extra-handshake"
    | "-no-rsa-pss-rsae-certs"
    | "-no-server-name-ack"
    | "-on-initial-expect-peer-cert-file"
    | "-on-initial-tls13-variant"
    | "-on-resume-early-write-after-message"
    | "-on-resume-enable-early-data"
    | "-on-resume-expect-accept-early-data"
    | "-on-resume-expect-early-data-reason"
    | "-on-resume-expect-reject-early-data-reason"
    | "-on-resume-expect-reject-early-data"
    | "-on-resume-export-early-keying-material"
    | "-on-resume-verify-fail"
    | "-on-retry-expect-early-data-reason"
    | "-psk"
    | "-renegotiate-freely"
    | "-require-any-client-certificate"
    | "-resumption-across-names-enabled"
    | "-retain-only-sha256-client-cert-initial"
    | "-reverify-on-resume"
    | "-select-empty-next-proto"
    | "-select-next-proto"
    | "-send-alert"
    | "-send-channel-id"
    | "-signed-cert-timestamps"
    | "-srtp-profiles"
    | "-ticket-key"
    | "-tls-unique"
    | "-use-client-ca-list"
    | "-use-custom-verify-callback"
    | "-use-exporter-between-reads"
    | "-use-null-client-ca-list"
    | "-use-ticket-aead-callback"
    | "-use-ticket-callback"
    | "-verify-fail"
    | "-wpa-202304" => {
      println!("Unimplemented: {arg}");
      process::exit(BOGO_NACK);
    }
    _ => {}
  }
}

fn decode_base64(data: &[u8]) -> Vector<u8> {
  let mut buffer = Vector::new();
  let _ = decode_base64_into_buffer(&mut buffer, data).unwrap();
  buffer
}

fn pkc8_from_pem_file(path: &str) -> Vector<u8> {
  let mut buffer = Vector::new();
  Pkcs8::<&[u8]>::from_pem(&mut buffer, fs::read_to_string(path).unwrap().as_bytes())
    .unwrap()
    .1
    .try_into()
    .unwrap()
}

fn split_protocols(protos: &str) -> Vector<String> {
  let mut ret = Vector::new();
  let mut idx = 0;
  while idx < protos.len() {
    let len: usize = protos.as_bytes().get(idx).copied().unwrap().into();
    let begin = idx.wrapping_add(1);
    let end = idx.wrapping_add(len).wrapping_add(1);
    let item = protos.get(begin..end).unwrap().into();
    ret.push(item).unwrap();
    idx = idx.wrapping_add(len.wrapping_add(1));
  }
  ret
}

pub(crate) fn verify_cert(options: &Options) -> bool {
  options.verify_peer || options.offer_no_client_cas
}
