//! The DB isolation provided by `#[wtx::db]` allows the concurrent execution of identical queries
//! without causing conflicts.
//!
//! The `dir` parameter is optional.

fn main() {}

#[cfg(test)]
mod tests {
  use std::net::TcpStream;
  use wtx::database::{
    Executor, Record,
    client::postgres::{ExecutorBuffer, PostgresExecutor},
  };

  #[wtx::db(dir("../.test-utils"))]
  async fn first_test(conn: PostgresExecutor<wtx::Error, ExecutorBuffer, TcpStream>) {
    common(conn).await;
  }

  #[wtx::db(dir("../.test-utils"))]
  async fn second_test(conn: PostgresExecutor<wtx::Error, ExecutorBuffer, TcpStream>) {
    common(conn).await;
  }

  #[wtx::db(dir("../.test-utils"))]
  async fn third_test(conn: PostgresExecutor<wtx::Error, ExecutorBuffer, TcpStream>) {
    common(conn).await;
  }

  async fn common(mut conn: PostgresExecutor<wtx::Error, ExecutorBuffer, TcpStream>) {
    conn
      .execute_ignored(
        "
        CREATE TABLE foo(id INT PRIMARY KEY, description TEXT NOT NULL);
        INSERT INTO foo VALUES (1, 'BAR!');
    ",
      )
      .await
      .unwrap();
    let id: &str = conn.execute_single("SELECT * FROM foo").await.unwrap().decode(0).unwrap();
    assert_eq!(id, "1");
  }
}
