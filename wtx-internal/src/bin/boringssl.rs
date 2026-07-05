//! Based on <https://github.com/rustls/rustls/tree/main/bogo>

#![expect(
  clippy::exit,
  clippy::panic,
  clippy::print_stderr,
  clippy::print_stdout,
  clippy::struct_excessive_bools,
  clippy::too_many_lines,
  clippy::unwrap_used,
  clippy::use_debug,
  reason = "does not matter"
)]

use core::fmt::Debug;
use std::{env, process};
use tokio::net::{TcpListener, TcpStream};
use wtx::{
  calendar::Instant,
  codec::{Base64Alphabet, base64_decode, base64_decoded_len_ub, hex_decode},
  collections::{ShortBoxSliceU8, Vector},
  crypto::SignatureTy,
  rng::{ChaCha20, CryptoSeedableRng as _},
  stream::{StreamReader, StreamWriter as _},
  tls::{
    HandshakePath, MaxFragmentLength, NamedGroup, ProtocolVersion, TlsAcceptor, TlsConfig,
    TlsConnector, TlsModeVerified, TlsStream,
  },
  x509::{Certificate, CvTrustAnchor},
};

const BOGO_NACK: i32 = 89;

#[tokio::main]
async fn main() {
  let mut args = env::args().skip(1);
  let mut options = Options::new();
  loop {
    let is_finished = options.parse_one(&mut args);
    if is_finished {
      break;
    }
  }
  if options.is_client {
    let tls_config = make_client_cfg(&options);
    exec_tests::<true>(options, tls_config).await;
  } else {
    let tls_config = make_client_cfg(&options);
    exec_tests::<false>(options, tls_config).await;
  }
}

#[derive(Debug)]
struct Options {
  check_close_notify: bool,
  enable_early_data: bool,
  expect_accept_early_data: bool,
  expect_curve_id: Option<NamedGroup>,
  expect_handshake_kind: Option<Vector<HandshakePath>>,
  expect_handshake_kind_resumed: Option<Vector<HandshakePath>>,
  expect_quic_transport_params: Vector<u8>,
  expect_reject_early_data: bool,
  expect_ticket_supports_early_data: bool,
  expect_version: u16,
  export_keying_material: usize,
  export_keying_material_context_used: bool,
  export_keying_material_context: String,
  export_keying_material_label: String,
  export_traffic_secrets: bool,
  groups: Option<Vector<NamedGroup>>,
  host_name: String,
  is_client: bool,
  max_fragment: Option<MaxFragmentLength>,
  max_version: Option<ProtocolVersion>,
  min_version: Option<ProtocolVersion>,
  offer_no_client_cas: bool,
  on_initial_expect_curve_id: Option<NamedGroup>,
  on_resume_expect_curve_id: Option<NamedGroup>,
  only_write_one_byte_after_handshake: bool,
  only_write_one_byte_after_handshake_on_resume: bool,
  port: u16,
  protocols: Vector<String>,
  queue_data_on_resume: bool,
  queue_data: bool,
  queue_early_data_after_received_messages: Vector<usize>,
  quic_transport_params: Vector<u8>,
  read_size: usize,
  reject_alpn: bool,
  require_any_client_cert: bool,
  resume_with_tickets_disabled: bool,
  resumes: usize,
  resumption_delay: u32,
  root_hint_subjects: Vector<Vector<u8>>,
  send_key_update: bool,
  server_preference: bool,
  server_supported_group_hint: Option<NamedGroup>,
  shim_id: u64,
  shut_down_after_handshake: bool,
  support_tls13: bool,
  tickets: bool,
  trusted_cert_file: String,
  use_sni: bool,
  verify_peer: bool,
  wait_for_debugger: bool,
}

