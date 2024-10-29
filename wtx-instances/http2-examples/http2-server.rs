//! HTTP/2 server that uses optioned parameters.

extern crate tokio;
extern crate tokio_rustls;
extern crate wtx;
extern crate wtx_instances;

use tokio::{io::WriteHalf, net::TcpStream};
use tokio_rustls::server::TlsStream;
use wtx::{
  http::{
    AutoStream, ManualServerStreamTokio, OptionedServer, ReqResBuffer, Response, StatusCode,
    StreamMode,
  },
  http2::{is_web_socket_handshake, Http2Buffer, Http2Params, WebSocketOverStream},
  misc::{simple_seed, TokioRustlsAcceptor, Vector, Xorshift64},
  web_socket::{Frame, OpCode},
};

#[tokio::main]
async fn main() -> wtx::Result<()> {
  OptionedServer::tokio_high_http2(
    &wtx_instances::host_from_args(),
    auto,
    || {
      Ok((
        (),
        Http2Buffer::new(Xorshift64::from(simple_seed())),
        Http2Params::default()
          .set_enable_connect_protocol(true)
          .set_max_hpack_len((128 * 1024, 128 * 1024)),
      ))
    },
    |error| eprintln!("{error}"),
    manual,
    || Ok((Vector::new(), ReqResBuffer::empty())),
    |headers, method, protocol| {
      Ok(if is_web_socket_handshake(headers, method, protocol) {
        StreamMode::Manual
      } else {
        StreamMode::Auto
      })
    },
    (
      || {
        TokioRustlsAcceptor::without_client_auth()
          .http2()
          .build_with_cert_chain_and_priv_key(wtx_instances::CERT, wtx_instances::KEY)
      },
      |acceptor| acceptor.clone(),
      |acceptor, stream| async move { Ok(tokio::io::split(acceptor.accept(stream).await?)) },
    ),
  )
  .await
}

async fn auto(mut ha: AutoStream<(), Vector<u8>>) -> Result<Response<ReqResBuffer>, wtx::Error> {
  ha.req.rrd.clear();
  Ok(ha.req.into_response(StatusCode::Ok))
}

async fn manual(
  mut hm: ManualServerStreamTokio<(), Vector<u8>, Http2Buffer, WriteHalf<TlsStream<TcpStream>>>,
) -> Result<(), wtx::Error> {
  let rng = Xorshift64::from(simple_seed());
  hm.headers.clear();
  let mut wos = WebSocketOverStream::new(&hm.headers, false, rng, hm.stream).await?;
  loop {
    let mut frame = wos.read_frame(&mut hm.sa).await?;
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
