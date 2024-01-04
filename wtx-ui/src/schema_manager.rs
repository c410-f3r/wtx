use crate::clap::{SchemaManager, SchemaManagerCommands};
use std::{borrow::Cow, env::current_dir, path::Path};
use wtx::{
  database::{
    schema_manager::{Commands, DbMigration, SchemaManagement, DEFAULT_CFG_FILE_NAME},
    Identifier, DEFAULT_URI_VAR,
  },
  misc::UriRef,
};

pub(crate) async fn schema_manager(sm: &SchemaManager) -> wtx::Result<()> {
  #[cfg(feature = "schema-manager-dev")]
  {
    let err = std::io::ErrorKind::NotFound;
    let _path = dotenv::dotenv().map_err(|_err| wtx::Error::IoError(err.into()))?;
  }
  #[cfg(feature = "schema-manager-dev")]
  wtx::misc::tracing_subscriber_init()?;

  let var = std::env::var(DEFAULT_URI_VAR)?;
  let uri = UriRef::new(&var);
  match uri.schema() {
    "postgres" | "postgresql" => {
      handle_commands((), sm).await?;
    }
    _ => return Err(wtx::Error::InvalidUrl),
  }
  Ok(())
}

fn toml_file_path(sm: &SchemaManager) -> wtx::Result<Cow<'_, Path>> {
  Ok(if let Some(el) = sm.toml.as_deref() {
    Cow::Borrowed(el)
  } else {
    let mut path_buf = current_dir()?;
    path_buf.push(DEFAULT_CFG_FILE_NAME);
    Cow::Owned(path_buf)
  })
}

#[inline]
async fn handle_commands<E>(executor: E, sm: &SchemaManager) -> wtx::Result<()>
where
  E: SchemaManagement,
{
  let _buffer_cmd = &mut String::new();
  let _buffer_db_migrations = &mut Vec::<DbMigration>::new();
  let _buffer_idents = &mut Vec::<Identifier>::new();

  let mut commands = Commands::new(sm.files_num, executor);
  match &sm.commands {
    #[cfg(feature = "schema-manager-dev")]
    SchemaManagerCommands::Clean {} => {
      commands.clear((_buffer_cmd, _buffer_idents)).await?;
    }
    SchemaManagerCommands::Migrate {} => {
      commands
        .migrate_from_toml_path((_buffer_cmd, _buffer_db_migrations), &toml_file_path(sm)?)
        .await?;
    }
    #[cfg(feature = "schema-manager-dev")]
    SchemaManagerCommands::MigrateAndSeed {} => {
      let (migration_groups, seeds) =
        wtx::database::schema_manager::misc::parse_root_toml(&toml_file_path(sm)?)?;
      commands
        .migrate_from_groups_paths((_buffer_cmd, _buffer_db_migrations), &migration_groups)
        .await?;
      commands.seed_from_dir(_buffer_cmd, seeds_file_path(sm, seeds.as_deref())?).await?;
    }
    SchemaManagerCommands::Rollback { versions: _versions } => {
      commands
        .rollback_from_toml((_buffer_cmd, _buffer_db_migrations), &toml_file_path(sm)?, &_versions)
        .await?;
    }
    #[cfg(feature = "schema-manager-dev")]
    SchemaManagerCommands::Seed {} => {
      let (_, seeds) = wtx::database::schema_manager::misc::parse_root_toml(&toml_file_path(sm)?)?;
      commands.seed_from_dir(_buffer_cmd, seeds_file_path(sm, seeds.as_deref())?).await?;
    }
    SchemaManagerCommands::Validate {} => {
      commands
        .validate_from_toml((_buffer_cmd, _buffer_db_migrations), &toml_file_path(sm)?)
        .await?;
    }
  }
  Ok(())
}

#[cfg(feature = "schema-manager-dev")]
fn seeds_file_path<'a, 'b, 'c>(
  sm: &'a SchemaManager,
  seeds_toml: Option<&'b Path>,
) -> wtx::Result<&'c Path>
where
  'a: 'c,
  'b: 'c,
{
  if let Some(el) = sm.seeds.as_deref() {
    return Ok(el);
  }
  if let Some(el) = seeds_toml {
    return Ok(el);
  }
  panic!("The `seeds` parameter must be provided through the CLI or the configuration file");
}
