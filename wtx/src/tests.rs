use core::borrow::BorrowMut;

use crate::{
  rng::Rng,
  web_socket::{compression::NegotiatedCompression, FrameBufferVec, WebSocketClient},
  PartitionedBuffer, Stream,
};

#[async_trait::async_trait]
pub trait AsyncTrait {
  async fn method(&mut self) -> crate::Result<()>;
}

#[async_trait::async_trait]
impl<NC, PB, RNG, S> AsyncTrait for (&mut FrameBufferVec, &mut WebSocketClient<NC, PB, RNG, S>)
where
  NC: NegotiatedCompression + Send + Sync,
  PB: BorrowMut<PartitionedBuffer> + Send + Sync,
  RNG: Rng + Send + Sync,
  S: Stream + Send + Sync,
  for<'read> S::Read<'read>: Send + Sync,
  for<'write> S::Write<'write>: Send + Sync,
{
  async fn method(&mut self) -> crate::Result<()> {
    let _ = self.1.borrow_mut().read_frame(self.0).await?;
    Ok(())
  }
}
