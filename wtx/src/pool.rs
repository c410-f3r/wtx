/// Implements external pool traits.
///
/// This structure only exists because tuples are not fundamental.
pub struct PoolElem<T>(
  /// Element
  pub T,
);

#[cfg(feature = "deadpool")]
mod deadpool {
  use crate::{web_socket::FrameBufferVec, PartitionedBuffer, PoolElem};

  #[async_trait::async_trait]
  impl deadpool::managed::Manager for PoolElem<(FrameBufferVec, PartitionedBuffer)> {
    type Error = crate::Error;
    type Type = (FrameBufferVec, PartitionedBuffer);

    async fn create(&self) -> Result<Self::Type, Self::Error> {
      Ok(<_>::default())
    }

    async fn recycle(
      &self,
      _: &mut Self::Type,
      _: &deadpool::managed::Metrics,
    ) -> deadpool::managed::RecycleResult<Self::Error> {
      Ok(())
    }
  }
}
