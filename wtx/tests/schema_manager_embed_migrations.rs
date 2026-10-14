//! Embed migrations

#![cfg(feature = "schema-manager")]

mod schema_manager_embedded_migrations;

use wtx::database::schema_manager::Commands;

/// Compiles
pub async fn compiles() {
  let mut commands = Commands::with_executor(());
  commands.migrate_from_groups(schema_manager_embedded_migrations::GROUPS).await.unwrap();
}
