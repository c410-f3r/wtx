
# Client Connection

PostgreSQL is currently the only supported database. Implements <https://www.postgresql.org/docs/16/protocol.html>.

Activation feature is called `postgres`.

![PostgreSQL Benchmark](https://i.imgur.com/vf2tYxY.jpeg)

## Example

The bellow snippet requires ~40 dependencies and has an optimized binary size of ~600K.

```rust,edition2021,no_run
{{#rustdoc_include ../../../wtx-instances/examples/database-client-postgres-tokio.rs}}
```
