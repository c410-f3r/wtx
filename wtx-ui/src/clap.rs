use crate::misc::{_connect, _serve};
use clap::Parser;

pub(crate) async fn init() -> wtx::Result<()> {
  let args = Cli::parse();
  match args.commands {
    Commands::Ws(ws_args) => match (ws_args.connect, ws_args.serve) {
      (None, None) | (Some(_), Some(_)) => {
        panic!("Please connect to a server using `-c` or listen to requests using `-s`");
      }
      (None, Some(uri)) => {
        _serve(
          &uri,
          |payload| println!("{payload:?}"),
          |err| println!("{err}"),
          |payload| println!("{payload}"),
        )
        .await?;
      }
      (Some(uri), None) => {
        _connect(&uri, |payload| println!("{payload}")).await?;
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
  Ws(WsArgs),
}

/// WebSocket subcommands
#[derive(Debug, clap::Args)]
#[command(args_conflicts_with_subcommands = true)]
struct WsArgs {
  /// Connects to a server
  #[arg(short = 'c', value_name = "URL")]
  connect: Option<String>,
  /// Listens external requests
  #[arg(short = 's', value_name = "URL")]
  serve: Option<String>,
}
