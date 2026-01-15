//! Low-level HTTP/2 server that uses optioned parameters.
//!
//! Automatic streams are handled by the system while manual stream are handled by the user. In
//! this particular example all manual streams are considered to be WebSocket connections over
//! HTTP/2.
//!
//! Please note that it is much easier to just use the HTTP server framework.

extern crate tokio;
extern crate wtx;
extern crate wtx_instances;

use tokio::net::tcp::OwnedWriteHalf;
use wtx::{
  collection::Vector,
  http::{
    AutoStream, ManualServerStream, OperationMode, OptionedServer, ReqResBuffer, Response,
    StatusCode, is_web_socket_handshake,
  },
  http2::{Http2Buffer, Http2Params, WebSocketOverStream},
  rng::{Xorshift64, simple_seed},
  tls::{TlsAcceptor, TlsBuffer, TlsConfig, TlsStreamWriter},
  web_socket::{Frame, OpCode},
};

#[tokio::main]
async fn main() -> wtx::Result<()> {
  OptionedServer::http2_tokio(
    (&wtx_instances::host_from_args(), (), ()),
    |_| Ok(()),
    |stream| async move {
      Ok(
        TlsAcceptor::default()
          .push_cert(wtx_instances::CERT)
          .push_priv_key(wtx_instances::KEY)
          .accept(stream, TlsBuffer::default(), &TlsConfig::default())
          .await?
          .into_split(|el| el.into_split()),
      )
    },
    |error| eprintln!("{error}"),
    |_| {
      Ok((
        (),
        Http2Buffer::new(&mut Xorshift64::from(simple_seed())),
        Http2Params::default()
          .set_enable_connect_protocol(true)
          .set_max_hpack_len((128 * 1024, 128 * 1024)),
      ))
    },
    |_| Ok(Vector::new()),
    |_, _, protocol, req, _| {
      Ok((
        (),
        if is_web_socket_handshake(&req.rrd.headers, req.method, protocol) {
          OperationMode::Manual
        } else {
          OperationMode::Auto
        },
      ))
    },
    |error| eprintln!("{error}"),
    auto,
    manual,
  )
  .await
}

async fn auto(
  _: (),
  mut ha: AutoStream<(), Vector<u8>>,
) -> Result<Response<ReqResBuffer>, wtx::Error> {
  ha.req.clear();
  Ok(ha.req.into_response(StatusCode::Ok))
}

async fn manual(
  _: (),
  mut hm: ManualServerStream<(), Http2Buffer, Vector<u8>, TlsStreamWriter<OwnedWriteHalf>>,
) -> Result<(), wtx::Error> {
  let rng = Xorshift64::from(simple_seed());
  hm.req.rrd.headers.clear();
  let mut wos = WebSocketOverStream::new(&hm.req.rrd.headers, false, rng, hm.stream).await?;
  loop {
    let mut frame = wos.read_frame(&mut hm.stream_aux).await?;
    match (frame.op_code(), frame.text_payload()) {
      (_, Some(elem)) => println!("{elem}"),
      (OpCode::Close, _) => break,
      _ => {}
    }
    wos.write_frame(&mut Frame::new_fin(OpCode::Text, frame.payload_mut())).await?;
  }
  wos.close().await?;
  Ok(())
}
