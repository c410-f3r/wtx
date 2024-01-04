//! Miscellaneous

#[cfg(feature = "std")]
macro_rules! opt_to_inv_mig {
  ($opt:expr) => {
    $opt().ok_or_else(|| crate::Error::InvalidMigration)
  };
}

use crate::database::{
  schema_manager::migration::{DbMigration, UserMigration},
  DatabaseTy,
};
use core::hash::{Hash, Hasher};
#[cfg(feature = "std")]
use {
  crate::database::schema_manager::{
    toml_parser::{toml, Expr, EXPR_ARRAY_MAX_LEN},
    MigrationGroup, Repeatability, UserMigrationOwned,
  },
  arrayvec::ArrayString,
  arrayvec::ArrayVec,
  core::cmp::Ordering,
  std::path::{Path, PathBuf},
  std::{
    fmt::Write,
    fs::{read_to_string, DirEntry, File},
    io::Read,
  },
};

#[cfg(feature = "std")]
type MigrationGroupParts = (String, i32);
#[cfg(feature = "std")]
type MigrationParts =
  (ArrayVec<DatabaseTy, { DatabaseTy::len() }>, String, Option<Repeatability>, String, String, i32);

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
) -> crate::Result<(MigrationGroup<String>, impl Iterator<Item = crate::Result<UserMigrationOwned>>)>
where
  F: FnMut(&PathBuf, &PathBuf) -> Ordering,
{
  use crate::database::schema_manager::migration_parser::{
    parse_migration_toml, parse_unified_migration,
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
      let mut dbs = ArrayVec::default();
      let name;
      let mut repeatability = None;
      let mut sql_down = String::default();
      let mut sql_up = String::default();
      let version;

      if local_path.is_dir() {
        let dir_name = opt_to_inv_mig!(|| local_path.file_name()?.to_str())?;
        let parts = dir_name_parts(dir_name)?;
        name = parts.0;
        version = parts.1;

        let mut cfg_file_name = ArrayString::<64>::new();
        cfg_file_name.write_fmt(format_args!("{dir_name}.toml"))?;

        let mut down_file_name = ArrayString::<64>::new();
        down_file_name.write_fmt(format_args!("{dir_name}_down.sql"))?;

        let mut up_file_name = ArrayString::<64>::new();
        up_file_name.write_fmt(format_args!("{dir_name}_up.sql"))?;

        for file_rslt in files(local_path.as_path())? {
          let file = file_rslt?;
          let file_path = file.path();
          let file_name = opt_to_inv_mig!(|| file_path.file_name()?.to_str())?;
          if file_name == &cfg_file_name {
            let mc = parse_migration_toml(File::open(file_path)?)?;
            dbs = mc.dbs;
            repeatability = mc.repeatability;
          } else if file_name == &down_file_name {
            sql_down = read_to_string(file_path)?;
          } else if file_name == &up_file_name {
            sql_up = read_to_string(file_path)?;
          } else {
            continue;
          }
        }
      } else if let Some(Some(file_name)) = local_path.file_name().map(|e| e.to_str()) {
        let parts = migration_file_name_parts(file_name)?;
        name = parts.0;
        version = parts.1;
        let pm = parse_unified_migration(File::open(local_path)?)?;
        dbs = pm.cfg.dbs;
        repeatability = pm.cfg.repeatability;
        sql_up = pm.sql_in;
        sql_down = pm.sql_out;
      } else {
        return Err(crate::Error::InvalidMigration);
      }
      Ok((dbs, name, repeatability, sql_down, sql_up, version))
    });

    Ok((mg, migrations))
  }

  let ((mg_name, mg_version), ms) = group_and_migrations_from_path(path, cb)?;
  let mg = MigrationGroup::new(mg_name, mg_version);
  let mapped = ms.map(|rslt| {
    let (dbs, name, repeatability, sql_down, sql_up, version) = rslt?;
    UserMigrationOwned::from_user_parts(dbs, name, repeatability, [sql_up, sql_down], version)
  });
  Ok((mg, mapped))
}

/// All paths to directories that contain migrations and optional seeds
#[cfg(feature = "std")]
#[inline]
pub fn parse_root_toml(
  cfg_path: &Path,
) -> crate::Result<(ArrayVec<PathBuf, EXPR_ARRAY_MAX_LEN>, Option<PathBuf>)> {
  let cfg_dir = cfg_path.parent().unwrap_or_else(|| Path::new("."));
  parse_root_toml_raw(File::open(cfg_path)?, cfg_dir)
}

