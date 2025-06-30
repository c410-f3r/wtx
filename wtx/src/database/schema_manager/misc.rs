//! Miscellaneous

#[cfg(feature = "std")]
macro_rules! opt_to_inv_mig {
  ($opt:expr) => {
    $opt().ok_or_else(|| SchemaManagerError::InvalidMigration)
  };
}

use crate::{
  collection::IndexedStorageMut,
  database::{
    DatabaseTy,
    schema_manager::{
      SchemaManagerError, Uid,
      migration::{DbMigration, UserMigration},
    },
  },
  misc::Lease,
};
use core::hash::{Hash, Hasher};
#[cfg(feature = "std")]
use {
  crate::{
    collection::{ArrayVectorU8, Vector},
    database::schema_manager::{
      Repeatability, UserMigrationGroup, UserMigrationOwned,
      toml_parser::{Expr, toml},
    },
  },
  alloc::string::String,
  core::{cmp::Ordering, fmt::Write},
  std::{
    fs::{DirEntry, File, read_to_string},
    io::Read,
    path::{Path, PathBuf},
  },
};

#[cfg(feature = "std")]
type MigrationGroupParts = (String, Uid);
#[cfg(feature = "std")]
type MigrationParts = (
  ArrayVectorU8<DatabaseTy, { DatabaseTy::len() }>,
  String,
  Option<Repeatability>,
  String,
  String,
  Uid,
);

/// All files of a given `path`.
#[cfg(feature = "std")]
#[inline]
pub fn files(dir: &Path) -> crate::Result<impl Iterator<Item = crate::Result<DirEntry>>> {
  Ok(read_dir(dir)?.filter_map(|entry_rslt| {
    let entry = entry_rslt.ok()?;
    let path = entry.path();
    path.is_file().then(|| Ok(entry))
  }))
}

/// All migrations and their related group that are located in a given `path`.`
#[cfg(feature = "std")]
#[inline]
pub fn group_and_migrations_from_path<F>(
  path: &Path,
  cb: F,
) -> crate::Result<(
  UserMigrationGroup<String>,
  impl Iterator<Item = crate::Result<UserMigrationOwned>>,
)>
where
  F: FnMut(&PathBuf, &PathBuf) -> Ordering,
{
  use crate::{
    collection::ArrayStringU8,
    database::schema_manager::{
      SchemaManagerError,
      migration_parser::{parse_migration_toml, parse_unified_migration},
    },
  };

  fn group_and_migrations_from_path<F>(
    path: &Path,
    cb: F,
  ) -> crate::Result<(MigrationGroupParts, impl Iterator<Item = crate::Result<MigrationParts>>)>
  where
    F: FnMut(&PathBuf, &PathBuf) -> Ordering,
  {
    let (mg, mut migrations_vec) = migrations_from_dir(path)?;
    migrations_vec.sort_by(cb);
    let migrations = migrations_vec.into_iter().map(move |local_path| {
      let name;
      let uid;
      let mut dbs = ArrayVectorU8::new();
      let mut repeatability = None;
      let mut sql_down = String::default();
      let mut sql_up = String::default();

      if local_path.is_dir() {
        let dir_name = opt_to_inv_mig!(|| local_path.file_name()?.to_str())?;
        let parts = dir_name_parts(dir_name)?;
        name = parts.0;
        uid = parts.1;

        let mut cfg_file_name = ArrayStringU8::<64>::new();
        cfg_file_name.write_fmt(format_args!("{dir_name}.toml"))?;

        let mut down_file_name = ArrayStringU8::<64>::new();
        down_file_name.write_fmt(format_args!("{dir_name}_down.sql"))?;

        let mut up_file_name = ArrayStringU8::<64>::new();
        up_file_name.write_fmt(format_args!("{dir_name}_up.sql"))?;

        for file_rslt in files(local_path.as_path())? {
          let file = file_rslt?;
          let file_path = file.path();
          let file_name = opt_to_inv_mig!(|| file_path.file_name()?.to_str())?;
          if file_name == cfg_file_name.as_str() {
            let mc = parse_migration_toml(File::open(file_path)?)?;
            dbs = mc.dbs;
            repeatability = mc.repeatability;
          } else if file_name == down_file_name.as_str() {
            sql_down = read_to_string(file_path)?;
          } else if file_name == up_file_name.as_str() {
            sql_up = read_to_string(file_path)?;
          } else {
            continue;
          }
        }
      } else if let Some(Some(file_name)) = local_path.file_name().map(|e| e.to_str()) {
        let parts = migration_file_name_parts(file_name)?;
        name = parts.0;
        uid = parts.1;
        let pm = parse_unified_migration(File::open(local_path)?)?;
        dbs = pm.cfg.dbs;
        repeatability = pm.cfg.repeatability;
        sql_up = pm.sql_in;
        sql_down = pm.sql_out;
      } else {
        return Err(SchemaManagerError::InvalidMigration.into());
      }
      Ok((dbs, name, repeatability, sql_down, sql_up, uid))
    });

    Ok((mg, migrations))
  }

  let ((mg_name, mg_uid), ms) = group_and_migrations_from_path(path, cb)?;
  let mg = UserMigrationGroup::new(mg_name, mg_uid);
  let mapped = ms.map(|rslt| {
    let (dbs, name, repeatability, sql_down, sql_up, uid) = rslt?;
    UserMigrationOwned::from_user_parts(dbs, name, repeatability, [sql_up, sql_down], uid)
  });
  Ok((mg, mapped))
}

