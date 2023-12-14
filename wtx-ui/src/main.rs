//! WTX - Cli

#[cfg(feature = "clap")]
mod clap;
#[cfg(feature = "embed-migrations")]
mod embed_migrations;
#[cfg(feature = "sm")]
mod sm;
#[cfg(feature = "web-socket")]
mod web_socket;

#[tokio::main]
async fn main() -> wtx::Result<()> {
  #[cfg(feature = "clap")]
  clap::init().await?;
  Ok(())
}
