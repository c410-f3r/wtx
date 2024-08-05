# Pool Manager

An asynchronous pool of arbitrary objects where each element is dynamically created or re-created when invalid.

Can also be used for database connections, which is quite handy because it enhances the performance of executing commands and alleviates the use of hardware resources.

Activation feature is called `pool`.

```rust,edition2021
extern crate wtx;

use wtx::pool::{ResourceManager, SimplePool, SimplePoolResource};
use core::cell::RefCell;

pub struct TestManager;

impl ResourceManager for TestManager {
  type CreateAux = ();
  type Error = wtx::Error;
  type RecycleAux = ();
  type Resource = i32;

  #[inline]
  async fn create(&self, _: &Self::CreateAux) -> Result<Self::Resource, Self::Error> {
    Ok(0)
  }

  async fn is_invalid(&self, _: &Self::Resource) -> bool {
    false
  }

  #[inline]
  async fn recycle(
    &self,
    _: &Self::RecycleAux,
    _: &mut Self::Resource,
  ) -> Result<(), Self::Error> {
    Ok(())
  }
}

pub fn pool() -> SimplePool<RefCell<SimplePoolResource<i32>>, TestManager> {
  SimplePool::new(1, TestManager)
}
```
