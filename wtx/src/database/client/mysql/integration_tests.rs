use crate::{
  database::{
    DatabaseError, Executor as _, Record, Records as _,
    client::mysql::{Config, ExecutorBuffer, MysqlExecutor},
  },
  misc::{UriRef, Xorshift64, simple_seed},
};
use alloc::string::String;
use core::fmt::Debug;
use std::{env, sync::LazyLock};
use tokio::net::TcpStream;

static URI: LazyLock<String> = LazyLock::new(|| env::var("DATABASE_URI_MYSQL").unwrap());

#[tokio::test]
async fn execute() {
  let mut exec = executor::<crate::Error>().await;

  assert_eq!(
    exec.execute_with_stmt("CREATE TABLE IF NOT EXISTS execute_test(id INT)", ()).await.unwrap(),
    0
  );
  assert_eq!(exec.execute_with_stmt("INSERT INTO execute_test VALUES (1)", ()).await.unwrap(), 1);
  assert_eq!(
    exec.execute_with_stmt("INSERT INTO execute_test VALUES (1), (1)", ()).await.unwrap(),
    2
  );
  assert_eq!(exec.execute_with_stmt("DROP TABLE execute_test", ()).await.unwrap(), 0);
}

#[tokio::test]
async fn record() {
  let mut exec = executor::<crate::Error>().await;

  let _0c_1p = exec.fetch_with_stmt("SELECT '1' WHERE 0=?", (1,)).await;
  assert!(matches!(_0c_1p.unwrap_err(), crate::Error::DatabaseError(DatabaseError::MissingRecord)));
  let _0c_2p = exec.fetch_with_stmt("SELECT '1' WHERE 0=? AND 1=?", (1, 2)).await;
  assert!(matches!(_0c_2p.unwrap_err(), crate::Error::DatabaseError(DatabaseError::MissingRecord)));

  let _1c_0p = exec.fetch_with_stmt("SELECT '1'", ()).await.unwrap();
  assert_eq!(_1c_0p.len(), 1);
  assert_eq!(_1c_0p.decode::<_, &str>(0).unwrap(), "1");
  let _1c_1p = exec.fetch_with_stmt("SELECT '1' WHERE 0=?", (0,)).await.unwrap();
  assert_eq!(_1c_1p.len(), 1);
  assert_eq!(_1c_1p.decode::<_, &str>(0).unwrap(), "1");
  let _1c_2p = exec.fetch_with_stmt("SELECT '1' WHERE 0=? AND 1=?", (0, 1)).await.unwrap();
  assert_eq!(_1c_2p.len(), 1);
  assert_eq!(_1c_2p.decode::<_, &str>(0).unwrap(), "1");

  let _2c_0p = exec.fetch_with_stmt("SELECT '1','2'", ()).await.unwrap();
  assert_eq!(_2c_0p.len(), 2);
  assert_eq!(_2c_0p.decode::<_, &str>(0).unwrap(), "1");
  assert_eq!(_2c_0p.decode::<_, &str>(1).unwrap(), "2");
  let _2c_1p = exec.fetch_with_stmt("SELECT '1','2' WHERE 0=?", (0,)).await.unwrap();
  assert_eq!(_2c_1p.len(), 2);
  assert_eq!(_2c_1p.decode::<_, &str>(0).unwrap(), "1");
  assert_eq!(_2c_1p.decode::<_, &str>(1).unwrap(), "2");
  let _2c_2p = exec.fetch_with_stmt("SELECT '1','2' WHERE 0=? AND 1=?", (0, 1)).await.unwrap();
  assert_eq!(_2c_2p.len(), 2);
  assert_eq!(_2c_2p.decode::<_, &str>(0).unwrap(), "1");
  assert_eq!(_2c_2p.decode::<_, &str>(1).unwrap(), "2");
}

