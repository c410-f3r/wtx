use crate::{
  database::schema_manager::{
    Commands, SchemaManagement, SchemaManagerError, misc::parse_root_toml,
  },
  de::DEController,
};
use std::env::current_dir;

impl<E> Commands<E>
where
  E: SchemaManagement,
{
  /// Useful for DB tests.
  #[inline]
  pub async fn clear_migrate_and_seed(
    &mut self,
  ) -> Result<(), <E::Database as DEController>::Error> {
    let (migration_groups, seeds) = parse_root_toml(&current_dir().map_err(crate::Error::from)?)?;
    self.clear().await?;
    self.migrate_from_groups_paths(&migration_groups).await?;
    self
      .seed_from_dir(seeds.as_deref().ok_or_else(|| SchemaManagerError::NoSeedDir.into())?)
      .await?;
    Ok(())
  }
}
