//! WTX - Internal

#[cfg(any(feature = "autobahn-client", feature = "autobahn-client-concurrent"))]
#[inline]
pub async fn autobahn_case_conn(
  case: u32,
  host: &str,
) -> wtx::Result<
  wtx::web_socket::WebSocket<
    Option<wtx::web_socket::compression::NegotiatedFlate2>,
    wtx::rng::Xorshift64,
    std::net::TcpStream,
    wtx::web_socket::WebSocketBuffer,
    true,
  >,
> {
  wtx::web_socket::WebSocketConnector::default()
    .compression(wtx::web_socket::compression::Flate2::default())
    .no_masking(false)
    .connect(
      std::net::TcpStream::connect(host)?,
      &wtx::collection::ArrayStringU8::<128>::try_from(format_args!(
        "http://{host}/runCase?case={case}&agent=wtx"
      ))?
      .as_str()
      .into(),
    )
    .await
}

#[cfg(any(feature = "autobahn-client", feature = "autobahn-client-concurrent"))]
#[inline]
pub async fn autobahn_close(host: &str) -> wtx::Result<()> {
  wtx::web_socket::WebSocketConnector::default()
    .connect(
      std::net::TcpStream::connect(host)?,
      &wtx::collection::ArrayStringU8::<128>::try_from(format_args!(
        "http://{host}/updateReports?agent=wtx"
      ))?
      .as_str()
      .into(),
    )
    .await?
    .write_frame(&mut wtx::web_socket::Frame::new_fin(wtx::web_socket::OpCode::Close, &mut []))
    .await
}

#[cfg(any(feature = "autobahn-client", feature = "autobahn-client-concurrent"))]
#[inline]
pub async fn autobahn_get_case_count(
  buffer: &mut wtx::collection::Vector<u8>,
  host: &str,
) -> wtx::Result<u32> {
  let mut ws = wtx::web_socket::WebSocketConnector::default()
    .connect(
      std::net::TcpStream::connect(host)?,
      &wtx::collection::ArrayStringU8::<128>::try_from(format_args!("http://{host}/getCaseCount"))?
        .as_str()
        .into(),
    )
    .await?;
  let rslt = ws
    .read_frame(buffer, wtx::web_socket::WebSocketPayloadOrigin::Adaptive)
    .await?
    .text_payload()
    .unwrap_or_default()
    .parse()?;
  ws.write_frame(&mut wtx::web_socket::Frame::new_fin(wtx::web_socket::OpCode::Close, &mut []))
    .await?;
  Ok(rslt)
}

/// Host from arguments
#[inline]
pub fn host_from_args() -> String {
  std::env::args().nth(1).unwrap_or_else(|| "127.0.0.1:9000".to_owned())
}

/// Uri from arguments
#[inline]
pub fn uri_from_args() -> String {
  std::env::args().nth(1).unwrap_or_else(|| "http://127.0.0.1:9000".to_owned())
}
