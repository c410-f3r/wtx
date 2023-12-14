# WTX 

[![CI](https://github.com/c410-f3r/wtx/workflows/CI/badge.svg)](https://github.com/c410-f3r/wtx/actions?query=workflow%3ACI)
[![crates.io](https://img.shields.io/crates/v/wtx.svg)](https://crates.io/crates/wtx)
[![Documentation](https://docs.rs/wtx/badge.svg)](https://docs.rs/wtx)
[![License](https://img.shields.io/badge/license-APACHE2-blue.svg)](./LICENSE)
[![Rustc](https://img.shields.io/badge/rustc-1.75-lightgray")](https://blog.rust-lang.org/2020/03/12/Rust-1.75.html)

A collection of different transport implementations and related tools focused primarily on web technologies.

Embedded devices that have a heap allocator can use this `no_std` crate.

1. [Benchmarks](#benchmarks)
2. [Client API Framework](#client-api-framework)
3. [Database](#database)
    - [Client Connection](#client-connection)
    - [Object–Relational Mapping](#object–relational-mapping)
    - [Schema Management](#schema-management)
4. [WebSocket](#websocket)

# Benchmarks

If you disagree with any of the mentioned charts, feel free to checkout `wtx-bench` to point any misunderstandings or misconfigurations.

There are mainly 2 things that impact performance, the chosen runtime and the number of pre-allocated bytes. Specially for servers that have to create a new instance for each handshake, pre-allocating a high number of bytes for short-lived or low-transfer connections can have a negative impact.

It is also possible to use libraries that manage pools of bytes to avoid having to heap-allocate all the time.

# Client API Framework

A flexible client API framework for writing asynchronous, fast, organizable, scalable and maintainable applications. Supports several data formats, transports and custom parameters.

Activation feature is called `client-api-framework`. Checkout the `wtx-apis` project to see a collection of APIs based on `wtx`.

## Objective

It is possible to directly decode responses using built-in methods provided by some transport implementations like `reqwest` or `surf` but as complexity grows, the cost of maintaining large sets of endpoints with ad-hoc solutions usually becomes unsustainable. Based on this scenario, `wtx` comes into play to organize and centralize data flow in a well-defined manner to increase productivity and maintainability.

For API consumers, the calling convention of `wtx` endpoints is based on fluent interfaces which makes the usage more pleasant and intuitive.

Moreover, the project may in the future create automatic bindings for other languages in order to avoid having duplicated API repositories.

# Database

## Client Connection

PostgreSQL is currently the only supported database and more SQL or NoSQL variants shouldn't be too difficult to implement architecture-wise.

Activation feature is called `postgres`.

![PostgreSQL Benchmark](https://i.imgur.com/0qhYBBs.jpg)

```rust
#[cfg(feature = "postgres")]
mod postgres {
  use core::borrow::BorrowMut;
  use wtx::{
    database::{client::postgres::{Executor, ExecutorBuffer}, Executor as _, Record, Records},
    misc::Stream,
  };

  async fn query_foo(
    executor: &mut Executor<impl BorrowMut<ExecutorBuffer>, impl Stream>,
  ) -> wtx::Result<(u32, String)> {
    let record = executor.record::<wtx::Error, _>("SELECT bar,baz FROM foo WHERE bar = $1 AND baz = $2;", (1u32, "2")).await?;
    Ok((record.decode("bar")?, record.decode("baz")?))
  }
}
```

## Object–Relational Mapping

A very rudimentary ORM that currently supports very few operations that are not well tested. You probably should look for other similar projects.

Activation feature is called `orm`.

```rust
#[cfg(feature = "orm")]
mod orm {
  use wtx::database::{
    orm::{Crud, FromSuffixRslt, NoTableAssociation, Table, TableField, TableParams},
    Database, FromRecords, Record, TableSuffix,
  };

  struct User<'conn> {
    id: u32,
    name: &'conn str,
    password: &'conn str,
  }

  impl<'conn> FromRecords for User<'conn> {
    type Database = ();
    type Error = wtx::Error;

    fn from_records(
      _: &mut String,
      curr_record: &<Self::Database as Database>::Record<'_>,
      _: &<Self::Database as Database>::Records<'_>,
      _: TableSuffix,
    ) -> Result<(usize, Self), Self::Error> {
      let id = curr_record.decode(0)?;
      let name = curr_record.decode(1)?;
      let password = curr_record.decode(2)?;
      Ok((1, Self { id, name, password }))
    }
  }

  impl<'conn, 'entity> Table<'entity> for User<'conn> {
    const PRIMARY_KEY_NAME: &'static str = "id";
    const TABLE_NAME: &'static str = "user";

    type Associations = NoTableAssociation<wtx::Error>;
    type Error = wtx::Error;
    type Fields = (TableField<&'conn str>, TableField<&'conn str>);
    type PrimaryKeyValue = &'entity u32;

    fn type_instances(_: TableSuffix) -> FromSuffixRslt<'entity, Self> {
      (NoTableAssociation::new(), (TableField::new("name"), TableField::new("password")))
    }

    fn update_all_table_fields(entity: &'entity Self, table: &mut TableParams<'entity, Self>) {
      *table.id_field_mut().value_mut() = Some((&entity.id).into());
      *table.fields_mut().0.value_mut() = Some((entity.name).into());
      *table.fields_mut().1.value_mut() = Some((entity.password).into());
    }
  }

  async fn all_users<'conn>(
    crud: &'conn mut impl Crud<Database = ()>,
  ) -> wtx::Result<Vec<User<'conn>>> {
    let mut buffer = String::new();
    let mut results = Vec::new();
    crud.read_all::<User<'conn>>(&mut buffer, &mut results, &TableParams::default()).await?;
    Ok(results)
  }
}
```

## Schema Management

Embedded and CLI workflows using raw SQL commands.

Activation feature is called `sm`.

### CLI

```bash
# Example

cargo install --git https://github.com/c410-f3r/wtx --features sm-dev wtx-ui
echo DATABASE_URI="postgres://USER:PW@localhost:5432/DB" > .env
RUST_LOG=debug wtx-cli migrate
```

The CLI application expects a configuration file that contains a set of paths where each path is a directory with multiple migrations.

```toml
# wtx.toml

migration_groups = [
  "migrations/1__initial",
  "migrations/2__fancy_stuff"
]
```

Each provided migration and group must contain an unique version and a name summarized by the following structure:

```txt
// Execution order of migrations is dictated by their numeric declaration order.

migrations
+-- 1__initial (Group)
    +-- 1__create_author.sql (Migration)
    +-- 2__create_post.sql (Migration)
+-- 2__fancy_stuff (Group)
    +-- 1__something_fancy.sql (Migration)
wtx.toml
```

The SQL file itself is composed by two parts, one for migrations (`-- wtx IN` section) and another for rollbacks (`-- wtx OUT` section).

```sql
-- wtx IN

CREATE TABLE author (
  id INT NOT NULL PRIMARY KEY,
  added TIMESTAMP NOT NULL,
  birthdate DATE NOT NULL,
  email VARCHAR(100) NOT NULL,
  first_name VARCHAR(50) NOT NULL,
  last_name VARCHAR(50) NOT NULL
);

-- wtx OUT

DROP TABLE author;
```

One cool thing about the expected file configuration is that it can also be divided into smaller pieces, for example, the above migration could be transformed into `1__author_up.sql` and `1__author_down.sql`.

```sql
-- 1__author_up.sql

CREATE TABLE author (
  id INT NOT NULL PRIMARY KEY,
  added TIMESTAMP NOT NULL,
  birthdate DATE NOT NULL,
  email VARCHAR(100) NOT NULL,
  first_name VARCHAR(50) NOT NULL,
  last_name VARCHAR(50) NOT NULL
);
```

```sql
-- 1__author_down.sql

DROP TABLE author;
```

```txt
migrations
+-- 1__some_group (Group)
    +-- 1__author (Migration directory)
        +-- 1__author_down.sql (Down migration)
        +-- 1__author_up.sql (Up migration)
        +-- 1__author.toml (Optional configuration)
wtx.toml
```

### Library

The library gives freedom to arrange groups and uses some external crates, bringing ~10 additional dependencies into your application. If this overhead is not acceptable, then you probably should discard the library and use the CLI binary instead as part of a custom deployment strategy.

```rust
#[cfg(all(feature = "sm", feature = "std"))]
mod sm {
  use wtx::{
    database::{sm::Commands, DEFAULT_URI_VAR},
    misc::UriParts,
    rng::StaticRng,
  };
  use std::path::Path;
  
  #[tokio::main]
  async fn main() {
    let mut commands = Commands::with_executor(());
    commands
      .migrate_from_dir(
        (&mut <_>::default(), &mut <_>::default()),
        Path::new("my_custom_migration_group_path"),
      )
      .await
      .unwrap();
  }
}
```

### Embedded migrations

To make deployment easier, the final binary of your application can embed all necessary migrations through the binary that is available in the `wtx-ui` crate.

```rust
#[cfg(feature = "sm")]
mod sm {
  // This is an example! The actual contents are filled by the `wtx-ui embed-migrations` binary call.
  mod embedded_migrations {
    pub(crate) const GROUPS: wtx::database::sm::EmbeddedMigrationsTy = &[];
  }
  
  use wtx::database::sm::Commands;
  
  async fn migrate() -> wtx::Result<()> {
    Commands::with_executor(())
      .migrate_from_groups((&mut String::new(), &mut Vec::new()), embedded_migrations::GROUPS)
      .await
  }
}
```

### Conditional migrations

If one particular migration needs to be executed in a specific set of databases, then it is possible to use the `-- wtx dbs` parameter in a file.

```sql
-- wtx dbs mssql,postgres

-- wtx IN

CREATE SCHEMA foo;

-- wtx OUT

DROP SCHEMA foo;
```

### Repeatable migrations

Repeatability can be specified with `-- wtx repeatability SOME_VALUE` where `SOME_VALUE` can be either `always` (regardless of the checksum) or `on-checksum-change` (runs only when the checksums changes).

```sql
-- wtx dbs postgres
-- wtx repeatability always

-- wtx IN

CREATE OR REPLACE PROCEDURE something() LANGUAGE SQL AS $$ $$

-- wtx OUT

DROP PROCEDURE something();
```

Keep in mind that repeatable migrations might break subsequent operations, therefore, you must known what you are doing. If desirable, they can be separated into dedicated groups.

```ini
migrations/1__initial_repeatable_migrations
migrations/2__normal_migrations
migrations/3__final_repeatable_migrations
```

### Namespaces/Schemas

For supported databases, there is no direct user parameter that inserts migrations inside a single database schema but it is possible to specify the schema inside the SQL file and arrange the migration groups structure in a way that most suits you.

```sql
-- wtx IN

CREATE TABLE cool_department_schema.author (
  id INT NOT NULL PRIMARY KEY,
  full_name VARCHAR(50) NOT NULL
);

-- wtx OUT

DROP TABLE cool_department_schema.author;
```

# WebSocket

Provides low and high level abstractions to dispatch frames, as such, it is up to you to implement [Stream](https://docs.rs/wtx/latest/wtx/trait.Stream.html) with any desired logic or use any of the built-in strategies through the selection of features.

Activation feature is called `web-socket`.

![WebSocket Benchmark](https://i.imgur.com/Iv2WzJV.jpg)

```rust
#[cfg(feature = "web-socket")]
mod web_socket {
  use wtx::{
    misc::Stream,
    rng::Rng,
    web_socket::{
      FrameBufferVec, FrameMutVec, FrameVecMut, compression::NegotiatedCompression, OpCode,
      WebSocketClientOwned
    }
  };
  
  pub async fn handle_client_frames(
    fb: &mut FrameBufferVec,
    ws: &mut WebSocketClientOwned<impl NegotiatedCompression, impl Rng, impl Stream>
  ) -> wtx::Result<()> {
    loop {
      let frame = match ws.read_frame(fb).await {
        Err(err) => {
          println!("Error: {err}");
          ws.write_frame(&mut FrameMutVec::new_fin(fb, OpCode::Close, &[])?).await?;
          break;
        }
        Ok(elem) => elem,
      };
      match (frame.op_code(), frame.text_payload()) {
        (_, Some(elem)) => println!("{elem}"),
        (OpCode::Close, _) => break,
        _ => {}
      }
    }
    Ok(())
  }
}
```

See the `examples` directory for more suggestions.

## Autobahn

All the `fuzzingclient`/`fuzzingserver` tests provided by the Autobahn|Testsuite project are passing and the full reports can found at <https://c410-f3r.github.io/wtx>.

## Compression

The "permessage-deflate" extension, described in [RFC-7692](https://datatracker.ietf.org/doc/html/rfc7692), is the only supported compression format and is backed by the fastest compression library available at the current time, which is "zlib-ng". It also means that a C compiler is required to use such a feature.