impl Options {
  fn new() -> Self {
    Self {
      check_close_notify: false,
      enable_early_data: false,
      expect_accept_early_data: false,
      expect_curve_id: None,
      expect_handshake_kind: None,
      expect_handshake_kind_resumed: Some(Vector::from_iterator([HandshakePath::Resumed]).unwrap()),
      expect_quic_transport_params: Vector::new(),
      expect_reject_early_data: false,
      expect_ticket_supports_early_data: false,
      expect_version: 0,
      export_keying_material: 0,
      export_keying_material_context_used: false,
      export_keying_material_context: String::new(),
      export_keying_material_label: String::new(),
      export_traffic_secrets: false,
      groups: None,
      host_name: "example.com".into(),
      is_client: true,
      max_fragment: None,
      max_version: None,
      min_version: None,
      offer_no_client_cas: false,
      on_initial_expect_curve_id: None,
      on_resume_expect_curve_id: None,
      only_write_one_byte_after_handshake_on_resume: false,
      only_write_one_byte_after_handshake: false,
      port: 0,
      protocols: Vector::new(),
      queue_data_on_resume: false,
      queue_data: false,
      queue_early_data_after_received_messages: Vector::new(),
      quic_transport_params: Vector::new(),
      read_size: 512,
      reject_alpn: false,
      require_any_client_cert: false,
      resume_with_tickets_disabled: false,
      resumes: 0,
      resumption_delay: 0,
      root_hint_subjects: Vector::new(),
      send_key_update: false,
      server_preference: false,
      server_supported_group_hint: None,
      shim_id: 0,
      shut_down_after_handshake: false,
      support_tls13: true,
      tickets: true,
      trusted_cert_file: String::new(),
      use_sni: false,
      verify_peer: false,
      wait_for_debugger: false,
    }
  }

