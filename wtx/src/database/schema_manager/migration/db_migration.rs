use crate::{
  database::{
    DatabaseTy, Identifier,
    schema_manager::{
      MigrationGroup, Repeatability, SchemaManagerError, VersionTy,
      migration::migration_common::MigrationCommon,
    },
  },
  misc::FromRadix10,
};
use chrono::{DateTime, NaiveDateTime, Utc};
use core::fmt;

/// Migration retrieved from a database.
#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct DbMigration {
  common: MigrationCommon<Identifier>,
  created_on: DateTime<Utc>,
  db_ty: DatabaseTy,
  group: MigrationGroup<Identifier>,
}

impl DbMigration {
  /// Data integrity
  #[inline]
  pub fn checksum(&self) -> u64 {
    self.common.checksum
  }

  /// When the migration was created.
  #[inline]
  pub fn created_on(&self) -> &DateTime<Utc> {
    &self.created_on
  }

  /// See [DatabaseTy].
  #[inline]
  pub fn db_ty(&self) -> DatabaseTy {
    self.db_ty
  }

  /// Group
  #[inline]
  pub fn group(&self) -> &MigrationGroup<Identifier> {
    &self.group
  }

  /// Name
  #[inline]
  pub fn name(&self) -> &str {
    &self.common.name
  }

  /// Version
  #[inline]
  pub fn version(&self) -> VersionTy {
    self.common.version
  }
}

#[cfg(feature = "mysql")]
impl<E> crate::database::FromRecord<crate::database::client::mysql::Mysql<E>> for DbMigration
where
  E: From<crate::Error>,
{
  #[inline]
  fn from_record(from: &crate::database::client::mysql::MysqlRecord<'_, E>) -> Result<Self, E> {
    use crate::database::Record as _;
    let checksum = _checksum_from_str(from.decode("checksum")?)?;
    let name = from.decode::<_, &str>("name")?.try_into()?;
    let repeatability = _from_u32(from.decode_opt("repeatability")?);
    let version = from.decode("version")?;
    Ok(Self {
      common: MigrationCommon { checksum, name, repeatability, version },
      created_on: from.decode("created_on")?,
      db_ty: DatabaseTy::Mysql,
      group: MigrationGroup::new(
        from.decode::<_, &str>("omg_name")?.try_into()?,
        from.decode("omg_version")?,
      ),
    })
  }
}

#[cfg(feature = "postgres")]
impl<E> crate::database::FromRecord<crate::database::client::postgres::Postgres<E>> for DbMigration
where
  E: From<crate::Error>,
{
  #[inline]
  fn from_record(
    from: &crate::database::client::postgres::PostgresRecord<'_, E>,
  ) -> Result<Self, E> {
    use crate::database::Record as _;
    Ok(Self {
      common: MigrationCommon {
        checksum: _checksum_from_str(from.decode("checksum")?)?,
        name: from.decode::<_, &str>("name")?.try_into()?,
        repeatability: _from_u32(from.decode_opt("repeatability")?),
        version: from.decode("version")?,
      },
      created_on: from.decode("created_on")?,
      db_ty: DatabaseTy::Postgres,
      group: MigrationGroup::new(
        from.decode::<_, &str>("omg_name")?.try_into()?,
        from.decode("omg_version")?,
      ),
    })
  }
}

impl fmt::Display for DbMigration {
  #[inline]
  fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(fmt, "{}__{}", self.common.version, self.common.name)
  }
}

#[inline]
fn _checksum_from_str(bytes: &[u8]) -> crate::Result<u64> {
  Ok(u64::from_radix_10(bytes).map_err(|_err| SchemaManagerError::ChecksumMustBeANumber)?)
}

#[inline]
fn _fixed_from_naive_utc(naive: NaiveDateTime) -> DateTime<Utc> {
  DateTime::<Utc>::from_naive_utc_and_offset(naive, Utc)
}

#[inline]
fn _from_u32(n: Option<u32>) -> Option<Repeatability> {
  match n? {
    0 => Some(Repeatability::Always),
    _ => Some(Repeatability::OnChecksumChange),
  }
}

#[inline]
fn _mssql_date_hack(s: &str) -> crate::Result<DateTime<Utc>> {
  let naive_rslt = NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S");
  let naive = naive_rslt?;
  Ok(_fixed_from_naive_utc(naive))
}
