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

#[path = "common/boringssl_options.rs"]
mod boringssl_options;

use crate::boringssl_options::OptionsIter;
use boringssl_options::{Options, cert_from_pem_file};
use std::{env, process};
use tokio::net::TcpStream;
use wtx::{
  calendar::Instant,
  collections::Vector,
  misc::Uri,
  rng::{ChaCha20, CryptoSeedableRng as _},
  stream::{StreamReader, StreamWriter as _},
  tls::{
    HandshakePath, NamedGroup, ServerName, TlsAcceptor, TlsConfig, TlsConnectorBuilder, TlsError,
    TlsMode, TlsModeUnverified, TlsModeVerified, TlsStream,
  },
  x509::CvTrustAnchor,
};

#[tokio::main]
async fn main() {
  wtx::misc::tracing_tree_init(None).unwrap();
  let mut options = Options::default();
  let mut options_iter = OptionsIter::new(env::args().skip(1), &mut options);
  while let Some(_) = options_iter.next() {}
  if options.is_client {
    if options.verify_peer {
      let tls_config = make_client_cfg::<TlsModeVerified>(&options);
      exec_tests::<_, true>(options, tls_config).await;
    } else {
      let tls_config = make_client_cfg::<TlsModeUnverified>(&options);
      exec_tests::<_, true>(options, tls_config).await;
    }
  } else {
    let tls_config = make_server_cfg::<TlsModeVerified>(&options);
    exec_tests::<_, false>(options, tls_config).await;
  }
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
}

async fn exec_tests<TM, const IS_CLIENT: bool>(options: Options, mut tls_config: TlsConfig<TM>)
where
  TM: TlsMode,
{
  for idx in 0..=options.resume_count {
    let uri = Uri::new(format!("localhost:{}", options.port));
    let rng = ChaCha20::from_std_random().unwrap();
    if IS_CLIENT {
      let fun = async {
        let mut connector = TlsConnectorBuilder::tokio(uri).build(&tls_config, rng).await?;
        connector.stream_mut().write_all(&options.shim_id.to_le_bytes()).await?;
        let mut rslt = connector.connect().await?;
        check_handshake_params(rslt.handshake_path, idx, rslt.named_group, &options);
        manage_after_handshake(&options, false, &mut rslt.tls_stream).await
      };
      handle_err(&options, fun.await);
    } else {
      let fun = async {
        let mut stream = TcpStream::connect(uri.hostname_with_implied_port()).await?;
        stream.write_all(&options.shim_id.to_le_bytes()).await?;
        let mut rslt = TlsAcceptor::new(&tls_config, rng, stream).accept().await?;
        check_handshake_params(rslt.handshake_path, idx, rslt.named_group, &options);
        manage_after_handshake(&options, false, &mut rslt.tls_stream).await
      };
      handle_err(&options, fun.await);
    }
    if options.is_client {
      tls_config = make_client_cfg(&options);
    } else {
      tls_config = make_server_cfg(&options);
    }
  }
}

fn handle_err(_opts: &Options, rslt: wtx::Result<()>) {
  let reason = match &rslt {
    Ok(_) => return,
    Err(wtx::Error::TlsError(err)) => match err {
      TlsError::UnsupportedTlsVersion(_) | TlsError::UnsupportedRecTlsVersion(_) => {
        ":WRONG_VERSION:"
      }
      TlsError::NoCertificate => ":PEER_DID_NOT_RETURN_A_CERTIFICATE:",
      _ => ":FIXME:",
    },
    Err(wtx::Error::TlsErrorFatal(TlsError::UnencryptedRecord, _)) => ":BAD_DECRYPT:",
    _ => ":FIXME:",
  };
  eprintln!("ERROR: {rslt:?}");
  quit(reason);
}

async fn manage_after_handshake<const IS_CLIENT: bool, TM>(
  options: &Options,
  mut _sent_message: bool,
  tls_stream: &mut TlsStream<TcpStream, TM, IS_CLIENT>,
) -> wtx::Result<()>
where
  TM: TlsMode,
{
  let mut quench_writes = false;
  let mut _sent_key_update = false;
  let mut sent_shutdown = false;

  if options.send_key_update && !_sent_key_update {
    tls_stream.refresh_traffic_keys().await?;
    _sent_key_update = true;
  }

  if (options.queue_data || options.only_write_one_byte_after_handshake) && !_sent_message {
    tls_stream.write_all(b"hello").await?;
    _sent_message = true;
    if options.only_write_one_byte_after_handshake {
      tls_stream.stream_mut().write_all(&[0]).await?;
      quench_writes = true;
    }
  }

  let mut buffer = Vector::from_iterator((0..options.read_size.max(1024)).map(|_| 0))?;

  loop {
    let len = match tls_stream.read(buffer.get_mut(..options.read_size).unwrap().into()).await {
      Ok(None) => {
        return Ok(());
      }
      Ok(Some(len)) => len.get(),
      Err(err) => return Err(err),
    };

    if options.shut_down_after_handshake && !sent_shutdown {
      tls_stream.send_close_notify().await?;
      sent_shutdown = true;
    }

    if quench_writes && len > 0 {
      quench_writes = false;
    }

    if len > 0 {
      for byte in buffer.get_mut(..len).unwrap() {
        *byte ^= 255;
      }
      tls_stream.write_all(buffer.get(..len).unwrap()).await?;
    }
  }
}

fn make_client_cfg<TM>(options: &Options) -> TlsConfig<TM>
where
  TM: TlsMode,
{
  let mut cfg = TlsConfig::new(TM::default(), Instant::now_date_time().unwrap());
  let (trust_anchor, _) = cert_from_pem_file(&options.trusted_cert_file);
  cfg
    .trust_anchors_mut()
    .push(CvTrustAnchor::from_certificate_ref(&trust_anchor).unwrap())
    .unwrap();
  *cfg.max_fragment_length_mut() = options.max_fragment;
  for protocol in &options.protocols {
    cfg
      .alpn_mut()
      .get_or_insert_default()
      .protocol_name_list
      .push(protocol.as_bytes().try_into().unwrap())
      .unwrap();
  }
  if options.use_sni {
    cfg
      .server_name_mut()
      .get_or_insert_default()
      .server_name_list
      .push(ServerName::from_name(options.host_name.as_str().try_into().unwrap()))
      .unwrap();
  }
  cfg
}

fn make_server_cfg<TM>(options: &Options) -> TlsConfig<TM>
where
  TM: TlsMode,
{
  let mut cfg = TlsConfig::from_keys_der(
    TM::default(),
    options.cert_der.as_slice(),
    &options.key_der.as_slice(),
  )
  .unwrap();
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

fn quit(why: &str) -> ! {
  eprintln!("{why}");
  process::exit(0)
}

fn _quit_err(why: &str) -> ! {
  eprintln!("{why}");
  process::exit(1)
}
