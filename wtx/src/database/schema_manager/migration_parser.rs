//! Migration file parser

use crate::database::{
  schema_manager::{
    toml_parser::{toml, Expr},
    Repeatability,
  },
  DatabaseTy,
};
use arrayvec::ArrayVec;
use std::io::{BufRead, BufReader, Read};

/// Auxiliary parameters of a migration file
#[derive(Debug, Default)]
pub struct MigrationCfg {
  /// All unique declared databases
  pub dbs: ArrayVec<DatabaseTy, { DatabaseTy::len() }>,
  /// Declared repeatability
  pub repeatability: Option<Repeatability>,
}

/// In-memory representation of a parsed migration file
#[derive(Debug, Default)]
pub struct ParsedMigration {
  /// See [MigrationCfg].
  pub cfg: MigrationCfg,
  /// -- wtx IN contents
  pub sql_in: String,
  /// -- wtx OUT contents
  pub sql_out: String,
}

/// Gets all information related to a migration from a reading source.
#[inline]
pub fn parse_unified_migration<R>(read: R) -> crate::Result<ParsedMigration>
where
  R: Read,
{
  let mut br = BufReader::new(read);
  let mut overall_buffer = String::with_capacity(16);
  let mut parsed_migration = ParsedMigration::default();

  iterations(&mut overall_buffer, &mut br, |_| false)?;

  if let Some(rslt) = overall_buffer.split("-- wtx dbs").nth(1) {
    for db_str in rslt.split(',') {
      if let Ok(db) = db_str.trim().try_into() {
        let is_not_already_inserted = !parsed_migration.cfg.dbs.contains(&db);
        if is_not_already_inserted {
          parsed_migration.cfg.dbs.try_push(db)?;
        }
      }
    }
    iterations(&mut overall_buffer, &mut br, |_| false)?;
  }

  if let Some(rslt) = overall_buffer.split("-- wtx repeatability").nth(1) {
    if let Ok(repeatability) = rslt.trim().try_into() {
      parsed_migration.cfg.repeatability = Some(repeatability);
    }
    iterations(&mut overall_buffer, &mut br, |_| false)?;
  }

  if !overall_buffer.contains("-- wtx IN") {
    return Err(crate::Error::IncompleteSqlFile);
  }

  iterations(&mut overall_buffer, &mut br, |str_read| !str_read.contains("-- wtx OUT"))?;

  if let Some(rslt) = overall_buffer.rsplit("-- wtx OUT").nth(1) {
    parsed_migration.sql_in = rslt.trim().into();
  } else {
    parsed_migration.sql_in = overall_buffer.trim().into();
    return Ok(parsed_migration);
  }

  iterations(&mut overall_buffer, &mut br, |_| true)?;

  parsed_migration.sql_out = overall_buffer.trim().into();

  if parsed_migration.sql_in.is_empty() {
    return Err(crate::Error::IncompleteSqlFile);
  }

  Ok(parsed_migration)
}

/// Gets all information related to a migration from a reading source.
#[inline]
pub(crate) fn parse_migration_toml<R>(read: R) -> crate::Result<MigrationCfg>
where
  R: Read,
{
  let mut migration_toml = MigrationCfg { dbs: ArrayVec::new(), repeatability: None };

  for (ident, toml_expr) in toml(read)? {
    match (ident.as_ref(), toml_expr) {
      ("dbs", Expr::Array(array)) => {
        for s in array {
          let Ok(elem) = s.as_str().try_into() else {
            continue;
          };
          migration_toml.dbs.try_push(elem)?;
        }
      }
      ("repeatability", Expr::String(s)) => {
        let Ok(elem) = s.as_str().try_into() else {
          continue;
        };
        migration_toml.repeatability = Some(elem);
      }
      _ => {}
    }
  }

  Ok(migration_toml)
}

