use crate::{
  database::{executor::Executor, schema_manager::Commands},
  de::DEController,
  misc::Lease,
};
use alloc::string::String;
#[cfg(feature = "std")]
use std::{fs::read_to_string, path::Path};

impl<E> Commands<E>
where
  E: Executor,
{
  /// Executes an arbitrary stream of SQL commands
  ///
  /// It is up to be caller to actually seed the database with data.
  #[inline]
  pub async fn seed<I, S>(
    &mut self,
    buffer_cmd: &mut String,
    seeds: I,
  ) -> Result<(), <E::Database as DEController>::Error>
  where
    I: Iterator<Item = S>,
    S: Lease<str>,
  {
    for elem in seeds {
      buffer_cmd.push_str(elem.lease());
    }
    self
      .executor
      .transaction(|this| async {
        this.execute_many(&mut (), buffer_cmd.as_str(), |_| Ok(())).await?;
        Ok(((), this))
      })
      .await?;
    buffer_cmd.clear();
    Ok(())
  }

  /// Applies `Commands::seed` from a set of files located inside a given `dir`.
  #[cfg(feature = "std")]
  #[inline]
  pub async fn seed_from_dir(
    &mut self,
    buffer_cmd: &mut String,
    dir: &Path,
  ) -> Result<(), <E::Database as DEController>::Error> {
    let iter = crate::database::schema_manager::misc::files(dir)?.filter_map(|el_rslt| {
      let el = el_rslt.ok()?;
      read_to_string(el.path()).ok()
    });
    self.seed(buffer_cmd, iter).await
  }
}