  fn parse_one(&mut self, mut args: impl Iterator<Item = String>) -> bool {
    let Some(arg) = args.next() else {
      return true;
    };
    match arg.as_str() {
      "-advertise-alpn" => {
        self.protocols = split_protocols(&args.next().unwrap());
      }
      "-cert-file" => {
        print!("TODO-0");
      }
      "-check-close-notify" => {
        self.check_close_notify = true;
      }
      "-curves" => {
        let group = NamedGroup::try_from(args.next().unwrap().parse::<u16>().unwrap()).unwrap();
        self.groups.get_or_insert(Vector::new()).push(group).unwrap();
      }
      "-enable-early-data" => {
        self.tickets = false;
        self.enable_early_data = true;
      }
      "-expect-accept-early-data" | "-on-resume-expect-accept-early-data" => {
        self.expect_accept_early_data = true;
      }
      "-expect-curve-id" => {
        self.expect_curve_id =
          Some(NamedGroup::try_from(args.next().unwrap().parse::<u16>().unwrap()).unwrap());
      }
      "-expect-early-data-reason" | "-on-resume-expect-reject-early-data-reason" => {
        let reason = args.next().unwrap();
        match reason.as_str() {
          "disabled" | "protocol_version" => {
            self.expect_reject_early_data = true;
          }
          _ => {
            println!("NYI early data reason: {reason}");
            process::exit(1);
          }
        }
      }
      "-expect-hrr" => {
        self.expect_handshake_kind =
          Some(Vector::from_iterator([HandshakePath::FullWithHelloRetryRequest]).unwrap());
        self.expect_handshake_kind_resumed =
          Some(Vector::from_iterator([HandshakePath::ResumedWithHelloRetryRequest]).unwrap());
      }
      "-expect-no-hrr" => {
        self.expect_handshake_kind = Some(Vector::from_iterator([HandshakePath::Full]).unwrap());
      }
      "-expect-quic-transport-params" => {
        self.expect_quic_transport_params = decode_base64(args.next().unwrap().as_bytes());
      }
      "-expect-reject-early-data" | "-on-resume-expect-reject-early-data" => {
        self.expect_reject_early_data = true;
      }
      "-expect-selected-credential" => {
        print!("TODO-1");
      }
      "-expect-session-miss" => {
        self.expect_handshake_kind_resumed = Some(
          Vector::from_iterator([
            HandshakePath::Full,
            HandshakePath::FullWithHelloRetryRequest,
          ])
          .unwrap(),
        );
      }
      "-expect-ticket-supports-early-data" => {
        self.expect_ticket_supports_early_data = true;
      }
      "-expect-version" => {
        self.expect_version = args.next().unwrap().parse::<u16>().unwrap();
      }
      "-export-context" => {
        self.export_keying_material_context = args.next().unwrap();
      }
      "-export-keying-material" => {
        self.export_keying_material = args.next().unwrap().parse::<usize>().unwrap();
      }
      "-export-label" => {
        self.export_keying_material_label = args.next().unwrap();
      }
      "-export-traffic-secrets" => {
        self.export_traffic_secrets = true;
      }
      "-fips-202205" => {
        println!("Not a FIPS build");
        process::exit(BOGO_NACK);
      }
      "-host-name" => {
        self.host_name = args.next().unwrap();
        self.use_sni = true;
      }
      "-key-file" => {
        print!("TODO-2");
      }
      "-key-update" => {
        self.send_key_update = true;
      }
      "-max-send-fragment" => {
        let max_fragment = args.next().unwrap().parse::<u8>().unwrap();
        self.max_fragment = Some(max_fragment.try_into().unwrap());
      }
      "-max-version" => {
        let max = args.next().unwrap().parse::<u16>().unwrap();
        self.max_version = Some(max.try_into().unwrap());
      }
      "-min-version" => {
        let min = args.next().unwrap().parse::<u16>().unwrap();
        self.min_version = Some(min.try_into().unwrap());
      }
      "-must-match-issuer" => {
        print!("TODO-3");
      }
      "-no-ticket" => {
        self.tickets = false;
      }
      "-no-tls13" => {
        self.support_tls13 = false;
      }
      "-on-initial-expect-curve-id" => {
        self.on_initial_expect_curve_id =
          Some(NamedGroup::try_from(args.next().unwrap().parse::<u16>().unwrap()).unwrap());
      }
      "-on-resume-early-write-after-message" => {
        self.queue_early_data_after_received_messages = match args.next().unwrap().parse::<u8>().unwrap() {
          2 => Vector::from_iterator([5 + 128 + 5 + 32]).unwrap(),
          8 => Vector::from_iterator([5 + 128 + 5 + 32, 5 + 64]).unwrap(),
          _ => {
            panic!("unhandled -on-resume-early-write-after-message");
          }
        };
        self.queue_data_on_resume = true;
      }
      "-on-resume-expect-curve-id" => {
        self.on_resume_expect_curve_id =
          Some(NamedGroup::try_from(args.next().unwrap().parse::<u16>().unwrap()).unwrap());
      }
      "-on-resume-no-ticket" => {
        self.resume_with_tickets_disabled = true;
      }
      "-on-resume-read-with-unfinished-write" => {
        self.queue_data_on_resume = true;
        self.only_write_one_byte_after_handshake_on_resume = true;
      }
      "-on-resume-shim-writes-first" => {
        self.queue_data_on_resume = true;
      }
      "-on-retry-expect-early-data-reason" | "-on-resume-expect-early-data-reason" => {
        if args.next().unwrap() == "hello_retry_request" {
          self.expect_handshake_kind_resumed =
            Some(Vector::from_iterator([HandshakePath::ResumedWithHelloRetryRequest]).unwrap());
        }
      }
      "-port" => {
        self.port = args.next().unwrap().parse::<u16>().unwrap();
      }
      "-quic-transport-params" => {
        self.quic_transport_params = decode_base64(args.next().unwrap().as_bytes());
      }
      "-read-size" => {
        let rdsz = args.next().unwrap().parse::<usize>().unwrap();
        self.read_size = rdsz;
      }
      "-read-with-unfinished-write" => {
        self.queue_data = true;
        self.only_write_one_byte_after_handshake = true;
      }
      "-reject-alpn" => {
        self.reject_alpn = true;
      }
      "-require-any-client-certificate" => {
        self.require_any_client_cert = true;
      }
      "-resume-count" => {
        self.resumes = args.next().unwrap().parse::<usize>().unwrap();
      }
      "-resumption-delay" => {
        self.resumption_delay = args.next().unwrap().parse::<u32>().unwrap();
      }
      "-select-alpn" => {
        self.protocols.push(args.next().unwrap()).unwrap();
      }
      "-server-preference" => {
        self.server_preference = true;
      }
      "-server-supported-groups-hint" => {
        let group = NamedGroup::try_from(args.next().unwrap().parse::<u16>().unwrap()).unwrap();
        self.server_supported_group_hint = Some(group);
      }
      "-server" => {
        self.is_client = false;
      }
      "-shim-id" => {
        self.shim_id = args.next().unwrap().parse::<u64>().unwrap();
      }
      "-shim-shuts-down" => {
        self.shut_down_after_handshake = true;
      }
      "-shim-writes-first" => {
        self.queue_data = true;
      }
      "-signing-prefs" => {
        print!("TODO-4");
      }
      "-tls13-variant" => {
        let variant = args.next().unwrap().parse::<u16>().unwrap();
        if variant != 1 {
          println!("NYI TLS1.3 variant selection: {arg:?} {variant:?}");
          process::exit(BOGO_NACK);
        }
      }
      "-trust-cert" => {
          self.trusted_cert_file = args.next().unwrap();
      }
      "-use-client-ca-list" => match args.next().unwrap().as_ref() {
        "<EMPTY>" | "<NULL>" => {
          self.root_hint_subjects = Vector::new();
        }
        list => {
          self.root_hint_subjects = Vector::from_iterator(list
            .split(',')
            .map(decode_hex))
            .unwrap();
        }
      },
      "-use-export-context" => {
        self.export_keying_material_context_used = true;
      }
      "-use-null-client-ca-list" => {
        self.offer_no_client_cas = true;
      }
      "-verify-peer" => {
        self.verify_peer = true;
      }
      "-verify-prefs" => {
        let _ = lookup_scheme(args.next().unwrap().parse::<u16>().unwrap());
      }
      "-wait-for-debugger" => {
        if cfg!(unix) {
          self.wait_for_debugger = true;
        } else {
          panic!("-wait-for-debugger is not supported");
        }
      }

      "-enable-ed25519"
      | "-enable-signed-cert-timestamps"
      | "-expect-no-session-id"
      | "-expect-secure-renegotiation"
      | "-expect-session-id"
      | "-expect-tls13-downgrade"
      | "-on-resume-expect-no-offer-early-data" => {
        println!("not checking {arg}; NYI");
      }

      // defaults:
      "-decline-alpn"
      | "-enable-all-curves"
      | "-expect-no-session"
      | "-expect-ticket-renewal"
      | "-forbid-renegotiation-after-handshake"
      | "-handoff"
      | "-ipv6"
      | "-no-ssl3"
      | "-no-tls1"
      | "-no-tls11"
      | "-no-tls12"
      | "-permute-extensions"
      | "-renegotiate-ignore"

      // internal openssl details:
      | "-async"
      | "-implicit-handshake"
      | "-use-old-client-cert-callback"
      | "-use-early-callback" => {}

      // Not implemented things
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
      | "-enable-grease"
      | "-enable-server-custom-extension"
      | "-expect-channel-id"
      | "-expect-cipher-aes"
      | "-expect-dhe-group-size"
      | "-expect-draft-downgrade"
      | "-expect-early-data-info"
      | "-expect-not-resumable-across-names"
      | "-expect-peer-cert-file"
      | "-expect-resumable-across-names"
      | "-expect-verify-result"
      | "-export-early-keying-material"
      | "-fail-cert-callback"
      | "-fail-early-callback"
      | "-fallback-scsv"
      | "-false-start"
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
      | "-on-resume-enable-early-data"
      | "-on-resume-export-early-keying-material"
      | "-on-resume-verify-fail"
      | "-psk"
      | "-renegotiate-freely"
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
      | "-use-custom-verify-callback"
      | "-use-exporter-between-reads"
      | "-use-ticket-aead-callback"
      | "-use-ticket-callback"
      | "-verify-fail"
      | "-wpa-202304" => {
        println!("NYI option {arg:?}");
        process::exit(BOGO_NACK);
      }

      "-application-settings"
      | "-expect-advertised-alpn"
      | "-expect-alpn"
      | "-expect-certificate-types"
      | "-expect-client-ca-list"
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
        println!("not checking {arg} {:?}; NYI", args.next());
      }

      "-is-handshaker-supported" => {
        println!("handshaker");
        process::exit(1);
      }

      _ => {
        println!("unknown option {arg:?}");
        process::exit(1);
      }
    }
    false
  }
}

