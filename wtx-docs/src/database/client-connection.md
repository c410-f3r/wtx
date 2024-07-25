
# Client Connection

PostgreSQL is currently the only supported database. Implements <https://www.postgresql.org/docs/16/protocol.html>.

Activation feature is called `postgres`.

![PostgreSQL Benchmark](https://i.imgur.com/vf2tYxY.jpg)

```rust,edition2021,no_run
extern crate tokio;
extern crate wtx;

use tokio::net::TcpStream;
use wtx::{
  database::{
    client::postgres::{Config, Executor, ExecutorBuffer},
    Executor as _, Record, Records,
  },
  misc::UriRef,
  rng::StdRng,
};

#[tokio::main]
async fn main() -> wtx::Result<()> {
  let uri_ref = UriRef::new("postgres://USER:PASSWORD@localhost:5432/DB_NAME");
  let mut rng = StdRng::default();
  let mut exec = Executor::connect(
    &Config::from_uri(&uri_ref)?,
    ExecutorBuffer::with_default_params(&mut rng)?,
    &mut rng,
    TcpStream::connect(uri_ref.host()).await?,
  )
  .await?;
  let records = exec
    .fetch_many_with_stmt("SELECT id, name FROM users", (), |_| Ok::<_, wtx::Error>(()))
    .await?;
  for record in records.iter() {
    println!("{}", record.decode::<_, u32>("id")?);
    println!("{}", record.decode::<_, &str>("name")?);
  }
  Ok(())
}
```