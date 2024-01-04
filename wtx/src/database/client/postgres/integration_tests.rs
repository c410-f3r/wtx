use crate::{
  database::{
    client::postgres::{Config, Executor, ExecutorBuffer, Postgres, Value},
    Decode, Encode, Executor as _, Record, Records as _,
  },
  misc::{FilledBufferWriter, UriRef},
  rng::StaticRng,
};
use tokio::net::TcpStream;

type Err = crate::Error;

#[tokio::test]
async fn conn_md5() {
  let mut _executor = executor::<crate::Error>().await;
}

#[cfg(feature = "_tokio-rustls-client")]
#[tokio::test]
async fn conn_scram() {
  let uri = "postgres://wtx_scram:wtx@localhost:5433/wtx";
  let uri = UriRef::new(&uri);
  let mut rng = StaticRng::default();
  let _executor = Executor::<crate::Error, _, _>::connect_encrypted(
    &Config::from_uri(&uri).unwrap(),
    ExecutorBuffer::with_default_params(&mut rng),
    TcpStream::connect(uri.host()).await.unwrap(),
    &mut rng,
    |stream| async {
      Ok(
        crate::misc::TokioRustlsConnector::from_webpki_roots()
          .push_certs(include_bytes!("../../../../../.certs/root-ca.crt"))
          .unwrap()
          .with_generic_stream(uri.hostname(), stream)
          .await
          .unwrap(),
      )
    },
  )
  .await
  .unwrap();
}

#[tokio::test]
async fn custom_domain() {
  struct CustomDomain {
    a: u8,
    b: f64,
    c: String,
  }

  impl Decode<'_, Postgres<crate::Error>> for CustomDomain {
    fn decode(input: &Value<'_>) -> Result<Self, crate::Error> {
      let a = <_ as Decode<Postgres<crate::Error>>>::decode(input)?;
      let b = <_ as Decode<Postgres<crate::Error>>>::decode(input)?;
      let c = <_ as Decode<Postgres<crate::Error>>>::decode(input)?;
      Ok(Self { a, b, c })
    }
  }

  impl Encode<Postgres<crate::Error>> for CustomDomain {
    fn encode(&self, buffer: &mut FilledBufferWriter<'_>) -> Result<(), crate::Error> {
      <_ as Encode<Postgres<crate::Error>>>::encode(&self.a, buffer)?;
      <_ as Encode<Postgres<crate::Error>>>::encode(&self.b, buffer)?;
      <_ as Encode<Postgres<crate::Error>>>::encode(&self.c, buffer)?;
      Ok(())
    }
  }

  let mut exec = executor::<crate::Error>().await;
  exec
    .execute(
      "DROP TYPE IF EXISTS custom_domain CASCADE; DROP TABLE IF EXISTS custom_domain_table",
      |_| {},
    )
    .await
    .unwrap();
  exec.execute("CREATE DOMAIN custom_domain AS VARCHAR(64)", |_| {}).await.unwrap();
  exec
    .execute("CREATE TABLE custom_domain_table (id INT, domain custom_domain)", |_| {})
    .await
    .unwrap();
  let _ = exec
    .execute_with_stmt(
      "INSERT INTO custom_domain_table VALUES ($1, $2)",
      (1, CustomDomain { a: 2, b: 3.0, c: String::from("456") }),
    )
    .await
    .unwrap();
}

#[tokio::test]
async fn custom_error() {
  #[derive(Debug)]
  enum CustomError {
    Wtx { _err: crate::Error },
  }

  impl From<crate::Error> for CustomError {
    fn from(from: crate::Error) -> Self {
      Self::Wtx { _err: from }
    }
  }

  let mut exec = executor::<CustomError>().await;
  let _ = exec.fetch_with_stmt("SELECT 1 WHERE 0=$1", (0,)).await.unwrap();
  let _ = exec.fetch_with_stmt("SELECT 1 WHERE 0=0", ()).await.unwrap();
}

