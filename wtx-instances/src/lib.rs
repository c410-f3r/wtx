//! WTX instances

#![allow(
  clippy::allow_attributes_without_reason,
  clippy::arithmetic_side_effects,
  clippy::cast_lossless,
  clippy::missing_inline_in_public_items,
  clippy::mod_module_files,
  clippy::shadow_unrelated,
  clippy::single_char_lifetime_names,
  clippy::std_instead_of_alloc,
  clippy::use_debug,
  clippy::wildcard_imports,
  missing_docs
)]

#[cfg(feature = "grpc")]
pub mod grpc_bindings;

/// Certificate
pub static CERT: &[u8] = include_bytes!("../../.certs/cert.pem");
/// Private key
pub static KEY: &[u8] = include_bytes!("../../.certs/key.pem");
/// Root CA
pub static ROOT_CA: &[u8] = include_bytes!("../../.certs/root-ca.crt");

#[cfg(any(feature = "autobahn-client", feature = "autobahn-client-concurrent"))]
pub async fn autobahn_case_conn(
  case: u32,
  host: &str,
) -> wtx::Result<
  wtx::web_socket::WebSocket<
    Option<wtx::web_socket::compression::NegotiatedFlate2>,
    wtx::rng::Xorshift64,
    tokio::net::TcpStream,
    wtx::web_socket::WebSocketBuffer,
    true,
  >,
> {
  wtx::web_socket::WebSocketConnector::default()
    .compression(wtx::web_socket::compression::Flate2::default())
    .no_masking(false)
    .connect(
      tokio::net::TcpStream::connect(host).await?,
      &wtx::collection::ArrayStringU8::<128>::try_from(format_args!(
        "http://{host}/runCase?case={case}&agent=wtx"
      ))?
      .as_str()
      .into(),
    )
    .await
}

#[cfg(any(feature = "autobahn-client", feature = "autobahn-client-concurrent"))]
pub async fn autobahn_close(host: &str) -> wtx::Result<()> {
  wtx::web_socket::WebSocketConnector::default()
    .connect(
      tokio::net::TcpStream::connect(host).await?,
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
pub async fn autobahn_get_case_count(
  buffer: &mut wtx::collection::Vector<u8>,
  host: &str,
) -> wtx::Result<u32> {
  let mut ws = wtx::web_socket::WebSocketConnector::default()
    .connect(
      tokio::net::TcpStream::connect(host).await?,
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

#[cfg(feature = "mysql")]
pub async fn executor_mysql(
  uri_str: &str,
) -> wtx::Result<
  wtx::database::client::mysql::MysqlExecutor<
    wtx::Error,
    wtx::database::client::mysql::ExecutorBuffer,
    tokio::net::TcpStream,
  >,
> {
  use wtx::rng::SeedableRng;
  let uri = wtx::misc::Uri::new(uri_str);
  let mut rng = wtx::rng::ChaCha20::from_getrandom()?;
  wtx::database::client::mysql::MysqlExecutor::connect(
    &wtx::database::client::mysql::Config::from_uri(&uri)?,
    wtx::database::client::mysql::ExecutorBuffer::new(usize::MAX, &mut rng),
    &mut rng,
    tokio::net::TcpStream::connect(uri.hostname_with_implied_port()).await?,
  )
  .await
}

#[cfg(feature = "postgres")]
pub async fn executor_postgres(
  uri_str: &str,
) -> wtx::Result<
  wtx::database::client::postgres::PostgresExecutor<
    wtx::Error,
    wtx::database::client::postgres::ExecutorBuffer,
    tokio::net::TcpStream,
  >,
> {
  use wtx::rng::SeedableRng;
  let uri = wtx::misc::Uri::new(uri_str);
  let mut rng = wtx::rng::ChaCha20::from_getrandom()?;
  wtx::database::client::postgres::PostgresExecutor::connect(
    &wtx::database::client::postgres::Config::from_uri(&uri)?,
    wtx::database::client::postgres::ExecutorBuffer::new(usize::MAX, &mut rng),
    &mut rng,
    tokio::net::TcpStream::connect(uri.hostname_with_implied_port()).await?,
  )
  .await
}

/// Host from arguments
pub fn host_from_args() -> String {
  std::env::args().nth(1).unwrap_or_else(|| "127.0.0.1:9000".to_owned())
}

/// Uri from arguments
pub fn uri_from_args() -> String {
  std::env::args().nth(1).unwrap_or_else(|| "http://127.0.0.1:9000".to_owned())
}
