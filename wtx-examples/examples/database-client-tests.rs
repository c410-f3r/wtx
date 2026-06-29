//! The DB isolation provided by `#[wtx::db]` allows the concurrent execution of identical queries
//! without causing conflicts.
//!
//! The `dir` parameter is optional.

fn main() {}

#[cfg(test)]
mod tests {
  use tokio::net::TcpStream;
  use wtx::database::{
    DbClient, Record,
    client::postgres::{ClientBuffer, PostgresClient},
  };

  type LocalPostgresClient = PostgresClient<ClientBuffer, wtx::Error, TcpStream>;

  #[wtx::db(dir("../.test-utils"))]
  async fn first_test(client: LocalPostgresClient) {
    common(client).await;
  }

  #[wtx::db(dir("../.test-utils"))]
  async fn second_test(client: LocalPostgresClient) {
    common(client).await;
  }

  #[wtx::db(dir("../.test-utils"))]
  async fn third_test(client: LocalPostgresClient) {
    common(client).await;
  }

  async fn common(mut client: LocalPostgresClient) {
    client
      .execute_ignored(
        "
          CREATE TABLE foo(id INT PRIMARY KEY, description TEXT NOT NULL);
          INSERT INTO foo VALUES (1, 'BAR!');
        ",
      )
      .await
      .unwrap();
    let id: &str = client.execute_single("SELECT * FROM foo").await.unwrap().decode(0).unwrap();
    assert_eq!(id, "1");
  }
}
