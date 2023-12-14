#![cfg(feature = "sm")]

#[allow(dead_code)]
mod embedded_migrations;

use wtx::database::sm::Commands;

async fn _compiles() {
  let mut commands = Commands::with_executor(());
  commands
    .migrate_from_groups((&mut String::new(), &mut Vec::new()), embedded_migrations::GROUPS)
    .await
    .unwrap();
}
