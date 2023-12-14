#[cfg(feature = "_tokio-rustls-client")]
#[path = "../../../../examples/tls_stream/mod.rs"]
mod tls_stream;

use crate::{
  database::{
    client::postgres::{Config, Executor, ExecutorBuffer},
    Executor as _, Record, Records as _,
  },
  misc::UriPartsRef,
  rng::StaticRng,
};
use tokio::net::TcpStream;

type Err = crate::Error;

#[tokio::test]
async fn conn_md5() {
  let mut _executor = executor().await;
}

#[cfg(feature = "_tokio-rustls-client")]
#[tokio::test]
async fn conn_scram() {
  let uri = "postgres://wtx_scram:wtx@localhost:5433/wtx";
  let up = UriPartsRef::new(&uri);
  let mut rng = StaticRng::default();
  let _executor = Executor::connect_encrypted(
    &Config::from_uri_parts(&up).unwrap(),
    ExecutorBuffer::with_default_params(&mut rng),
    TcpStream::connect(up.host()).await.unwrap(),
    &mut rng,
    |stream| async { Ok(tls_stream::_tls_stream_stream(up.hostname(), stream).await) },
  )
  .await
  .unwrap();
}

#[tokio::test]
async fn execute() {
  let mut exec = executor().await;

  assert_eq!(exec.execute::<Err, _>("", ()).await.unwrap(), 0);
  assert_eq!(
    exec.execute::<Err, _>("CREATE TABLE IF NOT EXISTS foo(id INT)", ()).await.unwrap(),
    0
  );
  assert_eq!(exec.execute::<Err, _>("INSERT INTO foo VALUES (1)", ()).await.unwrap(), 1);
  assert_eq!(exec.execute::<Err, _>("INSERT INTO foo VALUES (1), (1)", ()).await.unwrap(), 2);
  assert_eq!(exec.execute::<Err, _>("DROP TABLE foo", ()).await.unwrap(), 0);
}

#[tokio::test]
async fn record() {
  let mut exec = executor().await;

  let _0c_0p = exec.record::<Err, _>("", ()).await;
  assert!(matches!(_0c_0p.unwrap_err(), Err::NoRecord));
  let _0c_1p = exec.record::<Err, _>("SELECT 1 WHERE 0=$1", (1,)).await;
  assert!(matches!(_0c_1p.unwrap_err(), Err::NoRecord));
  let _0c_2p = exec.record::<Err, _>("SELECT 1 WHERE 0=$1 AND 1=$2", (1, 2)).await;
  assert!(matches!(_0c_2p.unwrap_err(), Err::NoRecord));

  let _1c_0p = exec.record::<Err, _>("SELECT 1", ()).await.unwrap();
  assert_eq!(_1c_0p.len(), 1);
  assert_eq!(_1c_0p.decode::<_, u32>(0).unwrap(), 1);
  let _1c_1p = exec.record::<Err, _>("SELECT 1 WHERE 0=$1", (0,)).await.unwrap();
  assert_eq!(_1c_1p.len(), 1);
  assert_eq!(_1c_1p.decode::<_, u32>(0).unwrap(), 1);
  let _1c_2p = exec.record::<Err, _>("SELECT 1 WHERE 0=$1 AND 1=$2", (0, 1)).await.unwrap();
  assert_eq!(_1c_2p.len(), 1);
  assert_eq!(_1c_2p.decode::<_, u32>(0).unwrap(), 1);

  let _2c_0p = exec.record::<Err, _>("SELECT 1,2", ()).await.unwrap();
  assert_eq!(_2c_0p.len(), 2);
  assert_eq!(_2c_0p.decode::<_, u32>(0).unwrap(), 1);
  assert_eq!(_2c_0p.decode::<_, u32>(1).unwrap(), 2);
  let _2c_1p = exec.record::<Err, _>("SELECT 1,2 WHERE 0=$1", (0,)).await.unwrap();
  assert_eq!(_2c_1p.len(), 2);
  assert_eq!(_2c_1p.decode::<_, u32>(0).unwrap(), 1);
  assert_eq!(_2c_1p.decode::<_, u32>(1).unwrap(), 2);
  let _2c_2p = exec.record::<Err, _>("SELECT 1,2 WHERE 0=$1 AND 1=$2", (0, 1)).await.unwrap();
  assert_eq!(_2c_2p.len(), 2);
  assert_eq!(_2c_2p.decode::<_, u32>(0).unwrap(), 1);
  assert_eq!(_2c_2p.decode::<_, u32>(1).unwrap(), 2);
}

