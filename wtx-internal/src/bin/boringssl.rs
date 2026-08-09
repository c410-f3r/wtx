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

use crate::boringssl_options::{OptionsIter, cert_pem_from_pem_file};
use boringssl_options::Options;
use std::{env, process};
use tokio::net::TcpStream;
use wtx::{
  collections::Vector,
  net::{StreamReader, StreamWriter as _, Uri},
  rng::{ChaCha20, CryptoSeedableRng as _},
  tls::{
    AlertDescription, Alpn, HandshakePath, NamedGroup, ServerName, SkCtx, TlsAcceptor, TlsConfig,
    TlsConnectorBuilder, TlsCtx, TlsCtxSk, TlsCtxSkLoader, TlsError, TlsStream, UnverifiedCtx,
  },
};

#[tokio::main]
async fn main() {
  wtx::misc::tracing_tree_init(None).unwrap();
  let mut options = Options::default();
  for _ in OptionsIter::new(env::args().skip(1), &mut options) {}
  if options.is_client {
    if boringssl_options::verify_cert(&options) {
      let tls_config = make_client_cfg(SkCtx::default(), &options);
      exec_tests::<_, true>(options, tls_config).await;
    } else {
      let tls_config = make_client_cfg(UnverifiedCtx::default(), &options);
      exec_tests::<_, true>(options, tls_config).await;
    }
  } else {
    let tls_config = make_server_cfg(&options);
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

async fn exec_tests<TCX, const IS_CLIENT: bool>(options: Options, tls_config: TlsConfig<TCX>)
where
  TCX: TlsCtxSk,
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
  }
}

fn handle_err(_opts: &Options, rslt: wtx::Result<()>) {
  let reason = match &rslt {
    Ok(_) => return,
    Err(wtx::Error::TlsError(err)) => match err {
      // Client
      TlsError::MissingKeyShares => ":MISSING_KEY_SHARE:",

      TlsError::AbortedHandshake(alert)
        if alert.description() == AlertDescription::HandshakeFailure =>
      {
        ":HANDSHAKE_FAILURE_ON_CLIENT_HELLO:"
      }
      TlsError::BadSignature => ":BAD_SIGNATURE:",
      TlsError::DigestCheckFailed => ":DIGEST_CHECK_FAILED:",
      TlsError::DuplicatedKeyShares => ":DUPLICATE_KEY_SHARE:",
      TlsError::InvalidAesData => ":BAD_DECRYPT:",
      TlsError::InvalidCertificateRequest => ":DECODE_ERROR:",
      TlsError::MismatchedCertificatePkAndSignature => ":WRONG_SIGNATURE_TYPE:",
      TlsError::MissingDigitalSignatureInKeyUsage => ":KEY_USAGE_BIT_INCORRECT:",
      TlsError::MissingSignatureAlgorithms => ":NO_COMMON_SIGNATURE_ALGORITHMS:",
      TlsError::NoCertificate => ":PEER_DID_NOT_RETURN_A_CERTIFICATE:",
      TlsError::SecretMismatch => ":WRONG_CURVE:",
      TlsError::TrailingDataInExtension => ":DECODE_ERROR:",
      TlsError::UnexpectedAfterHandshakeOuterRecord => ":INVALID_OUTER_RECORD_TYPE:",
      TlsError::UnknownNamedGroup => ":WRONG_CURVE:",
      TlsError::UnknownProtocolVersion => ":WRONG_VERSION_NUMBER:",
      TlsError::UnknownSignatureScheme => ":WRONG_SIGNATURE_TYPE:",
      TlsError::UnsupportedCipherSuite => ":WRONG_CIPHER_RETURNED:",
      TlsError::UnsupportedExtension => ":ERROR_PARSING_EXTENSION:",
      _ => ":FIXME:",
    },
    Err(wtx::Error::TlsErrorReply(err, _)) => match err {
      TlsError::ClientExpectedFinished => ":UNEXPECTED_MESSAGE:",
      TlsError::DiffieHellmanError => ":WRONG_CURVE:",
      TlsError::EmptyCertificateAuthorities => ":ERROR_PARSING_EXTENSION:",
      TlsError::EmptyNegotiatedAlpnClient => ":PARSE_TLSEXT:",
      TlsError::EmptyNegotiatedAlpnServer => ":INVALID_ALPN_PROTOCOL:",
      TlsError::EmptyNewSessionTicket => ":DECODE_ERROR:",
      TlsError::ExcessHandshakeData(_) => ":EXCESS_HANDSHAKE_DATA:",
      TlsError::IncompleteHandshake => ":UNEXPECTED_MESSAGE:",
      TlsError::InvalidExtensionTy => ":UNEXPECTED_EXTENSION:",
      TlsError::InvalidLegacyCompressionMethod => ":DECODE_ERROR:",
      TlsError::InvalidLegacyCompressionMethods => ":INVALID_COMPRESSION_LIST:",
      TlsError::InvalidLegacySessionId => ":DECODE_ERROR:",
      TlsError::InvalidNegotiatedServerName => ":UNEXPECTED_EXTENSION:",
      TlsError::InvalidServerNameList => ":ERROR_PARSING_EXTENSION:",
      TlsError::InvalidX509 => ":CANNOT_PARSE_LEAF_CERT:",
      TlsError::MismatchedExtension => ":UNEXPECTED_EXTENSION:",
      TlsError::MismatchedNegotiatedAlpnClient => ":INVALID_ALPN_PROTOCOL:",
      TlsError::MismatchedNegotiatedAlpnServer => ":NO_APPLICATION_PROTOCOL:",
      TlsError::MissingKeyShares => ":MISSING_KEY_SHARE:",
      TlsError::MissingSupportedGroups => ":NO_SHARED_GROUP:",
      TlsError::PostHandshakeDecError(handshake_ty) => {
        if handshake_ty.is_finished() {
          ":DIGEST_CHECK_FAILED:"
        } else {
          ":DECODE_ERROR:"
        }
      }
      TlsError::PreHandshakeDecError => ":EXCESS_HANDSHAKE_DATA:",
      TlsError::ReceivedRecordIsTooLarge => ":DATA_LENGTH_TOO_LONG:",
      TlsError::ServerHasNoCompatibleSignatureScheme => ":NO_COMMON_SIGNATURE_ALGORITHMS:",
      TlsError::ServerHasNoCompatibleKeyShare => ":UNEXPECTED_MESSAGE:",
      TlsError::TooManyKeyUpdates => ":TOO_MANY_KEY_UPDATES:",
      TlsError::TooManyWarningAlerts => ":TOO_MANY_WARNING_ALERTS:",
      TlsError::TrailingDataInExtension => ":ERROR_PARSING_EXTENSION:",
      TlsError::UnencryptedRecord => ":BAD_DECRYPT:",
      TlsError::UnexpectedAfterHandshakeInnerRecord => ":UNEXPECTED_RECORD:",
      TlsError::UnknownHandshakeTy(_) => ":UNEXPECTED_MESSAGE:",
      TlsError::UnknownRecordContentType => ":BAD_DECRYPT:",
      TlsError::UnofferedExtension => ":UNEXPECTED_EXTENSION:",
      TlsError::WrongAlert => ":BAD_ALERT:",
      _ => ":FIXME:",
    },
    _ => ":FIXME:",
  };
  eprintln!("ERROR: {rslt:?}");
  quit(reason);
}

