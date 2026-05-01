
# Database Client

Provides a set of functions that establish connections, execute queries and manage data transactions.

![Benchmark](https://i.imgur.com/vf2tYxY.jpeg)

Independent benchmarks are available at <https://github.com/diesel-rs/metrics>.

## PostgreSQL

Implements a subset of <https://www.postgresql.org/docs/16/protocol.html>. PostgreSQL is a robust, open-source relational database management system that, among other things, supports several data types and usually also excels in concurrent scenarios.

To use this functionality, it is necessary to activate the `postgres` feature.

### Example

```rust,edition2024,no_run
{{#rustdoc_include ../../../wtx-examples/database/database-client-postgres.rs}}
```

## Batch

PostgreSQL supports the sending of multiple statements in a single round-trip.

```rust,edition2024,no_run
{{#rustdoc_include ../../../wtx-examples/database/database-client-postgres-batch.rs}}
```

## Tests

The `#[wtx::db]` macro automatically migrates and seeds individual tests in isolation to allow concurrent evaluations.

Its current state is limited to PostgreSQL tests that use the standard `std::net::TcpStream` alongside the built-in executor. Connected users must have the right to create new databases.

To use this functionality, it is necessary to activate the `database-tests` feature.

```rust,edition2024,no_run
{{#rustdoc_include ../../../wtx-examples/database/database-client-tests.rs}}
```
