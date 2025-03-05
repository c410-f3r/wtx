//! Low level HTTP/2 server that only accepts one WebSocket stream for demonstration purposes.
//!
//! It is worth noting that the optioned server in the `http2-server` example automatically
//! handles WebSocket connections.

extern crate tokio;
extern crate wtx;
extern crate wtx_instances;

use core::mem;
use tokio::net::TcpListener;
use wtx::{
  http::{Headers, ReqResBuffer, is_web_socket_handshake},
  http2::{Http2Buffer, Http2Params, Http2Tokio, WebSocketOverStream},
  misc::{Either, TokioRustlsAcceptor, Vector, Xorshift64, simple_seed},
  web_socket::{Frame, OpCode},
};

#[tokio::main]
async fn main() -> wtx::Result<()> {
  let listener = TcpListener::bind(&wtx_instances::host_from_args()).await?;
  let mut rng = Xorshift64::from(simple_seed());
  let (tcp_stream, _) = listener.accept().await?;
  let acceptor = TokioRustlsAcceptor::without_client_auth()
    .http2()
    .build_with_cert_chain_and_priv_key(wtx_instances::CERT, wtx_instances::KEY)?;
  let (frame_reader, mut http2) = Http2Tokio::accept(
    Http2Buffer::new(&mut rng),
    Http2Params::default()
      .set_enable_connect_protocol(true)
      .set_max_hpack_len((128 * 1024, 128 * 1024)),
    tokio::io::split(acceptor.accept(tcp_stream).await?),
  )
  .await?;
  tokio::spawn(frame_reader);
  let (mut stream, headers_opt) = match http2
    .stream(ReqResBuffer::empty(), |req, protocol| {
      let rslt = is_web_socket_handshake(&req.rrd.headers, req.method, protocol);
      rslt.then(|| mem::take(&mut req.rrd.headers))
    })
    .await?
  {
    Either::Left(_) => return Ok(()),
    Either::Right(elem) => elem,
  };
  let Some(_headers) = headers_opt else {
    return Ok(());
  };
  let mut buffer = Vector::new();
  let mut wos = WebSocketOverStream::new(&Headers::new(), false, rng, &mut stream).await?;
  loop {
    let mut frame = wos.read_frame(&mut buffer).await?;
    match (frame.op_code(), frame.text_payload()) {
      (_, Some(elem)) => println!("{elem}"),
      (OpCode::Close, _) => break,
      _ => {}
    }
    wos.write_frame(&mut Frame::new_fin(OpCode::Text, frame.payload_mut())).await?;
  }
  wos.close().await?;
  stream.common().clear(false).await?;
  Ok(())
}
