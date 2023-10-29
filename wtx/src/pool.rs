/// Implements external pool traits.
///
/// This structure only exists because tuples are not fundamental.
#[derive(Debug)]
pub struct PoolElem<T>(
  /// Element
  pub T,
);

#[cfg(feature = "deadpool")]
mod deadpool {
  use crate::{web_socket::FrameBufferVec, PartitionedBuffer, PoolElem};
  use deadpool::managed::{Metrics, RecycleResult};

  #[async_trait::async_trait]
  impl deadpool::managed::Manager for PoolElem<(FrameBufferVec, PartitionedBuffer)> {
    type Error = crate::Error;
    type Type = (FrameBufferVec, PartitionedBuffer);

    async fn create(&self) -> Result<Self::Type, Self::Error> {
      Ok(<_>::default())
    }

    async fn recycle(&self, _: &mut Self::Type, _: &Metrics) -> RecycleResult<Self::Error> {
      Ok(())
    }
  }
}
