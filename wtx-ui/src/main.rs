#[cfg(feature = "clap")]
mod clap;
mod misc;

#[tokio::main]
async fn main() -> wtx::Result<()> {
  #[cfg(feature = "clap")]
  clap::init().await?;
  Ok(())
}