#[tokio::test]
async fn records() {
  let mut exec = executor::<crate::Error>().await;

  // 0 rows, 0 columns

  let _0r_0c_1p =
    exec.fetch_many_with_stmt("SELECT '1' WHERE 0=?", (1,), |_| Ok(())).await.unwrap();
  assert_eq!(_0r_0c_1p.len(), 0);
  let _0r_0c_2p =
    exec.fetch_many_with_stmt("SELECT '1' WHERE 0=? AND 1=?", (1, 2), |_| Ok(())).await.unwrap();
  assert_eq!(_0r_0c_2p.len(), 0);

  // 1 row,  1 column

  let _1r_1c_0p = exec.fetch_many_with_stmt("SELECT '1'", (), |_| Ok(())).await.unwrap();
  assert_eq!(_1r_1c_0p.len(), 1);
  assert_eq!(_1r_1c_0p.get(0).unwrap().decode::<_, &str>(0).unwrap(), "1");
  assert_eq!(_1r_1c_0p.get(0).unwrap().len(), 1);
  let _1r_1c_1p =
    exec.fetch_many_with_stmt("SELECT '1' WHERE 0=?", (0,), |_| Ok(())).await.unwrap();
  assert_eq!(_1r_1c_1p.len(), 1);
  assert_eq!(_1r_1c_1p.get(0).unwrap().decode::<_, &str>(0).unwrap(), "1");
  assert_eq!(_1r_1c_1p.get(0).unwrap().len(), 1);
  let _1r_1c_2p =
    exec.fetch_many_with_stmt("SELECT '1' WHERE 0=? AND 1=?", (0, 1), |_| Ok(())).await.unwrap();
  assert_eq!(_1r_1c_2p.len(), 1);
  assert_eq!(_1r_1c_2p.get(0).unwrap().decode::<_, &str>(0).unwrap(), "1");
  assert_eq!(_1r_1c_2p.get(0).unwrap().len(), 1);

  // 1 row, 2 columns

  let _1r_2c_0p = exec.fetch_many_with_stmt("SELECT '1','2'", (), |_| Ok(())).await.unwrap();
  assert_eq!(_1r_2c_0p.len(), 1);
  assert_eq!(_1r_2c_0p.get(0).unwrap().decode::<_, &str>(0).unwrap(), "1");
  assert_eq!(_1r_2c_0p.get(0).unwrap().decode::<_, &str>(1).unwrap(), "2");
  let _1r_2c_1p =
    exec.fetch_many_with_stmt("SELECT '1','2' WHERE 0=?", (0,), |_| Ok(())).await.unwrap();
  assert_eq!(_1r_2c_1p.len(), 1);
  assert_eq!(_1r_2c_1p.get(0).unwrap().decode::<_, &str>(0).unwrap(), "1");
  assert_eq!(_1r_2c_1p.get(0).unwrap().decode::<_, &str>(1).unwrap(), "2");
  let _1r_2c_2p = exec
    .fetch_many_with_stmt("SELECT '1','2' WHERE 0=? AND 1=?", (0, 1), |_| Ok(()))
    .await
    .unwrap();
  assert_eq!(_1r_2c_2p.len(), 1);
  assert_eq!(_1r_2c_2p.get(0).unwrap().decode::<_, &str>(0).unwrap(), "1");
  assert_eq!(_1r_2c_2p.get(0).unwrap().decode::<_, &str>(1).unwrap(), "2");

  // 2 rows, 1 column

  let _2r_1c_0p = exec
    .fetch_many_with_stmt("SELECT * FROM (SELECT '1' UNION ALL SELECT 2) AS foo", (), |_| Ok(()))
    .await
    .unwrap();
  assert_eq!(_2r_1c_0p.len(), 2);
  assert_eq!(_2r_1c_0p.get(0).unwrap().len(), 1);
  assert_eq!(_2r_1c_0p.get(0).unwrap().decode::<_, &str>(0).unwrap(), "1");
  assert_eq!(_2r_1c_0p.get(1).unwrap().len(), 1);
  assert_eq!(_2r_1c_0p.get(1).unwrap().decode::<_, &str>(0).unwrap(), "2");
  let _2r_1c_1p = exec
    .fetch_many_with_stmt(
      "SELECT * FROM (SELECT '1' UNION ALL SELECT 2) AS foo  WHERE 0=?",
      (0,),
      |_| Ok(()),
    )
    .await
    .unwrap();
  assert_eq!(_2r_1c_1p.len(), 2);
  assert_eq!(_2r_1c_1p.get(0).unwrap().len(), 1);
  assert_eq!(_2r_1c_1p.get(0).unwrap().decode::<_, &str>(0).unwrap(), "1");
  assert_eq!(_2r_1c_1p.get(1).unwrap().len(), 1);
  assert_eq!(_2r_1c_1p.get(1).unwrap().decode::<_, &str>(0).unwrap(), "2");
  let _2r_1c_2p = exec
    .fetch_many_with_stmt(
      "SELECT * FROM (SELECT '1' AS foo UNION ALL SELECT 2) AS t (foo) WHERE 0=? AND 1=?",
      (0, 1),
      |_| Ok(()),
    )
    .await
    .unwrap();
  assert_eq!(_2r_1c_2p.len(), 2);
  assert_eq!(_2r_1c_2p.get(0).unwrap().len(), 1);
  assert_eq!(_2r_1c_2p.get(0).unwrap().decode::<_, &str>(0).unwrap(), "1");
  assert_eq!(_2r_1c_2p.get(1).unwrap().len(), 1);
  assert_eq!(_2r_1c_2p.get(1).unwrap().decode::<_, &str>(0).unwrap(), "2");

  // 2 rows, 2 columns

  let _2r_2c_0p = exec
    .fetch_many_with_stmt(
      "SELECT * FROM (SELECT '1','2' UNION ALL SELECT 3,4) AS t (foo,bar)",
      (),
      |_| Ok(()),
    )
    .await
    .unwrap();
  assert_eq!(_2r_2c_0p.len(), 2);
  assert_eq!(_2r_2c_0p.get(0).unwrap().len(), 2);
  assert_eq!(_2r_2c_0p.get(0).unwrap().decode::<_, &str>(0).unwrap(), "1");
  assert_eq!(_2r_2c_0p.get(0).unwrap().decode::<_, &str>(1).unwrap(), "2");
  assert_eq!(_2r_2c_0p.get(1).unwrap().len(), 2);
  assert_eq!(_2r_2c_0p.get(1).unwrap().decode::<_, &str>(0).unwrap(), "3");
  assert_eq!(_2r_2c_0p.get(1).unwrap().decode::<_, &str>(1).unwrap(), "4");
  let _2r_2c_1p = exec
    .fetch_many_with_stmt(
      "SELECT * FROM (SELECT '1','2' UNION ALL SELECT 3,4) AS t (foo,bar) WHERE 0=?",
      (0,),
      |_| Ok(()),
    )
    .await
    .unwrap();
  assert_eq!(_2r_2c_1p.len(), 2);
  assert_eq!(_2r_2c_1p.get(0).unwrap().len(), 2);
  assert_eq!(_2r_2c_1p.get(0).unwrap().decode::<_, &str>(0).unwrap(), "1");
  assert_eq!(_2r_2c_1p.get(0).unwrap().decode::<_, &str>(1).unwrap(), "2");
  assert_eq!(_2r_2c_1p.get(1).unwrap().len(), 2);
  assert_eq!(_2r_2c_1p.get(1).unwrap().decode::<_, &str>(0).unwrap(), "3");
  assert_eq!(_2r_2c_1p.get(1).unwrap().decode::<_, &str>(1).unwrap(), "4");
  let _2r_2c_2p = exec
    .fetch_many_with_stmt(
      "SELECT * FROM (SELECT '1','2' UNION ALL SELECT 3,4) AS t (foo,bar) WHERE 0=? AND 1=?",
      (0, 1),
      |_| Ok(()),
    )
    .await
    .unwrap();
  assert_eq!(_2r_2c_2p.len(), 2);
  assert_eq!(_2r_2c_2p.get(0).unwrap().len(), 2);
  assert_eq!(_2r_2c_2p.get(0).unwrap().decode::<_, &str>(0).unwrap(), "1");
  assert_eq!(_2r_2c_2p.get(0).unwrap().decode::<_, &str>(1).unwrap(), "2");
  assert_eq!(_2r_2c_2p.get(1).unwrap().len(), 2);
  assert_eq!(_2r_2c_2p.get(1).unwrap().decode::<_, &str>(0).unwrap(), "3");
  assert_eq!(_2r_2c_2p.get(1).unwrap().decode::<_, &str>(1).unwrap(), "4");
}

