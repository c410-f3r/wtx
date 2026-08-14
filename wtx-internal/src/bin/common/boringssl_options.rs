use core::fmt::Debug;
use std::{fs, process};
use wtx::{
  collections::Vector,
  tls::{HandshakePath, MaxFragmentLength, NamedGroup, ProtocolVersion, SignatureScheme},
};

const BOGO_NACK: i32 = 89;

#[derive(Debug)]
pub struct Options {
  pub certs_pem: Vector<String>,
  pub expect_curve_id: Option<NamedGroup>,
  pub expect_handshake_kind: Option<Vector<HandshakePath>>,
  pub expect_selected_credential: Option<isize>,
  pub export_keying_material: usize,
  pub export_keying_material_context: String,
  pub export_keying_material_context_used: bool,
  pub export_keying_material_label: String,
  pub export_traffic_secrets: bool,
  pub groups: Option<Vector<NamedGroup>>,
  pub has_default_cert: bool,             // Not a cfg
  pub has_seen_new_x509_credential: bool, // Not a cfg
  pub host_name: String,
  pub is_client: bool,
  pub keys_pem: Vector<String>,
  pub max_fragment: Option<MaxFragmentLength>,
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
  pub shim_id: u64,
  pub shut_down_after_handshake: bool,
  pub signing_prefs: Vector<SignatureScheme>,
  pub trusted_cert_file: String,
  pub use_sni: bool,
  pub verify_peer: bool,
  pub verify_prefs: Option<SignatureScheme>,
}

impl Default for Options {
  fn default() -> Self {
    Self {
      certs_pem: Vector::new(),
      expect_curve_id: None,
      expect_handshake_kind: None,
      export_keying_material: 0,
      expect_selected_credential: None,
      export_keying_material_context_used: false,
      export_keying_material_context: String::new(),
      export_keying_material_label: String::new(),
      export_traffic_secrets: false,
      groups: None,
      has_default_cert: false,
      has_seen_new_x509_credential: false,
      host_name: "example.com".into(),
      is_client: true,
      keys_pem: Vector::new(),
      max_fragment: None,
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
      shim_id: 0,
      shut_down_after_handshake: false,
      signing_prefs: Vector::new(),
      trusted_cert_file: String::new(),
      use_sni: false,
      verify_peer: false,
      verify_prefs: None,
    }
  }
}

pub struct OptionsIter<'any, A> {
  args: A,
  options: &'any mut Options,
}

impl<'any, A> OptionsIter<'any, A> {
  pub const fn new(args: A, options: &'any mut Options) -> Self {
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
    let has_arg = check_implemented_arguments(&arg, &mut self.args, self.options)
      || check_ignored_arguments(&arg);
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
    if self.options.has_default_cert {
      if self.options.certs_pem.len() > 1 {
        self.options.certs_pem.rotate_left(1);
      }
      if self.options.keys_pem.len() > 1 {
        self.options.keys_pem.rotate_left(1);
      }
      if self.options.signing_prefs.len() > 1 {
        self.options.signing_prefs.rotate_left(1);
      }
    }
  }
}

pub fn cert_pem_from_pem_file(path: &str) -> String {
  if path.is_empty() {
    return String::new();
  }
  fs::read_to_string(path).unwrap()
}

pub fn quit(why: &str) -> ! {
  eprintln!("{why}");
  process::exit(0)
}

pub const fn verify_cert(options: &Options) -> bool {
  options.verify_peer || options.offer_no_client_cas
}

fn check_ignored_arguments(arg: &str) -> bool {
  match arg {
    "-async" // Async suffixes don't interfere
    | "-ipv6" // IPv6 is automatically handled
    | "-no-legacy-server-connect" // TLS 1.3 is not legacy
    => {
      println!("Ignored: {arg}");
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
      if !options.has_seen_new_x509_credential {
        options.has_default_cert = true;
      }
      options.certs_pem.push(cert_pem_from_pem_file(&args.next().unwrap())).unwrap();
    }
    "-curves" => {
      let Ok(group) = NamedGroup::try_from(args.next().unwrap().parse::<u16>().unwrap()) else {
        return true;
      };
      options.groups.get_or_insert(Vector::new()).push(group).unwrap();
    }
    "-expect-curve-id" => {
      options.expect_curve_id =
        Some(NamedGroup::try_from(args.next().unwrap().parse::<u16>().unwrap()).unwrap());
    }
    "-expect-no-hrr" => {
      options.expect_handshake_kind = Some(Vector::from_iterator([HandshakePath::Full]).unwrap());
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
      options.keys_pem.push(pkc8_pem_from_pem_file(&args.next().unwrap())).unwrap();
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
    "-new-x509-credential" => {
      options.has_seen_new_x509_credential = true;
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
      let num: u16 = args.next().unwrap().parse().unwrap();
      options.signing_prefs.push(SignatureScheme::try_from(num).unwrap()).unwrap();
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
      let Ok(el) = SignatureScheme::try_from(args.next().unwrap().parse::<u16>().unwrap()) else {
        process::exit(BOGO_NACK);
      };
      options.verify_prefs = Some(el);
    }
    _ => return false,
  }
  true
}

fn check_unimplemented_arguments(arg: &str) {
  match arg {
    "-allow-unknown-alpn-protos"
    | "-check-close-notify" // Looks like something that should be used in the shim
    | "-cipher"
    | "-enable-ocsp-stapling" // Deprecated
    | "-expect-advertised-alpn"
    | "-expect-cipher-aes"
    | "-expect-client-ca-list"
    | "-expect-early-data-reason"
    | "-expect-no-session-id" // Resumption is not supported
    | "-expect-not-resumable-across-names"
    | "-expect-peer-cert-file"
    | "-expect-peer-verify-pref"
    | "-expect-session-miss" // Resumption is not supported
    | "-expect-ticket-supports-early-data"
    | "-expect-verify-result"
    | "-fail-cert-callback"
    | "-fips-202205"
    | "-install-ddos-callback"
    | "-key-shares"
    | "-no-key-shares"
    | "-no-op-extra-handshake"
    | "-no-ticket" // Resumption is not supported
    | "-peek-then-read"
    | "-psk"
    | "-renegotiate-freely"
    | "-request-server-padding"
    | "-require-any-client-certificate"
    | "-server-supported-groups-hint"
    | "-server-supports-padding"
    | "-srtp-profiles"
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

fn pkc8_pem_from_pem_file(path: &str) -> String {
  fs::read_to_string(path).unwrap()
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
