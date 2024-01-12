use crate::misc::Agent;
use diesel::prelude::table;
use futures::stream::StreamExt;
use sqlx::{Connection, Either, Executor as _, Row, Statement};
use std::time::Instant;
use tokio::{net::TcpStream, task::JoinSet};
use tokio_postgres::NoTls;
use wtx::{
  database::{
    client::postgres::{Executor, ExecutorBuffer},
    Executor as _, Record as _, Records,
  },
  misc::UriRef,
  rng::{Rng, StdRng},
};

// Verifies the handling of concurrent calls.
const CONNECTIONS: usize = 64;
// Bytes to create and receive.
const DATA_LEN: usize = 32 * 1024;
// Number of sequential `SELECT` statements.
const QUERIES: usize = 8 * 1024;

const SELECT_QUERY: &str = "SELECT * FROM benchmark";

pub(crate) async fn bench(
  uri: &UriRef<'_>,
  [diesel_async, sqlx_postgres, tokio_postgres, wtx]: [&mut Agent; 4],
) {
  populate_db(&mut StdRng::default(), uri).await;
  bench_diesel_async(diesel_async, uri).await;
  bench_sqlx_postgres(sqlx_postgres, uri).await;
  bench_tokio_postgres(tokio_postgres, uri).await;
  bench_wtx(wtx, uri).await;
}

pub(crate) fn caption() -> String {
  format!(
    "{CONNECTIONS} connection(s) retrieving {QUERIES} sequential queries of {DATA_LEN} byte(s)"
  )
}

#[allow(clippy::single_char_lifetime_names, unused_qualifications, clippy::shadow_unrelated)]
async fn bench_diesel_async(agent: &mut Agent, uri: &UriRef<'_>) {
  use diesel_async::{AsyncPgConnection, RunQueryDsl};

  table! {
    benchmark(bar, baz) {
      bar -> Text,
      baz -> Text,
    }
  }

  let mut set = JoinSet::new();
  for _ in 0..CONNECTIONS {
    let _handle = set.spawn({
      let local_uri = uri.to_string();
      async move {
        let (client, conn) = tokio_postgres::Config::new()
          .dbname(local_uri.path().get(1..).unwrap())
          .host(local_uri.hostname())
          .password(local_uri.password())
          .port(local_uri.port().parse().unwrap())
          .user(local_uri.user())
          .connect(NoTls)
          .await
          .unwrap();
        let _handle = tokio::spawn(async move {
          if let Err(err) = conn.await {
            println!("Error: {err}");
          }
        });
        let mut pg_conn = AsyncPgConnection::try_from(client).await.unwrap();
        let instant = Instant::now();
        for _ in 0..QUERIES {
          let records = benchmark::table.load::<(String, String)>(&mut pg_conn).await.unwrap();
          assert!(!records[0].0.is_empty());
          assert!(!records[0].1.is_empty());
          assert!(!records[1].0.is_empty());
          assert!(!records[1].1.is_empty());
        }
        instant.elapsed().as_millis()
      }
    });
  }
  exec(agent, &mut set).await;
}

async fn bench_sqlx_postgres(agent: &mut Agent, uri: &UriRef<'_>) {
  let mut set = JoinSet::new();
  for _ in 0..CONNECTIONS {
    let _handle = set.spawn({
      let local_uri = uri.uri().to_owned();
      async move {
        let mut conn = sqlx::postgres::PgConnection::connect(&local_uri).await.unwrap();
        let stmt = conn.prepare(SELECT_QUERY).await.unwrap();
        let instant = Instant::now();
        for _ in 0..QUERIES {
          let mut rows = Vec::new();
          let mut stream = stmt.query().fetch_many(&mut conn);
          while let Some(result) = stream.next().await {
            match result.unwrap() {
              Either::Left(_) => {}
              Either::Right(row) => rows.push(row),
            }
          }
          assert!(!rows[0].get::<&str, _>(0).is_empty());
          assert!(!rows[0].get::<&str, _>(1).is_empty());
          assert!(!rows[1].get::<&str, _>(0).is_empty());
          assert!(!rows[1].get::<&str, _>(1).is_empty());
        }
        instant.elapsed().as_millis()
      }
    });
  }
  exec(agent, &mut set).await;
}

