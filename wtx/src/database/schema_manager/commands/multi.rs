use crate::{
  database::schema_manager::{
    Commands, DEFAULT_CFG_FILE_NAME, SchemaManagement, SchemaManagerError, misc::parse_root_toml,
  },
  de::DEController,
  misc::find_file,
};
use std::{env::current_dir, path::Path};

impl<E> Commands<E>
where
  E: SchemaManagement,
{
  /// Useful for DB tests.
  #[inline]
  pub async fn clear_migrate_and_seed(
    &mut self,
  ) -> Result<(), <E::Database as DEController>::Error> {
    let mut buffer = current_dir().map_err(crate::Error::from)?;
    find_file(&mut buffer, Path::new(DEFAULT_CFG_FILE_NAME)).map_err(crate::Error::from)?;
    let (migration_groups, seeds) = parse_root_toml(&buffer)?;
    self.clear().await?;
    self.migrate_from_groups_paths(&migration_groups).await?;
    self
      .seed_from_dir(seeds.as_deref().ok_or_else(|| SchemaManagerError::NoSeedDir.into())?)
      .await?;
    Ok(())
  }
}