fn cert_from_pem_file(_: &str) -> (Certificate<ShortBoxSliceU8<u8>>, &[u8]) {
  todo!()
  //let mut buffer = Vector::new();
  //Certificate::from_pem(&mut buffer, fs::read_to_string(file).unwrap().as_bytes()).unwrap()
}

fn check_handshake_params(
  handshake_path: HandshakePath,
  idx: usize,
  named_group: NamedGroup,
  options: &Options,
) {
  if let Some(elems) = options.expect_handshake_kind.as_ref() {
    assert!(elems.contains(&handshake_path));
  }
  if let Some(elem) = options.expect_curve_id {
    let actual = named_group;
    assert_eq!(elem, actual);
  }
  if let Some(elem) = &options.on_initial_expect_curve_id
    && idx == 0
  {
    assert_eq!(handshake_path, HandshakePath::Full);
    assert_eq!(named_group, *elem);
  }
  if let Some(elem) = &options.on_resume_expect_curve_id
    && idx > 0
  {
    assert!(matches!(
      handshake_path,
      HandshakePath::Resumed | HandshakePath::ResumedWithHelloRetryRequest
    ));
    assert_eq!(named_group, *elem);
  }
}

fn decode_base64(data: &[u8]) -> Vector<u8> {
  let len = base64_decoded_len_ub(data.len());
  let mut rslt = Vector::from_iterator((0..len).map(|_| 0u8)).unwrap();
  let written = base64_decode(Base64Alphabet::Standard, data, &mut rslt).unwrap().len();
  rslt.truncate(written);
  rslt
}

