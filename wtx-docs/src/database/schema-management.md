# Schema Management

Embedded and CLI workflows using raw SQL commands.

Activation feature is called `schema-manager`.

## CLI

```bash
# Example

cargo install --git https://github.com/c410-f3r/wtx --features schema-manager-dev wtx-ui
echo DATABASE_URI="postgres://USER:PASSWORD@localhost:5432/DATABASE" > .env
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

## Library

The library gives freedom to arrange groups and uses some external crates, bringing ~10 additional dependencies into your application. If this overhead is not acceptable, then you probably should discard the library and use the CLI binary instead as part of a custom deployment strategy.

```rust,edition2021,no_run
extern crate tokio;
extern crate wtx;

use std::path::Path;
use wtx::database::{schema_manager::Commands, DEFAULT_URI_VAR};

#[tokio::main]
async fn main() {
  let mut commands = Commands::with_executor(());
  commands
    .migrate_from_dir(
      (&mut String::default(), &mut Vec::default()),
      Path::new("my_custom_migration_group_path"),
    )
    .await
    .unwrap();
}
```

## Embedded migrations

To make deployment easier, the final binary of your application can embed all necessary migrations through the binary that is available in the `wtx-ui` crate.

```rust,edition2021,no_run
extern crate wtx;

// This is an example! The actual contents are filled by the `wtx-ui embed-migrations` binary call.
mod embedded_migrations {
  pub(crate) const GROUPS: wtx::database::schema_manager::EmbeddedMigrationsTy = &[];
}

use wtx::database::schema_manager::Commands;

async fn migrate() -> wtx::Result<()> {
  Commands::with_executor(())
    .migrate_from_groups((&mut String::new(), &mut Vec::new()), embedded_migrations::GROUPS)
    .await
}
```

## Conditional migrations

If one particular migration needs to be executed in a specific set of databases, then it is possible to use the `-- wtx dbs` parameter in a file.

```sql
-- wtx dbs mssql,postgres

-- wtx IN

CREATE SCHEMA foo;

-- wtx OUT

DROP SCHEMA foo;
```

## Repeatable migrations

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

## Namespaces/Schemas

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