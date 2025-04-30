// Many commands were retrieved from the flyway project (https://github.com/flyway) so credits to
// the authors.

macro_rules! _wtx_migration_columns {
  () => {
    "_wtx_migration_mg_uid INT NOT NULL, \
    checksum VARCHAR(20) NOT NULL, \
    name VARCHAR(128) NOT NULL, \
    repeatability INTEGER NULL, \
    uid INT NOT NULL, \
    CONSTRAINT _wtx_migration_unq UNIQUE (uid, _wtx_migration_mg_uid)"
  };
}

macro_rules! _wtx_migration_group_columns {
  () => {
    "uid INT NOT NULL PRIMARY KEY, \
    name VARCHAR(128) NOT NULL,
    version INT NOT NULL"
  };
}

macro_rules! _serial_id {
  () => {
    "id SERIAL NOT NULL PRIMARY KEY,"
  };
}

#[cfg(any(feature = "mysql", feature = "postgres"))]
pub(crate) mod common;
#[cfg(feature = "mysql")]
pub(crate) mod mysql;
#[cfg(feature = "postgres")]
pub(crate) mod postgres;