#[cfg(feature = "tokio-rustls")]
#[tokio::test]
async fn tls() {
  use crate::tests::_32_bytes_seed;
  use rand_chacha::{ChaCha20Rng, rand_core::SeedableRng};
  let uri_string = &*URI;
  let uri = UriRef::new(uri_string.as_str());
  let mut rng = ChaCha20Rng::from_seed(_32_bytes_seed());
  let _executor = MysqlExecutor::<crate::Error, _, _>::connect_encrypted(
    &Config::from_uri(&uri).unwrap(),
    ExecutorBuffer::new(usize::MAX, &mut rng),
    &mut rng,
    TcpStream::connect(uri.hostname_with_implied_port()).await.unwrap(),
    |stream| async {
      Ok(
        crate::misc::TokioRustlsConnector::default()
          .push_certs(include_bytes!("../../../../../.certs/root-ca.crt"))
          .unwrap()
          .connect_without_client_auth(uri.hostname(), stream)
          .await
          .unwrap(),
      )
    },
  )
  .await
  .unwrap();
}

async fn executor<E>() -> MysqlExecutor<E, ExecutorBuffer, TcpStream>
where
  E: Debug + From<crate::Error>,
{
  let uri_string = &*URI;
  let uri = UriRef::new(uri_string.as_str());
  let mut rng = Xorshift64::from(simple_seed());
  MysqlExecutor::connect(
    &Config::from_uri(&uri).unwrap(),
    ExecutorBuffer::new(usize::MAX, &mut rng),
    TcpStream::connect(uri.hostname_with_implied_port()).await.unwrap(),
  )
  .await
  .unwrap()
}
