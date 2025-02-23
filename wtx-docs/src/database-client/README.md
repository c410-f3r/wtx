
# Client Connection

Provides a set of functions that establish connections, execute queries and manage data transactions with different databases.

Independent benchmarks are available at <https://github.com/diesel-rs/metrics>.

## PostgreSQL

Implements a subset of <https://www.postgresql.org/docs/16/protocol.html>. PostgreSQL is a robust, open-source relational database management system that, among other things, supports several data types and usually also excels in concurrent scenarios.

To use this functionality, it is necessary to activate the `postgres` feature.

![PostgreSQL Benchmark](https://i.imgur.com/vf2tYxY.jpeg)

### Example

```rust,edition2024,no_run
{{#rustdoc_include ../../../wtx-instances/database-examples/database-client-postgres.rs}}
```

## MySQL

Implements a subset of <https://dev.mysql.com/doc/dev/mysql-server/latest/>. MySQL is also a robust, open-source relational database management system generally used in web applications.

`WTX` includes CI coverage for MariaDB and Percona, as such, interactions with these MySQL-based databases shouldn't be a problem.

To use this functionality, it is necessary to activate the `mysql` feature.

### Example

```rust,edition2024,no_run
{{#rustdoc_include ../../../wtx-instances/database-examples/database-client-mysql.rs}}
```