#[tokio::test]
async fn execute_with_stmt() {
  let mut exec = executor::<crate::Error>().await;

  assert_eq!(exec.execute_with_stmt("", ()).await.unwrap(), 0);
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
async fn multiple_notifications() {
  let mut exec = executor::<crate::Error>().await;
  let _ = exec
    .execute_with_stmt(
      "CREATE TABLE IF NOT EXISTS multiple_notifications_test (id SERIAL PRIMARY KEY, body TEXT)",
      (),
    )
    .await
    .unwrap();
  let _ =
    exec.execute_with_stmt("TRUNCATE TABLE multiple_notifications_test CASCADE", ()).await.unwrap();
}

#[tokio::test]
async fn record() {
  let mut exec = executor::<crate::Error>().await;

  let _0c_0p = exec.fetch_with_stmt("", ()).await;
  assert!(matches!(_0c_0p.unwrap_err(), Err::NoRecord));
  let _0c_1p = exec.fetch_with_stmt("SELECT 1 WHERE 0=$1", (1,)).await;
  assert!(matches!(_0c_1p.unwrap_err(), Err::NoRecord));
  let _0c_2p = exec.fetch_with_stmt("SELECT 1 WHERE 0=$1 AND 1=$2", (1, 2)).await;
  assert!(matches!(_0c_2p.unwrap_err(), Err::NoRecord));

  let _1c_0p = exec.fetch_with_stmt("SELECT 1", ()).await.unwrap();
  assert_eq!(_1c_0p.len(), 1);
  assert_eq!(_1c_0p.decode::<_, u32>(0).unwrap(), 1);
  let _1c_1p = exec.fetch_with_stmt("SELECT 1 WHERE 0=$1", (0,)).await.unwrap();
  assert_eq!(_1c_1p.len(), 1);
  assert_eq!(_1c_1p.decode::<_, u32>(0).unwrap(), 1);
  let _1c_2p = exec.fetch_with_stmt("SELECT 1 WHERE 0=$1 AND 1=$2", (0, 1)).await.unwrap();
  assert_eq!(_1c_2p.len(), 1);
  assert_eq!(_1c_2p.decode::<_, u32>(0).unwrap(), 1);

  let _2c_0p = exec.fetch_with_stmt("SELECT 1,2", ()).await.unwrap();
  assert_eq!(_2c_0p.len(), 2);
  assert_eq!(_2c_0p.decode::<_, u32>(0).unwrap(), 1);
  assert_eq!(_2c_0p.decode::<_, u32>(1).unwrap(), 2);
  let _2c_1p = exec.fetch_with_stmt("SELECT 1,2 WHERE 0=$1", (0,)).await.unwrap();
  assert_eq!(_2c_1p.len(), 2);
  assert_eq!(_2c_1p.decode::<_, u32>(0).unwrap(), 1);
  assert_eq!(_2c_1p.decode::<_, u32>(1).unwrap(), 2);
  let _2c_2p = exec.fetch_with_stmt("SELECT 1,2 WHERE 0=$1 AND 1=$2", (0, 1)).await.unwrap();
  assert_eq!(_2c_2p.len(), 2);
  assert_eq!(_2c_2p.decode::<_, u32>(0).unwrap(), 1);
  assert_eq!(_2c_2p.decode::<_, u32>(1).unwrap(), 2);
}

#[tokio::test]
async fn records() {
  let mut exec = executor::<crate::Error>().await;

  // 0 rows, 0 columns

  let _0r_0c_0p = exec.fetch_many_with_stmt("", (), |_| Ok(())).await.unwrap();
  assert_eq!(_0r_0c_0p.len(), 0);
  let _0r_0c_1p = exec.fetch_many_with_stmt("SELECT 1 WHERE 0=$1", (1,), |_| Ok(())).await.unwrap();
  assert_eq!(_0r_0c_1p.len(), 0);
  let _0r_0c_2p =
    exec.fetch_many_with_stmt("SELECT 1 WHERE 0=$1 AND 1=$2", (1, 2), |_| Ok(())).await.unwrap();
  assert_eq!(_0r_0c_2p.len(), 0);

  // 1 row,  1 column

  let _1r_1c_0p = exec.fetch_many_with_stmt("SELECT 1", (), |_| Ok(())).await.unwrap();
  assert_eq!(_1r_1c_0p.len(), 1);
  assert_eq!(_1r_1c_0p.get(0).unwrap().decode::<_, u32>(0).unwrap(), 1);
  assert_eq!(_1r_1c_0p.get(0).unwrap().len(), 1);
  let _1r_1c_1p = exec.fetch_many_with_stmt("SELECT 1 WHERE 0=$1", (0,), |_| Ok(())).await.unwrap();
  assert_eq!(_1r_1c_1p.len(), 1);
  assert_eq!(_1r_1c_1p.get(0).unwrap().decode::<_, u32>(0).unwrap(), 1);
  assert_eq!(_1r_1c_1p.get(0).unwrap().len(), 1);
  let _1r_1c_2p =
    exec.fetch_many_with_stmt("SELECT 1 WHERE 0=$1 AND 1=$2", (0, 1), |_| Ok(())).await.unwrap();
  assert_eq!(_1r_1c_2p.len(), 1);
  assert_eq!(_1r_1c_2p.get(0).unwrap().decode::<_, u32>(0).unwrap(), 1);
  assert_eq!(_1r_1c_2p.get(0).unwrap().len(), 1);

  // 1 row, 2 columns

  let _1r_2c_0p = exec.fetch_many_with_stmt("SELECT 1,2", (), |_| Ok(())).await.unwrap();
  assert_eq!(_1r_2c_0p.len(), 1);
  assert_eq!(_1r_2c_0p.get(0).unwrap().decode::<_, u32>(0).unwrap(), 1);
  assert_eq!(_1r_2c_0p.get(0).unwrap().decode::<_, u32>(1).unwrap(), 2);
  let _1r_2c_1p =
    exec.fetch_many_with_stmt("SELECT 1,2 WHERE 0=$1", (0,), |_| Ok(())).await.unwrap();
  assert_eq!(_1r_2c_1p.len(), 1);
  assert_eq!(_1r_2c_1p.get(0).unwrap().decode::<_, u32>(0).unwrap(), 1);
  assert_eq!(_1r_2c_1p.get(0).unwrap().decode::<_, u32>(1).unwrap(), 2);
  let _1r_2c_2p =
    exec.fetch_many_with_stmt("SELECT 1,2 WHERE 0=$1 AND 1=$2", (0, 1), |_| Ok(())).await.unwrap();
  assert_eq!(_1r_2c_2p.len(), 1);
  assert_eq!(_1r_2c_2p.get(0).unwrap().decode::<_, u32>(0).unwrap(), 1);
  assert_eq!(_1r_2c_2p.get(0).unwrap().decode::<_, u32>(1).unwrap(), 2);

  // 2 rows, 1 column

  let _2r_1c_0p = exec
    .fetch_many_with_stmt("SELECT * FROM (VALUES (1), (2)) AS t (foo)", (), |_| Ok(()))
    .await
    .unwrap();
  assert_eq!(_2r_1c_0p.len(), 2);
  assert_eq!(_2r_1c_0p.get(0).unwrap().len(), 1);
  assert_eq!(_2r_1c_0p.get(0).unwrap().decode::<_, u32>(0).unwrap(), 1);
  assert_eq!(_2r_1c_0p.get(1).unwrap().len(), 1);
  assert_eq!(_2r_1c_0p.get(1).unwrap().decode::<_, u32>(0).unwrap(), 2);
  let _2r_1c_1p = exec
    .fetch_many_with_stmt("SELECT * FROM (VALUES (1), (2)) AS t (foo) WHERE 0=$1", (0,), |_| Ok(()))
    .await
    .unwrap();
  assert_eq!(_2r_1c_1p.len(), 2);
  assert_eq!(_2r_1c_1p.get(0).unwrap().len(), 1);
  assert_eq!(_2r_1c_1p.get(0).unwrap().decode::<_, u32>(0).unwrap(), 1);
  assert_eq!(_2r_1c_1p.get(1).unwrap().len(), 1);
  assert_eq!(_2r_1c_1p.get(1).unwrap().decode::<_, u32>(0).unwrap(), 2);
  let _2r_1c_2p = exec
    .fetch_many_with_stmt(
      "SELECT * FROM (VALUES (1), (2)) AS t (foo) WHERE 0=$1 AND 1=$2",
      (0, 1),
      |_| Ok(()),
    )
    .await
    .unwrap();
  assert_eq!(_2r_1c_2p.len(), 2);
  assert_eq!(_2r_1c_2p.get(0).unwrap().len(), 1);
  assert_eq!(_2r_1c_2p.get(0).unwrap().decode::<_, u32>(0).unwrap(), 1);
  assert_eq!(_2r_1c_2p.get(1).unwrap().len(), 1);
  assert_eq!(_2r_1c_2p.get(1).unwrap().decode::<_, u32>(0).unwrap(), 2);

  // 2 rows, 2 columns

  let _2r_2c_0p = exec
    .fetch_many_with_stmt("SELECT * FROM (VALUES (1,2), (3,4)) AS t (foo,bar)", (), |_| Ok(()))
    .await
    .unwrap();
  assert_eq!(_2r_2c_0p.len(), 2);
  assert_eq!(_2r_2c_0p.get(0).unwrap().len(), 2);
  assert_eq!(_2r_2c_0p.get(0).unwrap().decode::<_, u32>(0).unwrap(), 1);
  assert_eq!(_2r_2c_0p.get(0).unwrap().decode::<_, u32>(1).unwrap(), 2);
  assert_eq!(_2r_2c_0p.get(1).unwrap().len(), 2);
  assert_eq!(_2r_2c_0p.get(1).unwrap().decode::<_, u32>(0).unwrap(), 3);
  assert_eq!(_2r_2c_0p.get(1).unwrap().decode::<_, u32>(1).unwrap(), 4);
  let _2r_2c_1p = exec
    .fetch_many_with_stmt(
      "SELECT * FROM (VALUES (1,2), (3,4)) AS t (foo,bar) WHERE 0=$1",
      (0,),
      |_| Ok(()),
    )
    .await
    .unwrap();
  assert_eq!(_2r_2c_1p.len(), 2);
  assert_eq!(_2r_2c_1p.get(0).unwrap().len(), 2);
  assert_eq!(_2r_2c_1p.get(0).unwrap().decode::<_, u32>(0).unwrap(), 1);
  assert_eq!(_2r_2c_1p.get(0).unwrap().decode::<_, u32>(1).unwrap(), 2);
  assert_eq!(_2r_2c_1p.get(1).unwrap().len(), 2);
  assert_eq!(_2r_2c_1p.get(1).unwrap().decode::<_, u32>(0).unwrap(), 3);
  assert_eq!(_2r_2c_1p.get(1).unwrap().decode::<_, u32>(1).unwrap(), 4);
  let _2r_2c_2p = exec
    .fetch_many_with_stmt(
      "SELECT * FROM (VALUES (1,2), (3,4)) AS t (foo,bar) WHERE 0=$1 AND 1=$2",
      (0, 1),
      |_| Ok(()),
    )
    .await
    .unwrap();
  assert_eq!(_2r_2c_2p.len(), 2);
  assert_eq!(_2r_2c_2p.get(0).unwrap().len(), 2);
  assert_eq!(_2r_2c_2p.get(0).unwrap().decode::<_, u32>(0).unwrap(), 1);
  assert_eq!(_2r_2c_2p.get(0).unwrap().decode::<_, u32>(1).unwrap(), 2);
  assert_eq!(_2r_2c_2p.get(1).unwrap().len(), 2);
  assert_eq!(_2r_2c_2p.get(1).unwrap().decode::<_, u32>(0).unwrap(), 3);
  assert_eq!(_2r_2c_2p.get(1).unwrap().decode::<_, u32>(1).unwrap(), 4);
}

#[tokio::test]
async fn records_after_prepare() {
  let mut exec = executor::<crate::Error>().await;
  let _ = exec.prepare("SELECT 1").await.unwrap();
  let _ = exec.fetch_many_with_stmt("SELECT 1", (), |_| Ok(())).await.unwrap();
}

#[tokio::test]
async fn reuses_cached_statement() {
  let mut exec = executor::<crate::Error>().await;
  let _record = exec.fetch_with_stmt("SELECT 1 WHERE 0=$1", (0,)).await.unwrap();
  let _record = exec.fetch_with_stmt("SELECT 1 WHERE 0=$1", (0,)).await.unwrap();
}

async fn executor<E>() -> Executor<E, ExecutorBuffer, TcpStream> {
  let uri = UriRef::new("postgres://wtx_md5:wtx@localhost:5432/wtx");
  let mut rng = StaticRng::default();
  Executor::connect(
    &Config::from_uri(&uri).unwrap(),
    ExecutorBuffer::with_default_params(&mut rng),
    &mut rng,
    TcpStream::connect(uri.host()).await.unwrap(),
  )
  .await
  .unwrap()
}
