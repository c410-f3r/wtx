use std::{fmt::Write, path::Path};
use tokio::{fs::OpenOptions, io::AsyncWriteExt};
use wtx::database::sm::misc::{group_and_migrations_from_path, parse_root_toml};

pub(crate) async fn embed_migrations(input: &str, output: &str) -> wtx::Result<()> {
  let (mut migration_groups, _) = parse_root_toml(Path::new(input))?;
  let mut buffer = String::new();

  migration_groups.sort();

  buffer.push_str(
    "#[rustfmt::skip]pub(crate) const GROUPS: wtx::database::sm::EmbeddedMigrationsTy = &[",
  );

  for mg_path in migration_groups {
    let (mg, ms) = group_and_migrations_from_path(&mg_path, Ord::cmp)?;
    let mg_name = mg.name().to_ascii_uppercase();
    let mg_version = mg.version();

    buffer.write_fmt(format_args!(
      concat!(
        "{{",
        r#"const {mg_name}: &wtx::database::sm::MigrationGroup<&'static str> = &wtx::database::sm::MigrationGroup::new("{mg_name}",{mg_version});"#,
        r#"const {mg_name}_MIGRATIONS: &[wtx::database::sm::UserMigrationRef<'static, 'static>] = &["#
      ),
      mg_name = mg_name,
      mg_version = mg_version
    ))?;

    for rslt in ms {
      let migration = rslt?;
      let checksum = migration.checksum();
      let dbs = migration.dbs();
      let name = migration.name();
      let sql_down = migration.sql_down();
      let sql_up = migration.sql_up();
      let version = migration.version();

      buffer.write_fmt(format_args!(
        "wtx::database::sm::UserMigrationRef::from_all_parts({checksum},&["
      ))?;
      for db in dbs {
        buffer.push_str("wtx::database::DatabaseTy::");
        buffer.push_str(db.strings().ident);
        buffer.push(',');
      }
      buffer.write_fmt(format_args!(r#"],"{name}","#))?;
      match migration.repeatability() {
        None => buffer.push_str("None"),
        Some(elem) => buffer.write_fmt(format_args!(
          "Some(wtx::database::sm::Repeatability::{})",
          elem.strings().ident
        ))?,
      }
      buffer.write_fmt(format_args!(r#","{sql_down}","{sql_up}",{version}),"#))?;
    }

    buffer.write_fmt(format_args!("];({mg_name},{mg_name}_MIGRATIONS)}},"))?;
  }

  buffer.push_str("];");

  OpenOptions::new()
    .create(true)
    .truncate(true)
    .write(true)
    .open(output)
    .await?
    .write_all(buffer.as_bytes())
    .await?;

  Ok(())
}
