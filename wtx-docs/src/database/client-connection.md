
# Client Connection

PostgreSQL is currently the only supported database. Implements <https://www.postgresql.org/docs/16/protocol.html>.

Activation feature is called `postgres`.

![PostgreSQL Benchmark](https://i.imgur.com/vf2tYxY.jpg)

```rust,edition2021
extern crate wtx;

use wtx::{
  database::{client::postgres::{Executor, ExecutorBuffer}, Executor as _, Record, Records},
  misc::{LeaseMut, Stream},
};

async fn query_foo<S>(
  executor: &mut Executor<wtx::Error, ExecutorBuffer, S>,
) -> wtx::Result<(u32, String)>
where
  S: Stream
{
  let record = executor.fetch_with_stmt(
    "SELECT bar,baz FROM foo WHERE bar = $1 AND baz = $2",
    (1u32, "2")
  ).await?;
  Ok((record.decode("bar")?, record.decode("baz")?))
}
```