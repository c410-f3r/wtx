//! Based on <https://github.com/rustls/rustls/tree/main/bogo>

#![expect(
  clippy::exit,
  clippy::print_stderr,
  clippy::print_stdout,
  clippy::struct_excessive_bools,
  clippy::too_many_lines,
  clippy::unwrap_used,
  clippy::use_debug,
  reason = "does not matter"
)]

macro_rules! manage_after_handshake {
  (
    $options:expr,
    $sent_message:expr,
    $stream:expr,
    $traffic_secrets:expr,
    |$stream_reader:ident| $reader_cb:expr,
    |$stream_writer:ident| $writer_cb:expr
  ) => {{
    let mut quench_writes = false;
    let mut _sent_key_update = false;
    let mut sent_shutdown = false;

    if $options.export_traffic_secrets {
      let $stream_writer = $stream;
      let stream_writer = $writer_cb;
      let (read_secret, write_secret) = $traffic_secrets;
      let read_len = u16::try_from(read_secret.len()).unwrap();
      stream_writer.write_all(&read_len.to_le_bytes()).await?;
      stream_writer.write_all(&read_secret).await?;
      stream_writer.write_all(&write_secret).await?;
    }

    if $options.export_keying_material > 0 {
      let mut export_buf = vec![0u8; $options.export_keying_material];
      let context = if $options.export_keying_material_context_used {
        Some($options.export_keying_material_context.as_bytes())
      } else {
        None
      };
      let $stream_writer = $stream;
      let stream_writer = $writer_cb;
      stream_writer.export_keying_material(
        context,
        $options.export_keying_material_label.as_bytes(),
        &mut export_buf,
      )?;
      stream_writer.write_all(&export_buf).await?;
    }

    if $options.send_key_update && !_sent_key_update {
      let $stream_writer = $stream;
      $writer_cb.refresh_traffic_keys().await?;
      _sent_key_update = true;
    }

    if ($options.queue_data || $options.only_write_one_byte_after_handshake) && !$sent_message {
      let $stream_writer = $stream;
      let stream_writer = $writer_cb;
      stream_writer.write_all(b"hello").await?;
      $sent_message = true;
      if $options.only_write_one_byte_after_handshake {
        stream_writer.stream_mut().write_all(&[0]).await?;
        quench_writes = true;
      }
    }

    let read_size = $options.read_size.min(2048);
    let mut buffer = Vector::from_iterator((0..read_size).map(|_| 0))?;

    loop {
      let read_rslt = {
        let $stream_reader = $stream;
        let reader = $reader_cb;
        reader.read(buffer.get_mut(..read_size).unwrap().into()).await
      };
      let len = match read_rslt {
        Ok(None) => break,
        Ok(Some(len)) => len.get(),
        Err(err) => return Err(err),
      };

      if $options.shut_down_after_handshake && !sent_shutdown {
        let $stream_writer = $stream;
        $writer_cb.send_close_notify().await?;
        sent_shutdown = true;
      }

      if quench_writes && len > 0 {
        quench_writes = false;
      }

      if len > 0 {
        for byte in buffer.get_mut(..len).unwrap() {
          *byte ^= 255;
        }
        let $stream_writer = $stream;
        $writer_cb.write_all(buffer.get(..len).unwrap()).await?;
      }
    }
  }};
}

#[path = "common/boringssl_handle_err.rs"]
mod boringssl_handle_err;
#[path = "common/boringssl_options.rs"]
mod boringssl_options;

use crate::boringssl_options::{OptionsIter, cert_pem_from_pem_file};
use boringssl_options::Options;
use std::{env, time::Duration};
use tokio::net::TcpStream;
use wtx::{
  collections::Vector,
  net::{Stream, StreamReader, StreamWriter as _, Uri},
  rng::{ChaCha20, CryptoSeedableRng as _},
  sync::{Arc, AsyncMutex},
  tls::{
    Alpn, HandshakePath, NamedGroup, ServerName, SkCtx, TlsAcceptor, TlsConfig,
    TlsConnectorBuilder, TlsCtx, TlsCtxSk, TlsCtxSkLoader, TlsStream, UnverifiedCtx,
  },
};

