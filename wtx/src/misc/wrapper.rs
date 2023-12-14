/// This structure only exists because of the current coherence rules.
#[derive(Debug)]
pub struct Wrapper<T>(
  /// Element
  pub T,
);

#[cfg(all(feature = "deadpool", feature = "web-socket"))]
mod deadpool_web_socket {
  use crate::{
    misc::{PartitionedFilledBuffer, Wrapper},
    web_socket::FrameBufferVec,
  };
  use alloc::boxed::Box;
  use deadpool::managed::{Manager, Metrics, RecycleResult};

  #[async_trait::async_trait]
  impl Manager for Wrapper<(FrameBufferVec, PartitionedFilledBuffer)> {
    type Error = crate::Error;
    type Type = (FrameBufferVec, PartitionedFilledBuffer);

    async fn create(&self) -> Result<Self::Type, Self::Error> {
      Ok(<_>::default())
    }

    async fn recycle(&self, _: &mut Self::Type, _: &Metrics) -> RecycleResult<Self::Error> {
      Ok(())
    }
  }
}
