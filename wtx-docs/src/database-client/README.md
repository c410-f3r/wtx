
# Client Connection

Provides a set of functions that establish connections, execute queries and manage data transactions with different databases.

At the current time PostgreSQL is the only supported database. Implements <https://www.postgresql.org/docs/16/protocol.html>.

More benchmarks are available at <https://github.com/diesel-rs/metrics>.

To use this functionality, it necessary to activate the `postgres` feature.

![PostgreSQL Benchmark](https://i.imgur.com/vf2tYxY.jpeg)

## Example

```rust,edition2021,no_run
{{#rustdoc_include ../../../wtx-instances/database-examples/database-client-postgres.rs}}
```
