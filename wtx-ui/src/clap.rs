use clap::Parser;

pub(crate) async fn init() -> wtx::Result<()> {
  let _args = Cli::parse();

  #[cfg(feature = "unified")]
  match _args.commands {
    Commands::_Nothing => {}
    #[cfg(feature = "embed-migrations")]
    Commands::EmbedMigrations(elem) => {
      crate::embed_migrations::embed_migrations(elem).await?;
    }
    #[cfg(feature = "http-client")]
    Commands::HttpClient(elem) => {
      crate::http_client::http_client(elem).await;
    }
    #[cfg(feature = "schema-manager")]
    Commands::SchemaManager(schema_manager) => {
      crate::schema_manager::schema_manager(schema_manager).await?;
    }
    #[cfg(feature = "web-socket")]
    Commands::WebSocket(elem) => manage_web_socket(elem).await,
  }

  #[cfg(not(feature = "unified"))]
  {
    #[cfg(all(
      feature = "embed-migrations",
      not(any(feature = "http-client", feature = "schema-manager", feature = "web-socket"))
    ))]
    crate::embed_migrations::embed_migrations(_args.commands).await?;

    #[cfg(all(
      feature = "http-client",
      not(any(feature = "embed-migrations", feature = "schema-manager", feature = "web-socket"))
    ))]
    crate::http_client::http_client(_args.commands).await;

    #[cfg(all(
      feature = "schema-manager",
      not(any(feature = "embed-migrations", feature = "http-client", feature = "web-socket"))
    ))]
    crate::schema_manager::schema_manager(_args.commands).await?;

    #[cfg(all(
      feature = "web-socket",
      not(any(feature = "embed-migrations", feature = "http-client", feature = "schema-manager"))
    ))]
    manage_web_socket(_args.commands).await;
  }

  Ok(())
}

/// Command-line interface for different web transport implementations
#[derive(Debug, clap::Parser)]
#[cfg(feature = "unified")]
#[command(author, long_about = None, name = "wtx", version)]
struct Cli {
  #[cfg(feature = "unified")]
  #[command(subcommand)]
  commands: Commands,
}

/// Command-line interface for different web transport implementations
#[derive(Debug, clap::Parser)]
#[cfg(not(feature = "unified"))]
#[command(author, long_about = None, name = "wtx", version)]
struct Cli {
  #[cfg(all(
    feature = "embed-migrations",
    not(any(feature = "http-client", feature = "schema-manager", feature = "web-socket"))
  ))]
  #[clap(flatten)]
  commands: EmbedMigrations,

  #[cfg(all(
    feature = "http-client",
    not(any(feature = "embed-migrations", feature = "schema-manager", feature = "web-socket"))
  ))]
  #[clap(flatten)]
  commands: HttpClient,

  #[cfg(all(
    feature = "schema-manager",
    not(any(feature = "embed-migrations", feature = "http-client", feature = "web-socket"))
  ))]
  #[clap(flatten)]
  commands: SchemaManager,

  #[cfg(all(
    feature = "web-socket",
    not(any(feature = "embed-migrations", feature = "http-client", feature = "schema-manager"))
  ))]
  #[clap(flatten)]
  commands: WebSocket,
}

#[cfg(feature = "unified")]
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
  WebSocket(WebSocket),
}

/// Embed migrations
#[cfg(feature = "embed-migrations")]
#[derive(Debug, clap::Args)]
pub(crate) struct EmbedMigrations {
  /// Configuration file path
  #[arg(default_value_t = wtx::database::schema_manager::DEFAULT_CFG_FILE_NAME.into(), short = 'i', value_name = "Path")]
  pub(crate) input: String,
  /// Rust file path
  #[arg(default_value = "embedded_migrations.rs", short = 'o', value_name = "Path")]
  pub(crate) output: String,
}

/// Http client
#[cfg(feature = "http-client")]
#[derive(Debug, clap::Args)]
pub(crate) struct HttpClient {
  /// HTTP POST data
  #[arg(long, short)]
  pub(crate) data: Option<String>,

  /// Pass custom header(s) to server
  #[arg(long, num_args = 1.., short = 'H', value_name = "HEADER")]
  pub(crate) header: Vec<String>,

  /// Specify request command to use
  #[arg(default_value = "GET", long, short = 'X')]
  pub(crate) method: wtx::http::Method,

  /// Write to file instead of stdout
  #[arg(long, short)]
  pub(crate) output: Option<String>,

  /// The URI to request
  pub(crate) uri: String,

  /// Specify a custom User-Agent
  #[arg(long = "user-agent", short = 'A')]
  pub(crate) user_agent: Option<String>,

  /// Verbose mode
  #[arg(action = clap::ArgAction::Count, help = "Output verbosity (-v, -vv or -vvv)", long, short)]
  pub(crate) verbose: u8,
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
struct WebSocket {
  /// Connects to a server
  #[arg(short = 'c', value_name = "URI")]
  connect: Option<String>,
  /// Listens external requests
  #[arg(short = 's', value_name = "URI")]
  serve: Option<String>,
}

#[cfg(feature = "web-socket")]
async fn manage_web_socket(elem: WebSocket) {
  match (elem.connect, elem.serve) {
    (None, None) | (Some(_), Some(_)) => {
      panic!("Please connect to a server using `-c` or listen to requests using `-s`");
    }
    (None, Some(uri)) => {
      crate::web_socket::serve(
        &uri,
        |payload| println!("{payload:?}"),
        |err| println!("{err}"),
        |payload| println!("{payload}"),
      )
      .await
      .unwrap();
    }
    (Some(uri), None) => {
      crate::web_socket::connect(&uri, |payload| println!("{payload}")).await.unwrap();
    }
  }
}
