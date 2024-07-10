use clap::Parser;

pub(crate) async fn init() -> wtx::Result<()> {
  let _args = Cli::parse();
  match _args.commands {
    Commands::_Nothing => {}
    #[cfg(feature = "embed-migrations")]
    Commands::EmbedMigrations(elem) => {
      crate::embed_migrations::embed_migrations(&elem.input, &elem.output).await?;
    }
    #[cfg(feature = "http-client")]
    Commands::HttpClient(elem) => {
      crate::http_client::http_client(elem.uri).await?;
    }
    #[cfg(feature = "schema-manager")]
    Commands::SchemaManager(schema_manager) => {
      crate::schema_manager::schema_manager(&schema_manager).await?;
    }
    #[cfg(feature = "web-socket")]
    Commands::Ws(elem) => match (elem.connect, elem.serve) {
      (None, None) | (Some(_), Some(_)) => {
        panic!("Please connect to a server using `-c` or listen to requests using `-s`");
      }
      (None, Some(uri)) => {
        crate::web_socket::_serve(
          &uri,
          |payload| println!("{payload:?}"),
          |err| println!("{err}"),
          |payload| println!("{payload}"),
        )
        .await?;
      }
      (Some(uri), None) => {
        crate::web_socket::_connect(&uri, |payload| println!("{payload}")).await?;
      }
    },
  }
  Ok(())
}

/// Command-line interface for different web transport implementations
#[derive(Debug, clap::Parser)]
#[command(author, long_about = None, name = "wtx", version)]
struct Cli {
  #[command(subcommand)]
  commands: Commands,
}

#[derive(Debug, clap::Subcommand)]
enum Commands {
  #[clap(skip)]
  _Nothing,
  #[cfg(feature = "embed-migrations")]
  EmbedMigrations(EmbedMigrations),
  #[cfg(feature = "http-client")]
  HttpClient(HttpClient),
  #[cfg(feature = "schema-manager")]
  SchemaManager(SchemaManager),
  #[cfg(feature = "web-socket")]
  Ws(Ws),
}

/// Embed migrations
#[cfg(feature = "embed-migrations")]
#[derive(Debug, clap::Args)]
struct EmbedMigrations {
  /// Configuration file path
  #[arg(default_value_t = wtx::database::schema_manager::DEFAULT_CFG_FILE_NAME.into(), short = 'i', value_name = "Path")]
  input: String,
  /// Rust file path
  #[arg(default_value = "embedded_migrations.rs", short = 'o', value_name = "Path")]
  output: String,
}

/// Http client
#[cfg(feature = "http-client")]
#[derive(Debug, clap::Args)]
struct HttpClient {
  /// URI
  #[arg()]
  uri: String,
}

/// Schema Manager
#[cfg(feature = "schema-manager")]
#[derive(Debug, clap::Args)]
pub(crate) struct SchemaManager {
  /// Configuration file path. If not specified, defaults to "wtx.toml" in the current directory.
  #[arg(short = 'c')]
  pub(crate) toml: Option<std::path::PathBuf>,

  #[command(subcommand)]
  pub(crate) commands: SchemaManagerCommands,

  /// Number of files (migrations or seeds) that is going to be sent to the database in a
  /// single transaction.
  #[arg(default_value_t = wtx::database::schema_manager::DEFAULT_BATCH_SIZE, short = 'f')]
  pub(crate) files_num: usize,

  /// Seeds directory. If not specified, defaults to the optional directory specified in the
  /// configuration file.
  /// Returns an error if none of the options are available.
  #[cfg(feature = "schema-manager-dev")]
  #[arg(short = 's')]
  pub(crate) seeds: Option<std::path::PathBuf>,

  /// Environment variable name that contains the database URI.
  #[arg(default_value_t = wtx::database::DEFAULT_URI_VAR.into(), short = 'v')]
  pub(crate) var: String,
}

#[derive(Debug, clap::Subcommand)]
pub(crate) enum SchemaManagerCommands {
  /// Clean all database objects. For example, tables, triggers or procedures
  #[cfg(feature = "schema-manager-dev")]
  Clean {},
  /// Process local migrations that aren't in the database
  Migrate {},
  /// Shortcut.
  #[cfg(feature = "schema-manager-dev")]
  MigrateAndSeed {},
  /// Returns database state to a point
  Rollback { versions: Vec<i32> },
  /// Populates the database with data intended for testing
  #[cfg(feature = "schema-manager-dev")]
  Seed {},
  /// Checks if the database state is in sync with the local data
  Validate {},
}

/// WebSocket
#[cfg(feature = "web-socket")]
#[derive(Debug, clap::Args)]
struct Ws {
  /// Connects to a server
  #[arg(short = 'c', value_name = "URI")]
  connect: Option<String>,
  /// Listens external requests
  #[arg(short = 's', value_name = "URI")]
  serve: Option<String>,
}
