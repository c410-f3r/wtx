use crate::{
  database::schema_manager::{
    Commands, DEFAULT_CFG_FILE_NAME, SchemaManagement, misc::parse_root_toml,
  },
  de::DEController,
  misc::find_file,
};
use std::{
  env::current_dir,
  path::{Path, PathBuf},
};

impl<E> Commands<E>
where
  E: SchemaManagement,
{
  /// Useful for DB tests.
  #[inline]
  pub async fn clear_migrate_and_seed(
    &mut self,
    dir: Option<&str>,
  ) -> Result<(), <E::Database as DEController>::Error> {
    let mut buffer = if let Some(elem) = dir {
      PathBuf::from(elem)
    } else {
      current_dir().map_err(crate::Error::from)?
    };
    find_file(&mut buffer, Path::new(DEFAULT_CFG_FILE_NAME)).map_err(crate::Error::from)?;
    let (migration_groups, seeds) = parse_root_toml(&buffer)?;
    self.clear().await?;
    self.migrate_from_groups_paths(&migration_groups).await?;
    if let Some(elem) = seeds {
      self.seed_from_dir(&elem).await?;
    }
    Ok(())
  }
}
