use crate::misc::Agent;
use futures::stream::StreamExt;
use sqlx::{Connection, Either, Executor as _, Row};
use std::time::Instant;
use tokio::{net::TcpStream, task::JoinSet};
use tokio_postgres::NoTls;
use wtx::{
  database::{
    client::postgres::{Executor, ExecutorBuffer},
    Executor as _, Record as _, Records,
  },
  misc::UriPartsRef,
  rng::{Rng, StdRng},
};

// Verifies the handling of concurrent calls.
const CONNECTIONS: usize = 64;
// Bytes to create and receive.
const DATA_LEN: usize = 1028;
// Number of sequential `SELECT` statements.
const QUERIES: usize = 1024;

pub(crate) async fn bench(
  up: &UriPartsRef<'_>,
  [sqlx_postgres, tokio_postgres, wtx]: [&mut Agent; 3],
) {
  populate_db(&mut StdRng::default(), up).await;
  bench_sqlx_postgres(sqlx_postgres, up).await;
  bench_tokio_postgres(tokio_postgres, up).await;
  bench_wtx(wtx, up).await;
}

async fn bench_sqlx_postgres(agent: &mut Agent, up: &UriPartsRef<'_>) {
  let instant = Instant::now();
  let mut set = JoinSet::new();
  for _ in 0..CONNECTIONS {
    let _handle = set.spawn({
      let local_uri = up.uri().to_owned();
      async move {
        let mut conn = sqlx::postgres::PgConnection::connect(&local_uri).await.unwrap();
        for _ in 0..QUERIES {
          let mut rows = Vec::new();
          let mut stream = conn.fetch_many("SELECT * FROM foo");
          while let Some(result) = stream.next().await {
            match result.unwrap() {
              Either::Left(_) => {}
              Either::Right(row) => rows.push(row),
            }
          }
          assert!(!rows[0].get::<&str, _>("bar").is_empty());
          assert!(!rows[0].get::<&str, _>("baz").is_empty());
          assert!(!rows[1].get::<&str, _>("bar").is_empty());
          assert!(!rows[1].get::<&str, _>("baz").is_empty());
        }
      }
    });
  }
  while let Some(rslt) = set.join_next().await {
    rslt.unwrap();
  }
  agent.result = instant.elapsed().as_millis();
}

async fn bench_tokio_postgres(agent: &mut Agent, up: &UriPartsRef<'_>) {
  let instant = Instant::now();
  let mut set = JoinSet::new();
  for _ in 0..CONNECTIONS {
    let _handle = set.spawn({
      let local_up = up.clone().into_string();
      async move {
        let (client, conn) = tokio_postgres::Config::new()
          .host(local_up.hostname())
          .user(local_up.user())
          .port(local_up.port().parse().unwrap())
          .password(local_up.password())
          .dbname(local_up.path().get(1..).unwrap())
          .connect(NoTls)
          .await
          .unwrap();
        let _handle = tokio::spawn(async move {
          if let Err(e) = conn.await {
            println!("Error: {e}");
          }
        });
        for _ in 0..QUERIES {
          let rows = client.query("SELECT * FROM foo", &[]).await.unwrap();
          assert!(!rows[0].get::<_, &str>("bar").is_empty());
          assert!(!rows[0].get::<_, &str>("baz").is_empty());
          assert!(!rows[1].get::<_, &str>("bar").is_empty());
          assert!(!rows[1].get::<_, &str>("baz").is_empty());
        }
      }
    });
  }
  while let Some(rslt) = set.join_next().await {
    rslt.unwrap();
  }
  agent.result = instant.elapsed().as_millis();
}

async fn bench_wtx(agent: &mut Agent, up: &UriPartsRef<'_>) {
  let instant = Instant::now();
  let mut set = JoinSet::new();
  for _ in 0..CONNECTIONS {
    let _handle = set.spawn({
      let local_up = up.clone().into_string();
      async move {
        let mut executor = wtx_executor(&mut StdRng::default(), &local_up.as_ref()).await;
        for _ in 0..QUERIES {
          let records = executor.records("SELECT * FROM foo", (), |_| Ok(())).await.unwrap();
          assert!(!records.record(0).unwrap().decode::<_, &str>("bar").unwrap().is_empty());
          assert!(!records.record(0).unwrap().decode::<_, &str>("baz").unwrap().is_empty());
          assert!(!records.record(1).unwrap().decode::<_, &str>("bar").unwrap().is_empty());
          assert!(!records.record(1).unwrap().decode::<_, &str>("baz").unwrap().is_empty());
        }
      }
    });
  }
  while let Some(rslt) = set.join_next().await {
    rslt.unwrap();
  }
  agent.result = instant.elapsed().as_millis();
}

pub(crate) fn caption() -> String {
  format!(
    "{CONNECTIONS} connection(s) retrieving {QUERIES} sequential queries of {DATA_LEN} byte(s)"
  )
}

fn fill_and_split_data<'data>(
  data: &'data mut String,
  rng: &mut StdRng,
) -> (&'data str, &'data str) {
  data.extend((0..DATA_LEN).map(|_| {
    let byte = rng.u8();
    if byte.is_ascii_alphanumeric() {
      char::from(byte)
    } else {
      'a'
    }
  }));
  data.split_at(data.len() / 2)
}

async fn populate_db(rng: &mut StdRng, up: &UriPartsRef<'_>) {
  let mut executor = wtx_executor(rng, up).await;
  let mut data = String::new();
  let _ = executor.execute("DROP TABLE IF EXISTS foo;", ()).await.unwrap();
  let _ =
    executor.execute("CREATE TABLE foo(bar TEXT NOT NULL, baz TEXT NOT NULL)", ()).await.unwrap();
  let (bar0, baz0) = fill_and_split_data(&mut data, rng);
  let _ =
    executor.execute(&format!("INSERT INTO foo VALUES ('{bar0}', '{baz0}')"), ()).await.unwrap();
  data.clear();
  let (bar1, baz1) = fill_and_split_data(&mut data, rng);
  let _ =
    executor.execute(&format!("INSERT INTO foo VALUES ('{bar1}', '{baz1}')"), ()).await.unwrap();
}

async fn wtx_executor(
  rng: &mut StdRng,
  up: &UriPartsRef<'_>,
) -> Executor<ExecutorBuffer, TcpStream> {
  Executor::connect(
    &wtx::database::client::postgres::Config::from_uri_parts(up).unwrap(),
    ExecutorBuffer::with_default_params(rng),
    rng,
    TcpStream::connect(up.host()).await.unwrap(),
  )
  .await
  .unwrap()
}
