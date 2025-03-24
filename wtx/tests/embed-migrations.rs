//! Embed migrations

#![cfg(feature = "schema-manager")]

#[allow(dead_code)]
mod embedded_migrations;

use wtx::{database::schema_manager::Commands, misc::Vector};

/// Compiles
pub async fn compiles() {
  let mut commands = Commands::with_executor(());
  commands
    .migrate_from_groups(
      (&mut String::new(), &mut Vector::new(), &mut Vector::new()),
      embedded_migrations::GROUPS,
    )
    .await
    .unwrap();
}
