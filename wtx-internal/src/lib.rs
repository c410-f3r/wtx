//! WTX - Internal

#[cfg(any(feature = "autobahn-client", feature = "autobahn-client-concurrent"))]
use {
  tokio::net::TcpStream,
  wtx::{
    collections::{ArrayStringU8, Vector},
    misc::Uri,
    rng::{ChaCha20, CryptoSeedableRng as _},
    tls::{TlsConfig, TlsConnectorBuilder, TlsModePlainText},
    web_socket::{
      Frame, OpCode, WebSocket, WebSocketConnector, WebSocketPayloadOrigin,
      web_socket_compression::{NegotiatedZlibRs, ZlibRs},
    },
  },
};

/// Used by autobahn
#[cfg(any(feature = "autobahn-client", feature = "autobahn-client-concurrent"))]
#[inline]
pub async fn autobahn_case_conn(
  case: u32,
  host: &str,
) -> wtx::Result<WebSocket<Option<NegotiatedZlibRs>, TcpStream, TlsModePlainText, true>> {
  let uri =
    ArrayStringU8::<128>::try_from(format_args!("http://{host}/runCase?case={case}&agent=wtx"))?;
  WebSocketConnector::default()
    .set_compression(ZlibRs::default())
    .set_no_masking(false)
    .connect(
      TlsConnectorBuilder::tokio(Uri::new(uri.as_str()))
        .build(TlsConfig::plaintext(), ChaCha20::from_std_random()?)
        .await?,
    )
    .await
}

/// Used by autobahn
#[cfg(any(feature = "autobahn-client", feature = "autobahn-client-concurrent"))]
#[inline]
pub async fn autobahn_close(host: &str) -> wtx::Result<()> {
  let uri = ArrayStringU8::<128>::try_from(format_args!("http://{host}/updateReports?agent=wtx"))?;
  WebSocketConnector::default()
    .connect(
      TlsConnectorBuilder::tokio(Uri::new(uri.as_str()))
        .build(TlsConfig::plaintext(), ChaCha20::from_std_random()?)
        .await?,
    )
    .await?
    .write_frame(&mut Frame::new_fin(OpCode::Close, &mut [])?)
    .await
}

/// Used by autobahn
#[cfg(any(feature = "autobahn-client", feature = "autobahn-client-concurrent"))]
#[inline]
pub async fn autobahn_get_case_count(buffer: &mut Vector<u8>, host: &str) -> wtx::Result<u32> {
  let uri = ArrayStringU8::<128>::try_from(format_args!("http://{host}/getCaseCount"))?;
  let mut ws = WebSocketConnector::default()
    .connect(
      TlsConnectorBuilder::tokio(Uri::new(uri.as_str()))
        .build(TlsConfig::plaintext(), ChaCha20::from_std_random()?)
        .await?,
    )
    .await?;
  let rslt = ws
    .read_frame(buffer, WebSocketPayloadOrigin::Adaptive)
    .await?
    .text_payload()
    .unwrap_or_default()
    .parse()?;
  ws.write_frame(&mut Frame::new_fin(OpCode::Close, &mut [])?).await?;
  Ok(rslt)
}
