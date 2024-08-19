//! Minimal code that shows the creation of a management structure that always yields `123`.

extern crate tokio;
extern crate wtx;

use std::sync::LazyLock;
use wtx::pool::{Pool, ResourceManager, SimplePool, SimplePoolTokio};

static POOL: LazyLock<SimplePoolTokio<CustomManager>> =
  LazyLock::new(|| SimplePool::new(1, CustomManager));

pub struct CustomManager;

impl ResourceManager for CustomManager {
  type CreateAux = ();
  type Error = wtx::Error;
  type RecycleAux = ();
  type Resource = i32;

  async fn create(&self, _: &Self::CreateAux) -> Result<Self::Resource, Self::Error> {
    Ok(123)
  }

  async fn is_invalid(&self, _: &Self::Resource) -> bool {
    false
  }

  async fn recycle(&self, _: &Self::RecycleAux, _: &mut Self::Resource) -> Result<(), Self::Error> {
    Ok(())
  }
}

#[tokio::main]
async fn main() {
  let resource = ***POOL.get(&(), &()).await.unwrap();
  assert_eq!(resource, 123);
}