#[inline]
fn iterations<F, R>(
  overall_buffer: &mut String,
  br: &mut BufReader<R>,
  mut cb: F,
) -> crate::Result<()>
where
  F: FnMut(&str) -> bool,
  R: Read,
{
  overall_buffer.clear();
  let mut bytes_read = 0;

  loop {
    let curr_bytes_read = br.read_line(overall_buffer)?;

    if curr_bytes_read == 0 {
      break;
    }

    let Some(str_read) = overall_buffer.get(bytes_read..) else { break };
    let trimmed = str_read.trim();

    bytes_read = bytes_read.saturating_add(curr_bytes_read);

    if trimmed.is_empty() || trimmed.starts_with("//") {
      continue;
    }

    if !cb(trimmed) {
      break;
    }
  }

  Ok(())
}

#[cfg(test)]
mod tests {
  use crate::database::{
    schema_manager::{migration_parser::parse_unified_migration, Repeatability},
    DatabaseTy,
  };

  #[test]
  fn does_not_take_into_consideration_white_spaces_and_comments() {
    let s = "// FOO\n\t\n-- wtx IN\nSOMETHING\nFOO\n";
    let rslt = parse_unified_migration(s.as_bytes()).unwrap();
    assert_eq!("SOMETHING\nFOO", rslt.sql_in);
  }

  #[test]
  fn must_have_obrigatory_params() {
    assert!(parse_unified_migration(&[][..]).is_err());
  }

  #[test]
  fn parses_optional_dbs() {
    let s = "-- wtx IN\nSOMETHING";
    let no_declaration = parse_unified_migration(s.as_bytes()).unwrap();
    assert!(no_declaration.cfg.dbs.is_empty());

    let s = "-- wtx dbs\n-- wtx IN\nSOMETHING";
    let with_initial_declaration = parse_unified_migration(s.as_bytes()).unwrap();
    assert!(with_initial_declaration.cfg.dbs.is_empty());

    let s = "-- wtx dbs bird,apple\n-- wtx IN\nSOMETHING";
    let with_incorrect_declaration = parse_unified_migration(s.as_bytes()).unwrap();
    assert!(with_incorrect_declaration.cfg.dbs.is_empty());

    let s = "-- wtx dbs mssql,postgres\n-- wtx IN\nSOMETHING";
    let two_dbs = parse_unified_migration(s.as_bytes()).unwrap();
    assert_eq!(two_dbs.cfg.dbs[0], DatabaseTy::Mssql);
    assert_eq!(two_dbs.cfg.dbs[1], DatabaseTy::Postgres);
  }

  #[test]
  fn parses_down() {
    let s = "\n-- wtx IN\n\nSOMETHING\nFOO\n\n-- wtx OUT\n\nBAR\n";
    let rslt = parse_unified_migration(s.as_bytes()).unwrap();
    assert_eq!("SOMETHING\nFOO", rslt.sql_in);
    assert_eq!("BAR", rslt.sql_out);
  }

  #[test]
  fn parses_repeatability() {
    let s = "-- wtx IN\nSOMETHING";
    let no_declaration = parse_unified_migration(s.as_bytes()).unwrap();
    assert!(no_declaration.cfg.repeatability.is_none());

    let s = "-- wtx repeatability\n-- wtx IN\nSOMETHING";
    let with_initial_declaration = parse_unified_migration(s.as_bytes()).unwrap();
    assert!(with_initial_declaration.cfg.repeatability.is_none());

    let s = "-- wtx repeatability FOO\n-- wtx IN\nSOMETHING";
    let with_incorrect_declaration = parse_unified_migration(s.as_bytes()).unwrap();
    assert!(with_incorrect_declaration.cfg.repeatability.is_none());

    let s = "-- wtx repeatability always\n-- wtx IN\nSOMETHING";
    let always = parse_unified_migration(s.as_bytes()).unwrap();
    assert_eq!(always.cfg.repeatability, Some(Repeatability::Always));

    let s = "-- wtx repeatability on-checksum-change\n-- wtx IN\nSOMETHING";
    let on_checksum_change = parse_unified_migration(s.as_bytes()).unwrap();
    assert_eq!(on_checksum_change.cfg.repeatability, Some(Repeatability::OnChecksumChange));
  }

  #[test]
  fn parses_mandatory_params() {
    let s = "-- wtx IN\n\nSOMETHING\nFOO";
    let rslt = parse_unified_migration(s.as_bytes()).unwrap();
    assert_eq!("SOMETHING\nFOO", rslt.sql_in);
  }
}