/// Similar to `parse_root_toml`, takes a stream of bytes and a base path as arguments.
#[cfg(feature = "std")]
#[inline]
pub fn parse_root_toml_raw<R>(
  read: R,
  root: &Path,
) -> crate::Result<(ArrayVec<PathBuf, EXPR_ARRAY_MAX_LEN>, Option<PathBuf>)>
where
  R: Read,
{
  let mut migration_groups = ArrayVec::new();
  let mut seeds = None;

  for (ident, toml_expr) in toml(read)? {
    match (ident.as_ref(), toml_expr) {
      ("migration_groups", Expr::Array(array)) => {
        for elem in array {
          let path = root.join(elem.as_str());
          let name_opt = || path.file_name()?.to_str();
          let Some(name) = name_opt() else {
            continue;
          };
          if elem.is_empty() || !path.is_dir() || dir_name_parts(name).is_err() {
            continue;
          }
          migration_groups.try_push(path)?;
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

#[inline]
pub(crate) fn calc_checksum(name: &str, sql_up: &str, sql_down: &str, version: i32) -> u64 {
  #[allow(deprecated)]
  let mut hasher = core::hash::SipHasher::new();
  name.hash(&mut hasher);
  sql_up.hash(&mut hasher);
  sql_down.hash(&mut hasher);
  version.hash(&mut hasher);
  hasher.finish()
}

#[inline]
pub(crate) fn is_migration_divergent<DBS, S>(
  db_migrations: &[DbMigration],
  migration: &UserMigration<DBS, S>,
) -> bool
where
  DBS: AsRef<[DatabaseTy]>,
  S: AsRef<str>,
{
  let version = migration.version();
  let opt = binary_search_migration_by_version(version, db_migrations);
  let Some(db_migration) = opt else {
    return false;
  };
  migration.checksum() != db_migration.checksum()
    || migration.name() != db_migration.name()
    || migration.version() != db_migration.version()
}

pub(crate) fn is_sorted_and_unique<T>(slice: &[T]) -> crate::Result<()>
where
  T: PartialOrd,
{
  let mut iter = slice.windows(2);
  while let Some([first, second, ..]) = iter.next() {
    if first >= second {
      return Err(crate::Error::DatabasesMustBeSortedAndUnique);
    }
  }
  Ok(())
}

#[inline]
fn binary_search_migration_by_version(
  version: i32,
  migrations: &[DbMigration],
) -> Option<&DbMigration> {
  match migrations.binary_search_by(|m| m.version().cmp(&version)) {
    Err(_) => None,
    Ok(rslt) => migrations.get(rslt),
  }
}

#[cfg(feature = "std")]
#[inline]
fn dir_name_parts(s: &str) -> crate::Result<(String, i32)> {
  let f = || {
    if !s.is_ascii() {
      return None;
    }
    let mut split = s.split("__");
    let version = split.next()?.parse::<i32>().ok()?;
    let name = split.next()?.into();
    Some((name, version))
  };
  f().ok_or(crate::Error::InvalidMigration)
}

#[cfg(feature = "std")]
#[inline]
fn migration_file_name_parts(s: &str) -> crate::Result<(String, i32)> {
  let f = || {
    if !s.is_ascii() {
      return None;
    }
    let mut split = s.split("__");
    let version = split.next()?.parse::<i32>().ok()?;
    let name = split.next()?.strip_suffix(".sql")?.into();
    Some((name, version))
  };
  f().ok_or(crate::Error::InvalidMigration)
}

#[cfg(feature = "std")]
#[inline]
fn migrations_from_dir(path: &Path) -> crate::Result<(MigrationGroupParts, Vec<PathBuf>)> {
  let path_str = opt_to_inv_mig!(|| path.file_name()?.to_str())?;
  let (mg_name, mg_version) = dir_name_parts(path_str)?;
  let migration_paths = read_dir(path)?
    .map(|entry_rslt| Ok(entry_rslt?.path()))
    .collect::<crate::Result<Vec<PathBuf>>>()?;
  Ok(((mg_name, mg_version), migration_paths))
}

#[cfg(feature = "std")]
#[inline]
fn read_dir(dir: &Path) -> crate::Result<impl Iterator<Item = crate::Result<DirEntry>>> {
  Ok(std::fs::read_dir(dir)?.map(|entry_rslt| entry_rslt.map_err(Into::into)))
}
