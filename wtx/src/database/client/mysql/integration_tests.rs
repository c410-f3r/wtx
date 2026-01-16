use crate::{
  database::client::mysql::{Config, ExecutorBuffer, MysqlExecutor},
  misc::UriRef,
  rng::{ChaCha20, SeedableRng, simple_32_seed},
  tests::_vars,
};
use core::fmt::Debug;

#[test]
fn execute() {
  crate::database::client::integration_tests::execute(executor::<crate::Error>());
}

#[test]
fn execute_interleaved() {
  crate::database::client::integration_tests::execute_interleaved(executor::<crate::Error>());
}

#[test]
fn execute_stmt_inserts() {
  crate::database::client::integration_tests::execute_stmt_inserts(executor::<crate::Error>());
}

#[test]
fn execute_stmt_selects() {
  crate::database::client::integration_tests::execute_stmt_selects(
    executor::<crate::Error>(),
    "?",
    "?",
  );
}

#[test]
fn ping() {
  crate::database::client::integration_tests::ping(executor::<crate::Error>());
}

#[test]
fn records_after_prepare() {
  crate::database::client::integration_tests::records_after_prepare(executor::<crate::Error>());
}

#[test]
fn reuses_cached_statement() {
  crate::database::client::integration_tests::reuses_cached_statement(
    executor::<crate::Error>(),
    "?",
  );
}

#[cfg(feature = "rust-crypto")]
#[test]
fn tls() {
  use crate::executor::Runtime;

  Runtime::new()
    .block_on(async {
      //let uri = UriRef::new(URI.as_str());
      //let mut rng = ChaCha20::from_seed(_32_bytes_seed()).unwrap();
      //  let _executor = MysqlExecutor::<crate::Error, _, _>::connect_encrypted(
      //    &Config::from_uri(&uri).unwrap(),
      //    ExecutorBuffer::new(usize::MAX, &mut rng),
      //    &mut rng,
      //    tokio::net::TcpStream::connect(uri.hostname_with_implied_port()).await.unwrap(),
      //    |stream| async {
      //      Ok(
      //        crate::misc::TokioRustlsConnector::default()
      //          .push_certs(include_bytes!("../../../../../.certs/root-ca.crt"))
      //          .unwrap()
      //          .connect_without_client_auth(uri.hostname(), stream)
      //          .await
      //          .unwrap(),
      //      )
      //    },
      //  )
      //  .await
      //  .unwrap();
    });
}

async fn executor<E>() -> MysqlExecutor<E, ExecutorBuffer, std::net::TcpStream>
where
  E: Debug + From<crate::Error>,
{
  let uri = UriRef::new(&_vars().database_uri_mysql.as_str());
  let mut rng = ChaCha20::from_seed(simple_32_seed()).unwrap();
  MysqlExecutor::connect(
    &Config::from_uri(&uri).unwrap(),
    ExecutorBuffer::new(usize::MAX, &mut rng),
    &mut rng,
    std::net::TcpStream::connect(uri.hostname_with_implied_port()).unwrap(),
  )
  .await
  .unwrap()
}