fn decode_hex(hex: &str) -> Vector<u8> {
  let mut vec = Vector::from_iterator((0..hex.len()).map(|_| 0u8)).unwrap();
  let _ = hex_decode(hex.as_bytes(), &mut vec).unwrap();
  vec
}

async fn exec_tests<const IS_CLIENT: bool>(
  mut options: Options,
  mut tls_config: TlsConfig<TlsModeVerified>,
) {
  for idx in 0..=options.resumes {
    let addr = (options.host_name.as_str(), options.port);
    let rng = ChaCha20::from_std_random().unwrap();
    if IS_CLIENT {
      let mut stream = TcpStream::connect(addr).await.unwrap();
      stream.write_all(&options.shim_id.to_le_bytes()).await.unwrap();
      let mut rslt = TlsConnector::new(&tls_config, rng, stream).connect().await.unwrap();
      check_handshake_params(rslt.handshake_path, idx, rslt.named_group, &options);
      manage_after_handshake(&options, false, &mut rslt.tls_stream).await;
    } else {
      let listener = TcpListener::bind(addr).await.unwrap();
      let mut stream = listener.accept().await.unwrap();
      stream.0.write_all(&options.shim_id.to_le_bytes()).await.unwrap();
      let mut rslt = TlsAcceptor::new(&tls_config, rng, stream.0).accept().await.unwrap();
      check_handshake_params(rslt.handshake_path, idx, rslt.named_group, &options);
      manage_after_handshake(&options, false, &mut rslt.tls_stream).await;
    }
    if options.resume_with_tickets_disabled {
      options.tickets = false;
      if options.is_client {
        tls_config = make_client_cfg(&options);
      } else {
        tls_config = make_server_cfg(&options);
      }
    }
    options.expect_handshake_kind = options.expect_handshake_kind_resumed.clone();
  }
}

fn _handle_err(_opts: &Options, _err: wtx::Error) -> ! {
  panic!()
}

fn lookup_scheme(scheme: u16) -> SignatureTy {
  match scheme {
    0x0403 => SignatureTy::EcdsaSecp256r1Sha256,
    0x0503 => SignatureTy::EcdsaSecp384r1Sha384,
    0x0804 => SignatureTy::RsaPssRsaeSha256,
    0x0805 => SignatureTy::RsaPssRsaeSha384,
    0x0807 => SignatureTy::Ed25519,
    _ => {
      process::exit(BOGO_NACK);
    }
  }
}

