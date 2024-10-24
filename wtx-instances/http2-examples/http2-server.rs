//! Serves requests using low-level HTTP/2 resources along side self-made certificates.

extern crate tokio;
extern crate wtx;
extern crate wtx_instances;

use tokio::{io::WriteHalf, net::TcpStream};
use tokio_rustls::server::TlsStream;
use wtx::{
  http::{Headers, OptionedServer, ReqResBuffer, Request, Response, StatusCode},
  http2::{Http2Buffer, Http2Params, ServerStreamTokio, WebSocketOverStream},
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

async fn auto(
  _: (),
  _: Vector<u8>,
  mut req: Request<ReqResBuffer>,
) -> Result<Response<ReqResBuffer>, wtx::Error> {
  req.rrd.clear();
  Ok(req.into_response(StatusCode::Ok))
}

async fn manual(
  _: (),
  mut buffer: Vector<u8>,
  _: Headers,
  stream: ServerStreamTokio<Http2Buffer, WriteHalf<TlsStream<TcpStream>>, false>,
) -> Result<(), wtx::Error> {
  let rng = Xorshift64::from(simple_seed());
  let mut wos = WebSocketOverStream::new(&Headers::new(), rng, stream).await?;
  loop {
    let mut frame = wos.read_frame(&mut buffer).await?;
    match (frame.op_code(), frame.text_payload()) {
      (_, Some(elem)) => println!("{elem}"),
      (OpCode::Close, _) => break,
      _ => {}
    }
    wos.write_frame(&mut Frame::new_fin(OpCode::Text, frame.payload_mut())).await?;
  }
  Ok(())
}
