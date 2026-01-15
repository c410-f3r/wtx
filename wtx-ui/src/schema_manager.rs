use crate::clap::{SchemaManager, SchemaManagerCommands};
use std::{borrow::Cow, env::current_dir, path::Path};
use tokio::net::TcpStream;
use wtx::{
  collection::Vector,
  database::{
    DEFAULT_URI_VAR, Identifier,
    client::postgres::{Config, ExecutorBuffer, PostgresExecutor},
    schema_manager::{
      Commands, DEFAULT_CFG_FILE_NAME, DbMigration, MigrationStatus, SchemaManagement,
    },
  },
  de::DEController,
  misc::{EnvVars, FromVars, UriRef, find_file},
  rng::{ChaCha20, SeedableRng},
};

pub(crate) async fn schema_manager(sm: SchemaManager) -> wtx::Result<()> {
  #[cfg(feature = "schema-manager-dev")]
  let var = {
    wtx::misc::tracing_tree_init(None)?;
    EnvVars::<DefaultUriVar>::from_available()?.finish().0
  };
  #[cfg(not(feature = "schema-manager-dev"))]
  let var = EnvVars::<DefaultUriVar>::from_process()?.finish().0;

  let uri = UriRef::new(&var);
  match uri.scheme() {
    "postgres" | "postgresql" => {
      let mut rng = ChaCha20::from_getrandom()?;
      let executor = PostgresExecutor::<wtx::Error, _, _>::connect(
        &Config::from_uri(&uri)?,
        ExecutorBuffer::new(usize::MAX, &mut rng),
        &mut rng,
        TcpStream::connect(uri.hostname_with_implied_port()).await?,
      )
      .await?;
      handle_commands(executor, &sm).await?;
    }
    _ => return Err(wtx::Error::InvalidUri),
  }
  Ok(())
}

struct DefaultUriVar(String);

impl FromVars for DefaultUriVar {
  fn from_vars(vars: impl IntoIterator<Item = (String, String)>) -> wtx::Result<Self> {
    let mut rslt = None;
    for (key, value) in vars {
      if key == DEFAULT_URI_VAR {
        rslt = Some(value)
      }
    }
    Ok(Self(rslt.ok_or_else(|| wtx::Error::MissingVar(DEFAULT_URI_VAR.into()))?))
  }
}

fn toml_file_path(sm: &SchemaManager) -> wtx::Result<Cow<'_, Path>> {
  Ok(if let Some(el) = sm.toml.as_deref() {
    Cow::Borrowed(el)
  } else {
    let mut buffer = current_dir()?;
    find_file(&mut buffer, Path::new(DEFAULT_CFG_FILE_NAME))?;
    Cow::Owned(buffer)
  })
}

async fn handle_commands<E>(
  executor: E,
  sm: &SchemaManager,
) -> Result<(), <E::Database as DEController>::Error>
where
  E: SchemaManagement,
{
  let _buffer_cmd = &mut String::new();
  let _buffer_db_migrations = &mut Vector::<DbMigration>::new();
  let _buffer_idents = &mut Vector::<Identifier>::new();
  let _buffer_status = &mut Vector::<MigrationStatus>::new();

  let mut commands = Commands::new(sm.files_num, executor);
  match &sm.commands {
    SchemaManagerCommands::CheckFullRollback {} => {
      commands.rollback_from_toml(&toml_file_path(sm)?, None).await?;
      let mut iter = commands.all_elements().await?.into_iter();
      let (Some("_wtx"), None) = (iter.next().as_deref(), iter.next()) else {
        eprintln!("{_buffer_idents:?}");
        let msg = String::from("The rollback operation didn't leave the database in a clean state");
        return Err(wtx::Error::Generic(msg.into()).into());
      };
    }
    #[cfg(feature = "schema-manager-dev")]
    SchemaManagerCommands::Clean {} => {
      commands.clear().await?;
    }
    SchemaManagerCommands::Migrate {} => {
      commands.migrate_from_toml_path(&toml_file_path(sm)?).await?;
    }
    #[cfg(feature = "schema-manager-dev")]
    SchemaManagerCommands::MigrateAndSeed {} => {
      use wtx::database::schema_manager::misc::parse_root_toml;
      let (migration_groups, seeds) = parse_root_toml(&toml_file_path(sm)?)?;
      commands.migrate_from_groups_paths(&migration_groups).await?;
      commands.seed_from_dir(seeds_file_path(sm, seeds.as_deref())?).await?;
    }
    SchemaManagerCommands::Rollback { versions: _versions } => {
      commands.rollback_from_toml(&toml_file_path(sm)?, Some(_versions)).await?;
    }
    #[cfg(feature = "schema-manager-dev")]
    SchemaManagerCommands::Seed {} => {
      use wtx::database::schema_manager::misc::parse_root_toml;
      let (_, seeds) = parse_root_toml(&toml_file_path(sm)?)?;
      commands.seed_from_dir(seeds_file_path(sm, seeds.as_deref())?).await?;
    }
    SchemaManagerCommands::Validate {} => {
      commands.validate_from_toml(&toml_file_path(sm)?).await?;
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
