# Pool Manager

An asynchronous pool of arbitrary objects where each element is dynamically created or re-created when invalid.

Can also be used for database connections, which is quite handy because it enhances the performance of executing commands and alleviates the use of hardware resources.

Activation feature is called `pool`.

```ignore,rust,edition2021
use wtx::pool_manager::{ResourceManager, StaticPool};
use core::cell::RefCell;

struct TestManager;

impl ResourceManager for TestManager {
  type Persistent = ();
  type Error = crate::Error;
  type Resource = i32;

  #[inline]
  async fn create(&self) -> Result<Self::Resource, Self::Error> {
    Ok(0)
  }

  #[inline]
  fn check_integrity(&self, _: &mut Self::Resource) -> Option<Self::Persistent> {
    None
  }

  #[inline]
  async fn recycle(
    &self,
    _: Self::Persistent,
    _: &mut Self::Resource,
  ) -> Result<(), Self::Error> {
    Ok(())
  }
}

fn pool() -> StaticPool<RefCell<Option<i32>>, TestManager, 2> {
  StaticPool::new(TestManager).unwrap()
}
```
