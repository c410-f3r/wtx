use crate::{
  calendar::{DateTime, Utc},
  database::{
    DatabaseTy, Identifier,
    schema_manager::{DbMigrationGroup, Uid, migration::migration_common::MigrationCommon},
  },
};
use core::fmt;

/// Migration retrieved from a database.
#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct DbMigration {
  common: MigrationCommon<Identifier>,
  created_on: DateTime<Utc>,
  db_ty: DatabaseTy,
  group: DbMigrationGroup<Identifier>,
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
  pub fn group(&self) -> &DbMigrationGroup<Identifier> {
    &self.group
  }

  /// Name
  #[inline]
  pub fn name(&self) -> &str {
    &self.common.name
  }

  /// User ID
  #[inline]
  pub fn uid(&self) -> Uid {
    self.common.uid
  }
}

#[cfg(feature = "mysql")]
impl<'exec, E> crate::database::FromRecords<'exec, crate::database::client::mysql::Mysql<E>>
  for DbMigration
where
  E: From<crate::Error>,
{
  const ID_IDX: Option<usize> = None;
  type IdTy = ();

  #[inline]
  fn from_records(
    curr_params: &mut crate::database::FromRecordsParams<
      crate::database::client::mysql::MysqlRecord<'exec, E>,
    >,
    _: &crate::database::client::mysql::MysqlRecords<'exec, E>,
  ) -> Result<Self, E> {
    use crate::database::Record as _;
    let rslt = Self {
      common: MigrationCommon {
        checksum: checksum_from_str(curr_params.curr_record.decode("checksum")?)?,
        name: curr_params.curr_record.decode::<_, &str>("name")?.try_into()?,
        repeatability: from_u32(curr_params.curr_record.decode_opt("repeatability")?),
        uid: curr_params.curr_record.decode("uid")?,
      },
      created_on: curr_params.curr_record.decode("created_on")?,
      db_ty: DatabaseTy::Mysql,
      group: DbMigrationGroup::new(
        curr_params.curr_record.decode::<_, &str>("mg_name")?.try_into()?,
        curr_params.curr_record.decode("mg_uid")?,
        curr_params.curr_record.decode("mg_version")?,
      ),
    };
    curr_params.inc_consumed_records(1);
    Ok(rslt)
  }
}

#[cfg(feature = "postgres")]
impl<'exec, E> crate::database::FromRecords<'exec, crate::database::client::postgres::Postgres<E>>
  for DbMigration
where
  E: From<crate::Error>,
{
  const ID_IDX: Option<usize> = None;
  type IdTy = ();

  #[inline]
  fn from_records(
    curr_params: &mut crate::database::FromRecordsParams<
      crate::database::client::postgres::PostgresRecord<'exec, E>,
    >,
    _: &crate::database::client::postgres::PostgresRecords<'exec, E>,
  ) -> Result<Self, E> {
    use crate::database::Record as _;
    let rslt = Self {
      common: MigrationCommon {
        checksum: checksum_from_str(curr_params.curr_record.decode("checksum")?)?,
        name: curr_params.curr_record.decode::<_, &str>("name")?.try_into()?,
        repeatability: from_u32(curr_params.curr_record.decode_opt("repeatability")?),
        uid: curr_params.curr_record.decode("uid")?,
      },
      created_on: curr_params.curr_record.decode("created_on")?,
      db_ty: DatabaseTy::Postgres,
      group: DbMigrationGroup::new(
        curr_params.curr_record.decode::<_, &str>("mg_name")?.try_into()?,
        curr_params.curr_record.decode("mg_uid")?,
        curr_params.curr_record.decode("mg_version")?,
      ),
    };
    curr_params.inc_consumed_records(1);
    Ok(rslt)
  }
}

impl fmt::Display for DbMigration {
  #[inline]
  fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(fmt, "{}__{}", self.common.uid, self.common.name)
  }
}

#[cfg(any(feature = "mysql", feature = "postgres"))]
fn checksum_from_str(bytes: &[u8]) -> crate::Result<u64> {
  use crate::misc::FromRadix10;
  Ok(
    u64::from_radix_10(bytes)
      .map_err(|_err| crate::database::schema_manager::SchemaManagerError::ChecksumMustBeANumber)?,
  )
}

#[cfg(any(feature = "mysql", feature = "postgres"))]
fn from_u32(n: Option<u32>) -> Option<crate::database::schema_manager::Repeatability> {
  match n? {
    0 => Some(crate::database::schema_manager::Repeatability::Always),
    _ => Some(crate::database::schema_manager::Repeatability::OnChecksumChange),
  }
}
