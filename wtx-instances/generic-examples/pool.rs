//! Minimal code that shows the creation of a management structure that always yields `123`.

extern crate tokio;
extern crate wtx;

use wtx::pool::{ResourceManager, SimplePoolTokio};

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
async fn main() -> wtx::Result<()> {
  let pool = SimplePoolTokio::new(1, CustomManager);
  let resource = ***pool.get().await?;
  assert_eq!(resource, 123);
  Ok(())
}