/// All paths to directories that contain migrations and optional seeds
#[cfg(feature = "std")]
#[inline]
pub fn parse_root_toml(cfg_path: &Path) -> crate::Result<(Vector<PathBuf>, Option<PathBuf>)> {
  let cfg_dir = cfg_path.parent().unwrap_or_else(|| Path::new("."));
  parse_root_toml_raw(File::open(cfg_path)?, cfg_dir)
}

/// Similar to `parse_root_toml`, takes a stream of bytes and a base path as arguments.
#[cfg(feature = "std")]
#[inline]
pub fn parse_root_toml_raw<R>(
  read: R,
  root: &Path,
) -> crate::Result<(Vector<PathBuf>, Option<PathBuf>)>
where
  R: Read,
{
  let mut migration_groups = Vector::new();
  let mut seeds = None;

  for (ident, toml_expr) in toml(read)? {
    match (ident.as_str(), toml_expr) {
      ("migration_groups", Expr::Array(array)) => {
        for elem in array.into_iter() {
          let path = root.join(elem.as_str());
          let name_opt = || path.file_name()?.to_str();
          let Some(name) = name_opt() else {
            continue;
          };
          if elem.is_empty() || !path.is_dir() || dir_name_parts(name).is_err() {
            continue;
          }
          migration_groups.push(path)?;
        }
      }
      ("seeds", Expr::String(elem)) => {
        let path = root.join(elem.as_str());
        if !path.is_dir() {
          continue;
        }
        seeds = Some(path);
      }
      _ => {}
    }
  }

  Ok((migration_groups, seeds))
}

pub(crate) fn calc_checksum(name: &str, sql_up: &str, sql_down: &str, uid: Uid) -> u64 {
  #[expect(
    deprecated,
    reason = "IFAICT, this use-case doesn't require a different hashing algorithm"
  )]
  let mut hasher = core::hash::SipHasher::new();
  name.hash(&mut hasher);
  sql_up.hash(&mut hasher);
  sql_down.hash(&mut hasher);
  uid.hash(&mut hasher);
  hasher.finish()
}

pub(crate) fn is_migration_divergent<DBS, S>(
  db_migrations: &[DbMigration],
  migration: &UserMigration<DBS, S>,
) -> bool
where
  DBS: Lease<[DatabaseTy]>,
  S: Lease<str>,
{
  let uid = migration.uid();
  let opt = binary_search_migration_by_uid(db_migrations, uid);
  let Some(db_migration) = opt else {
    return false;
  };
  migration.checksum() != db_migration.checksum()
    || migration.name() != db_migration.name()
    || migration.uid() != db_migration.uid()
}

pub(crate) fn is_sorted_and_unique<T>(slice: &[T]) -> crate::Result<()>
where
  T: PartialOrd,
{
  let mut iter = slice.windows(2);
  while let Some([first, second, ..]) = iter.next() {
    if first >= second {
      return Err(SchemaManagerError::DatabasesMustBeSortedAndUnique.into());
    }
  }
  Ok(())
}

fn binary_search_migration_by_uid(migrations: &[DbMigration], uid: Uid) -> Option<&DbMigration> {
  match migrations.binary_search_by(|m| m.uid().cmp(&uid)) {
    Err(_) => None,
    Ok(rslt) => migrations.get(rslt),
  }
}

#[cfg(feature = "std")]
fn dir_name_parts(s: &str) -> crate::Result<(String, Uid)> {
  let f = || {
    if !s.is_ascii() {
      return None;
    }
    let mut split = s.split("__");
    let uid = split.next()?.parse().ok()?;
    let name = split.next()?.into();
    Some((name, uid))
  };
  f().ok_or(SchemaManagerError::InvalidMigration.into())
}

#[cfg(feature = "std")]
fn migration_file_name_parts(s: &str) -> crate::Result<(String, Uid)> {
  let f = || {
    if !s.is_ascii() {
      return None;
    }
    let mut split = s.split("__");
    let uid = split.next()?.parse().ok()?;
    let name = split.next()?.strip_suffix(".sql")?.into();
    Some((name, uid))
  };
  f().ok_or(SchemaManagerError::InvalidMigration.into())
}

#[cfg(feature = "std")]
fn migrations_from_dir(path: &Path) -> crate::Result<(MigrationGroupParts, Vector<PathBuf>)> {
  let path_str = opt_to_inv_mig!(|| path.file_name()?.to_str())?;
  let (mg_name, mg_uid) = dir_name_parts(path_str)?;
  let mut migration_paths = Vector::new();
  for rslt in read_dir(path)? {
    migration_paths.push(rslt?.path())?;
  }
  Ok(((mg_name, mg_uid), migration_paths))
}

#[cfg(feature = "std")]
fn read_dir(dir: &Path) -> crate::Result<impl Iterator<Item = crate::Result<DirEntry>>> {
  Ok(std::fs::read_dir(dir)?.map(|entry_rslt| entry_rslt.map_err(Into::into)))
}