#[tokio::test]
async fn records() {
  let mut exec = executor().await;

  // 0 rows, 0 columns

  let _0r_0c_0p = exec.records::<Err, _>("", (), |_| Ok(())).await.unwrap();
  assert_eq!(_0r_0c_0p.len(), 0);
  let _0r_0c_1p = exec.records::<Err, _>("SELECT 1 WHERE 0=$1", (1,), |_| Ok(())).await.unwrap();
  assert_eq!(_0r_0c_1p.len(), 0);
  let _0r_0c_2p =
    exec.records::<Err, _>("SELECT 1 WHERE 0=$1 AND 1=$2", (1, 2), |_| Ok(())).await.unwrap();
  assert_eq!(_0r_0c_2p.len(), 0);

  // 1 row,  1 column

  let _1r_1c_0p = exec.records::<Err, _>("SELECT 1", (), |_| Ok(())).await.unwrap();
  assert_eq!(_1r_1c_0p.len(), 1);
  assert_eq!(_1r_1c_0p.record(0).unwrap().decode::<_, u32>(0).unwrap(), 1);
  assert_eq!(_1r_1c_0p.record(0).unwrap().len(), 1);
  let _1r_1c_1p = exec.records::<Err, _>("SELECT 1 WHERE 0=$1", (0,), |_| Ok(())).await.unwrap();
  assert_eq!(_1r_1c_1p.len(), 1);
  assert_eq!(_1r_1c_1p.record(0).unwrap().decode::<_, u32>(0).unwrap(), 1);
  assert_eq!(_1r_1c_1p.record(0).unwrap().len(), 1);
  let _1r_1c_2p =
    exec.records::<Err, _>("SELECT 1 WHERE 0=$1 AND 1=$2", (0, 1), |_| Ok(())).await.unwrap();
  assert_eq!(_1r_1c_2p.len(), 1);
  assert_eq!(_1r_1c_2p.record(0).unwrap().decode::<_, u32>(0).unwrap(), 1);
  assert_eq!(_1r_1c_2p.record(0).unwrap().len(), 1);

  // 1 row, 2 columns

  let _1r_2c_0p = exec.records::<Err, _>("SELECT 1,2", (), |_| Ok(())).await.unwrap();
  assert_eq!(_1r_2c_0p.len(), 1);
  assert_eq!(_1r_2c_0p.record(0).unwrap().decode::<_, u32>(0).unwrap(), 1);
  assert_eq!(_1r_2c_0p.record(0).unwrap().decode::<_, u32>(1).unwrap(), 2);
  let _1r_2c_1p = exec.records::<Err, _>("SELECT 1,2 WHERE 0=$1", (0,), |_| Ok(())).await.unwrap();
  assert_eq!(_1r_2c_1p.len(), 1);
  assert_eq!(_1r_2c_1p.record(0).unwrap().decode::<_, u32>(0).unwrap(), 1);
  assert_eq!(_1r_2c_1p.record(0).unwrap().decode::<_, u32>(1).unwrap(), 2);
  let _1r_2c_2p =
    exec.records::<Err, _>("SELECT 1,2 WHERE 0=$1 AND 1=$2", (0, 1), |_| Ok(())).await.unwrap();
  assert_eq!(_1r_2c_2p.len(), 1);
  assert_eq!(_1r_2c_2p.record(0).unwrap().decode::<_, u32>(0).unwrap(), 1);
  assert_eq!(_1r_2c_2p.record(0).unwrap().decode::<_, u32>(1).unwrap(), 2);

  // 2 rows, 1 column

  let _2r_1c_0p = exec
    .records::<Err, _>("SELECT * FROM (VALUES (1), (2)) AS t (foo)", (), |_| Ok(()))
    .await
    .unwrap();
  assert_eq!(_2r_1c_0p.len(), 2);
  assert_eq!(_2r_1c_0p.record(0).unwrap().len(), 1);
  assert_eq!(_2r_1c_0p.record(0).unwrap().decode::<_, u32>(0).unwrap(), 1);
  assert_eq!(_2r_1c_0p.record(1).unwrap().len(), 1);
  assert_eq!(_2r_1c_0p.record(1).unwrap().decode::<_, u32>(0).unwrap(), 2);
  let _2r_1c_1p = exec
    .records::<Err, _>("SELECT * FROM (VALUES (1), (2)) AS t (foo) WHERE 0=$1", (0,), |_| Ok(()))
    .await
    .unwrap();
  assert_eq!(_2r_1c_1p.len(), 2);
  assert_eq!(_2r_1c_1p.record(0).unwrap().len(), 1);
  assert_eq!(_2r_1c_1p.record(0).unwrap().decode::<_, u32>(0).unwrap(), 1);
  assert_eq!(_2r_1c_1p.record(1).unwrap().len(), 1);
  assert_eq!(_2r_1c_1p.record(1).unwrap().decode::<_, u32>(0).unwrap(), 2);
  let _2r_1c_2p = exec
    .records::<Err, _>(
      "SELECT * FROM (VALUES (1), (2)) AS t (foo) WHERE 0=$1 AND 1=$2",
      (0, 1),
      |_| Ok(()),
    )
    .await
    .unwrap();
  assert_eq!(_2r_1c_2p.len(), 2);
  assert_eq!(_2r_1c_2p.record(0).unwrap().len(), 1);
  assert_eq!(_2r_1c_2p.record(0).unwrap().decode::<_, u32>(0).unwrap(), 1);
  assert_eq!(_2r_1c_2p.record(1).unwrap().len(), 1);
  assert_eq!(_2r_1c_2p.record(1).unwrap().decode::<_, u32>(0).unwrap(), 2);

  // 2 rows, 2 columns

  let _2r_2c_0p = exec
    .records::<Err, _>("SELECT * FROM (VALUES (1,2), (3,4)) AS t (foo,bar)", (), |_| Ok(()))
    .await
    .unwrap();
  assert_eq!(_2r_2c_0p.len(), 2);
  assert_eq!(_2r_2c_0p.record(0).unwrap().len(), 2);
  assert_eq!(_2r_2c_0p.record(0).unwrap().decode::<_, u32>(0).unwrap(), 1);
  assert_eq!(_2r_2c_0p.record(0).unwrap().decode::<_, u32>(1).unwrap(), 2);
  assert_eq!(_2r_2c_0p.record(1).unwrap().len(), 2);
  assert_eq!(_2r_2c_0p.record(1).unwrap().decode::<_, u32>(0).unwrap(), 3);
  assert_eq!(_2r_2c_0p.record(1).unwrap().decode::<_, u32>(1).unwrap(), 4);
  let _2r_2c_1p = exec
    .records::<Err, _>(
      "SELECT * FROM (VALUES (1,2), (3,4)) AS t (foo,bar) WHERE 0=$1",
      (0,),
      |_| Ok(()),
    )
    .await
    .unwrap();
  assert_eq!(_2r_2c_1p.len(), 2);
  assert_eq!(_2r_2c_1p.record(0).unwrap().len(), 2);
  assert_eq!(_2r_2c_1p.record(0).unwrap().decode::<_, u32>(0).unwrap(), 1);
  assert_eq!(_2r_2c_1p.record(0).unwrap().decode::<_, u32>(1).unwrap(), 2);
  assert_eq!(_2r_2c_1p.record(1).unwrap().len(), 2);
  assert_eq!(_2r_2c_1p.record(1).unwrap().decode::<_, u32>(0).unwrap(), 3);
  assert_eq!(_2r_2c_1p.record(1).unwrap().decode::<_, u32>(1).unwrap(), 4);
  let _2r_2c_2p = exec
    .records::<Err, _>(
      "SELECT * FROM (VALUES (1,2), (3,4)) AS t (foo,bar) WHERE 0=$1 AND 1=$2",
      (0, 1),
      |_| Ok(()),
    )
    .await
    .unwrap();
  assert_eq!(_2r_2c_2p.len(), 2);
  assert_eq!(_2r_2c_2p.record(0).unwrap().len(), 2);
  assert_eq!(_2r_2c_2p.record(0).unwrap().decode::<_, u32>(0).unwrap(), 1);
  assert_eq!(_2r_2c_2p.record(0).unwrap().decode::<_, u32>(1).unwrap(), 2);
  assert_eq!(_2r_2c_2p.record(1).unwrap().len(), 2);
  assert_eq!(_2r_2c_2p.record(1).unwrap().decode::<_, u32>(0).unwrap(), 3);
  assert_eq!(_2r_2c_2p.record(1).unwrap().decode::<_, u32>(1).unwrap(), 4);
}

#[tokio::test]
async fn reuses_cached_statement() {
  let mut exec = executor().await;
  let _record = exec.record::<Err, _>("SELECT 1 WHERE 0=$1", (0,)).await.unwrap();
  let _record = exec.record::<Err, _>("SELECT 1 WHERE 0=$1", (0,)).await.unwrap();
}

async fn executor() -> Executor<ExecutorBuffer, TcpStream> {
  let uri = "postgres://wtx_md5:wtx@localhost:5432/wtx";
  let up = UriPartsRef::new(&uri);
  let mut rng = StaticRng::default();
  Executor::connect(
    &Config::from_uri_parts(&up).unwrap(),
    ExecutorBuffer::with_default_params(&mut rng),
    &mut rng,
    TcpStream::connect(up.host()).await.unwrap(),
  )
  .await
  .unwrap()
}
