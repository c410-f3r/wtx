
# Client Connection

PostgreSQL is currently the only supported database and more SQL or NoSQL variants shouldn't be too difficult to implement architecture-wise.

Activation feature is called `postgres`.

![PostgreSQL Benchmark](https://i.imgur.com/vf2tYxY.jpg)

```ignore,rust,edition2021
use wtx::{
  database::{client::postgres::{Executor, ExecutorBuffer}, Executor as _, Record, Records},
  misc::{LeaseMut, Stream},
};

async fn query_foo(
  executor: &mut Executor<impl LeaseMut<ExecutorBuffer>, impl Stream>,
) -> wtx::Result<(u32, String)> {
  let record = executor.fetch_with_stmt::<wtx::Error, _, _>(
    "SELECT bar,baz FROM foo WHERE bar = $1 AND baz = $2",
    (1u32, "2")
  ).await?;
  Ok((record.decode("bar")?, record.decode("baz")?))
}
```