async fn manage_after_handshake<const IS_CLIENT: bool>(
  options: &Options,
  mut _sent_message: bool,
  tls_stream: &mut TlsStream<TcpStream, TlsModeVerified, IS_CLIENT>,
) {
  let mut quench_writes = false;
  let mut _sent_key_update = false;
  let mut sent_shutdown = false;

  if options.send_key_update && !_sent_key_update {
    tls_stream.refresh_traffic_keys().await.unwrap();
    _sent_key_update = true;
  }

  if options.only_write_one_byte_after_handshake && !_sent_message {
    tls_stream.write_all(b"hello").await.unwrap();
    _sent_message = true;
    tls_stream.write_all(&[0]).await.unwrap();
    quench_writes = true;
  }

  loop {
    let mut buf = [0u8; 1024];
    let len =
      match tls_stream.stream_mut().read(buf.get_mut(..options.read_size).unwrap().into()).await {
        Ok(None) => {
          if options.check_close_notify {
            println!("close notify ok");
          }
          println!("EOF (tls)");
          return;
        }
        Ok(Some(len)) => len.get(),
        //Err(err) if err.kind() == io::ErrorKind::WouldBlock => 0,
        //Err(err) if err.kind() == io::ErrorKind::UnexpectedEof => {
        //  if options.check_close_notify {
        //    quit_err(":CLOSE_WITHOUT_CLOSE_NOTIFY:");
        //  }
        //  return;
        //}
        Err(err) => panic!("unhandled read error {err:?}"),
      };

    if options.shut_down_after_handshake && !sent_shutdown {
      tls_stream.send_close_notify().await.unwrap();
      sent_shutdown = true;
    }

    if quench_writes && len > 0 {
      quench_writes = false;
    }

    if len > 0 {
      for byte in &mut buf {
        *byte ^= 255;
      }
      let slice = buf.get(..len).unwrap();
      tls_stream.stream_mut().write_all(slice).await.unwrap();
    }
  }
}

fn make_client_cfg(options: &Options) -> TlsConfig<TlsModeVerified> {
  let mut cfg = TlsConfig::new(TlsModeVerified::default(), Instant::now_date_time().unwrap());
  if options.verify_peer || options.offer_no_client_cas || options.require_any_client_cert {
    let (trust_anchor, _) = cert_from_pem_file(&options.trusted_cert_file);
    cfg
      .trust_anchors_mut()
      .push(CvTrustAnchor::from_certificate_ref(&trust_anchor).unwrap())
      .unwrap();
  }
  // cfg.enable_sni = options.use_sni;
  *cfg.max_fragment_length_mut() = options.max_fragment;
  for protocol in &options.protocols {
    cfg
      .alpn_mut()
      .get_or_insert_default()
      .protocol_name_list
      .push(protocol.as_bytes().try_into().unwrap())
      .unwrap();
  }
  cfg
}

fn make_server_cfg(options: &Options) -> TlsConfig<TlsModeVerified> {
  let mut cfg = TlsConfig::new(TlsModeVerified::default(), Instant::now_date_time().unwrap());
  if options.verify_peer || options.offer_no_client_cas || options.require_any_client_cert {
    let (trust_anchor, _) = cert_from_pem_file(&options.trusted_cert_file);
    cfg
      .trust_anchors_mut()
      .push(CvTrustAnchor::from_certificate_ref(&trust_anchor).unwrap())
      .unwrap();
  }
  *cfg.max_fragment_length_mut() = options.max_fragment;
  for protocol in &options.protocols {
    cfg
      .alpn_mut()
      .get_or_insert_default()
      .protocol_name_list
      .push(protocol.as_bytes().try_into().unwrap())
      .unwrap();
  }
  if options.reject_alpn {
    cfg
      .alpn_mut()
      .get_or_insert_default()
      .protocol_name_list
      .push("invalid".as_bytes().try_into().unwrap())
      .unwrap();
  }
  cfg
}

fn _quit(why: &str) -> ! {
  eprintln!("{why}");
  process::exit(0)
}

fn _quit_err(why: &str) -> ! {
  eprintln!("{why}");
  process::exit(1)
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