#[tokio::main]
async fn main() {
  let mut is_concurrent = false;
  for var in std::env::vars() {
    if var.0 == "WTX_BORINGSSL_IS_CONCURRENT" {
      is_concurrent = var.1.parse::<u8>().unwrap() == 1;
      break;
    }
  }
  wtx::misc::tracing_tree_init(None).unwrap();
  let mut options = Options::default();
  for _ in OptionsIter::new(env::args().skip(1), &mut options) {}
  if options.is_client {
    if boringssl_options::verify_cert(&options) {
      let tls_config = make_client_cfg(SkCtx::default(), &options);
      exec_tests::<_, true>(is_concurrent, options, tls_config).await;
    } else {
      let tls_config = make_client_cfg(UnverifiedCtx::default(), &options);
      exec_tests::<_, true>(is_concurrent, options, tls_config).await;
    }
  } else {
    let tls_config = make_server_cfg(&options);
    exec_tests::<_, false>(is_concurrent, options, tls_config).await;
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

async fn exec_tests<TCX, const IS_CLIENT: bool>(
  is_concurrent: bool,
  options: Options,
  tls_config: TlsConfig<TCX>,
) where
  TCX: TlsCtxSk + Send + 'static,
{
  for idx in 0..=options.resume_count {
    let uri = Uri::new(format!("localhost:{}", options.port));
    let rng = ChaCha20::from_std_random().unwrap();
    let rslt = if IS_CLIENT {
      let fut = async {
        let mut connector = TlsConnectorBuilder::tokio(uri).build(&tls_config, rng).await?;
        connector.stream_mut().write_all(&options.shim_id.to_le_bytes()).await?;
        let rslt = connector.connect().await?;
        check_handshake_params(rslt.handshake_path, idx, rslt.named_group, &options);
        manage_after_handshake(is_concurrent, &options, false, rslt.tls_stream).await
      };
      fut.await
    } else {
      let fut = async {
        let mut stream = TcpStream::connect(uri.hostname_with_implied_port()).await?;
        stream.write_all(&options.shim_id.to_le_bytes()).await?;
        let rslt = TlsAcceptor::new(&tls_config, rng, stream).accept().await?;
        check_handshake_params(rslt.handshake_path, idx, rslt.named_group, &options);
        manage_after_handshake(is_concurrent, &options, false, rslt.tls_stream).await
      };
      fut.await
    };
    wtx::futures::Sleep::new(Duration::from_millis(50)).unwrap().await.unwrap();
    boringssl_handle_err::handle_err(&options, rslt);
  }
}

async fn manage_after_handshake<const IS_CLIENT: bool, TCX>(
  is_concurrent: bool,
  options: &Options,
  mut _sent_message: bool,
  mut tls_stream: TlsStream<TcpStream, TCX, IS_CLIENT>,
) -> wtx::Result<()>
where
  TCX: TlsCtx + Send + 'static,
{
  if is_concurrent {
    let (bridge, mut reader, writer) = tls_stream.into_split()?;
    let shared_writer = Arc::new(AsyncMutex::new(writer));
    let bridge_writer = shared_writer.clone();
    let _jh = tokio::task::spawn(async move {
      loop {
        let data = bridge.listen().await;
        if bridge_writer.lock().await.manage_bridge_data(data).await? {
          break;
        }
      }
      wtx::Result::Ok(())
    });
    let traffic_secrets_vec = (
      reader.export_traffic_secret().to_vec(),
      shared_writer.lock().await.export_traffic_secret().to_vec(),
    );
    manage_after_handshake!(
      options,
      _sent_message,
      (&mut reader, &shared_writer),
      traffic_secrets_vec,
      |local_stream| local_stream.0,
      |local_stream| &mut local_stream.1.lock().await
    );
  } else {
    let traffic_secrets_vec = {
      let traffic_secrets = tls_stream.export_traffic_secrets();
      (traffic_secrets.0.to_vec(), traffic_secrets.1.to_vec())
    };
    manage_after_handshake!(
      options,
      _sent_message,
      &mut tls_stream,
      traffic_secrets_vec,
      |local_stream| local_stream,
      |local_stream| local_stream
    );
  }
  Ok(())
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
    SkCtx::from_pems(options.keys_pem.iter().map(|elem| elem.as_bytes()), &mut rng).unwrap(),
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