async fn manage_after_handshake<const IS_CLIENT: bool, TCX>(
  options: &Options,
  mut _sent_message: bool,
  tls_stream: &mut TlsStream<TcpStream, TCX, IS_CLIENT>,
) -> wtx::Result<()>
where
  TCX: TlsCtx,
{
  let mut quench_writes = false;
  let mut _sent_key_update = false;
  let mut sent_shutdown = false;

  if options.export_keying_material > 0 {
    let mut export_buf = vec![0u8; options.export_keying_material];
    let context = if options.export_keying_material_context_used {
      Some(options.export_keying_material_context.as_bytes())
    } else {
      None
    };
    tls_stream.export_keying_material(
      context,
      options.export_keying_material_label.as_bytes(),
      &mut export_buf,
    )?;
    tls_stream.write_all(&export_buf).await?;
  }

  if options.export_traffic_secrets {
    let (read_secret, write_secret) = tls_stream.export_traffic_secrets();
    let (read_secret_vec, write_secret_vec) = (read_secret.to_vec(), write_secret.to_vec());
    let read_len = u16::try_from(read_secret_vec.len()).unwrap();
    tls_stream.write_all(&read_len.to_le_bytes()).await?;
    tls_stream.write_all(&read_secret_vec).await?;
    tls_stream.write_all(&write_secret_vec).await?;
  }

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

  let read_size = options.read_size.min(2048);
  let mut buffer = Vector::from_iterator((0..read_size).map(|_| 0))?;

  loop {
    let len = match tls_stream.read(buffer.get_mut(..read_size).unwrap().into()).await {
      Ok(None) => return Ok(()),
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

fn make_client_cfg<TCX>(ctx: TCX, options: &Options) -> TlsConfig<TCX>
where
  TCX: TlsCtx,
{
  let mut cfg = TlsConfig::new(ctx).unwrap();
  if !options.trusted_cert_file.is_empty() {
    let pem = cert_pem_from_pem_file(&options.trusted_cert_file);
    cfg.set_trust_anchors_pem([pem.as_bytes()]).unwrap();
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
  if options.use_sni {
    cfg
      .server_name_mut()
      .get_or_insert_default()
      .server_name_list
      .push(ServerName::from_name(options.host_name.as_str().try_into().unwrap()))
      .unwrap();
  }
  if let Some(named_groups) = &options.groups {
    cfg.supported_groups_mut().named_group_list.clear();
    for named_group in named_groups {
      cfg.supported_groups_mut().named_group_list.push(*named_group).unwrap();
    }
  }
  if let Some(el) = options.verify_prefs {
    cfg.signature_algorithms_mut().signature_schemes.clear();
    cfg.signature_algorithms_mut().signature_schemes.push(el).unwrap();
  }
  cfg
}

fn make_server_cfg(options: &Options) -> TlsConfig<SkCtx> {
  let mut rng = ChaCha20::from_std_random().unwrap();
  let mut cfg = TlsConfig::new(
    SkCtx::from_pems(
      options.keys_pem.iter().map(|elem| elem.as_bytes().try_into().unwrap()),
      &mut rng,
    )
    .unwrap(),
  )
  .unwrap();
  cfg.set_public_keys_pem(options.certs_pem.iter().map(|el| el.as_bytes())).unwrap();
  *cfg.max_fragment_length_send_mut() = options.max_fragment;
  if options.select_empty_alpn {
    *cfg.alpn_mut() = Some(Alpn::default());
  }
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
  if !options.signing_prefs.is_empty() {
    cfg.signature_algorithms_mut().signature_schemes.clear();
    for el in &options.signing_prefs {
      cfg.signature_algorithms_mut().signature_schemes.push(*el).unwrap();
    }
  }
  if options.expect_selected_credential.is_some() {
    *cfg.unique_signature_algorithms_mut() = true;
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
