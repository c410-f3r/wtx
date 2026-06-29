//! WTX - Cli

#![allow(
  clippy::panic,
  clippy::print_stderr,
  clippy::print_stdout,
  clippy::use_debug,
  reason = "CLI application"
)]

#[cfg(feature = "schema-manager")]
extern crate alloc;

#[cfg(feature = "clap")]
mod clap;
#[cfg(feature = "embed-migrations")]
mod embed_migrations;
#[cfg(feature = "http-client")]
mod http_client;
#[cfg(feature = "schema-manager")]
mod schema_manager;
#[cfg(feature = "web-socket")]
mod web_socket;

#[tokio::main]
async fn main() -> wtx::Result<()> {
  #[cfg(feature = "clap")]
  clap::init().await?;
  Ok(())
}
