#![cfg(feature = "schema-manager")]

#[allow(dead_code)]
mod embedded_migrations;

use wtx::database::schema_manager::Commands;

pub async fn compiles() {
  let mut commands = Commands::with_executor(());
  commands
    .migrate_from_groups((&mut String::new(), &mut Vec::new()), embedded_migrations::GROUPS)
    .await
    .unwrap();
}