async fn bench_tokio_postgres(agent: &mut Agent, uri: &UriRef<'_>) {
  let mut set = JoinSet::new();
  for _ in 0..CONNECTIONS {
    let _handle = set.spawn({
      let local_uri = uri.to_string();
      async move {
        let (client, conn) = tokio_postgres::Config::new()
          .dbname(local_uri.path().get(1..).unwrap())
          .host(local_uri.hostname())
          .password(local_uri.password())
          .port(local_uri.port().parse().unwrap())
          .user(local_uri.user())
          .connect(NoTls)
          .await
          .unwrap();
        let _handle = tokio::spawn(async move {
          if let Err(err) = conn.await {
            println!("Error: {err}");
          }
        });
        let stmt = client.prepare(SELECT_QUERY).await.unwrap();
        let instant = Instant::now();
        for _ in 0..QUERIES {
          let rows = client.query(&stmt, &[]).await.unwrap();
          assert!(!rows[0].get::<_, &str>(0).is_empty());
          assert!(!rows[0].get::<_, &str>(1).is_empty());
          assert!(!rows[1].get::<_, &str>(0).is_empty());
          assert!(!rows[1].get::<_, &str>(1).is_empty());
        }
        instant.elapsed().as_millis()
      }
    });
  }
  exec(agent, &mut set).await;
}

async fn bench_wtx(agent: &mut Agent, uri: &UriRef<'_>) {
  let mut set = JoinSet::new();
  for _ in 0..CONNECTIONS {
    let _handle = set.spawn({
      let local_uri = uri.to_string();
      async move {
        let mut executor = wtx_executor(&mut StdRng::default(), &local_uri.to_ref()).await;
        let stmt = executor.prepare(SELECT_QUERY).await.unwrap();
        let instant = Instant::now();
        for _ in 0..QUERIES {
          let records = executor.fetch_many_with_stmt(stmt, (), |_| Ok(())).await.unwrap();
          assert!(!records.get(0).unwrap().decode::<_, &str>(0).unwrap().is_empty());
          assert!(!records.get(0).unwrap().decode::<_, &str>(1).unwrap().is_empty());
          assert!(!records.get(1).unwrap().decode::<_, &str>(0).unwrap().is_empty());
          assert!(!records.get(1).unwrap().decode::<_, &str>(1).unwrap().is_empty());
        }
        instant.elapsed().as_millis()
      }
    });
  }
  exec(agent, &mut set).await;
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

async fn exec(agent: &mut Agent, set: &mut JoinSet<u128>) {
  let mut sum = 0;
  while let Some(rslt) = set.join_next().await {
    sum += rslt.unwrap();
  }
  agent.result = sum / u128::try_from(CONNECTIONS).unwrap();
}

async fn populate_db(rng: &mut StdRng, uri: &UriRef<'_>) {
  let mut executor = wtx_executor(rng, uri).await;
  let mut data = String::new();
  let _ = executor.execute_with_stmt("DROP TABLE IF EXISTS benchmark", ()).await.unwrap();
  let _ = executor
    .execute_with_stmt("CREATE TABLE benchmark(bar TEXT NOT NULL, baz TEXT NOT NULL)", ())
    .await
    .unwrap();
  let (bar0, baz0) = fill_and_split_data(&mut data, rng);
  let _ = executor
    .execute_with_stmt(format!("INSERT INTO benchmark VALUES ('{bar0}', '{baz0}')").as_str(), ())
    .await
    .unwrap();
  data.clear();
  let (bar1, baz1) = fill_and_split_data(&mut data, rng);
  let _ = executor
    .execute_with_stmt(format!("INSERT INTO benchmark VALUES ('{bar1}', '{baz1}')").as_str(), ())
    .await
    .unwrap();
}

async fn wtx_executor(
  rng: &mut StdRng,
  uri: &UriRef<'_>,
) -> Executor<wtx::Error, ExecutorBuffer, TcpStream> {
  Executor::connect(
    &wtx::database::client::postgres::Config::from_uri(uri).unwrap(),
    ExecutorBuffer::with_default_params(rng),
    rng,
    TcpStream::connect(uri.host()).await.unwrap(),
  )
  .await
  .unwrap()
}
