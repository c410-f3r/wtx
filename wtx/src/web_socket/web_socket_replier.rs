use crate::{
  collection::ArrayVectorU8,
  sync::{AtomicCell, AtomicWaker},
  web_socket::{FrameControlArray, MAX_CONTROL_PAYLOAD_LEN, OpCode},
};
use core::{future::poll_fn, task::Poll};

/// The RFC-6455 requires all parties (Client or Server) to send back carefully managed `Close`
/// frames read from the stream. Received `Ping` frames must also reply with `Pong` frames.
///
/// `WTX` automatically enforces these rules in sequential code but how are the reader
/// part going to access the writer part in concurrent scenarios? In fact, there are numerous ways
/// to approach this and the choice is yours to make.
///
/// You can see this structure as a bridge between the reader and the writer. Examples about
/// possible utilizations are available at the `wtx-instances` directory in the repository.
///
/// #### Noteworthy
///
/// Reply frames sent without the usage of [`WebSocketReplier`] in concurrent scenarios are not
/// RFC-6455 compliant.
#[derive(Debug)]
pub struct WebSocketReplier<const IS_CLIENT: bool> {
  data: AtomicCell<(bool, Option<(OpCode, [u8; MAX_CONTROL_PAYLOAD_LEN], u8)>)>,
  waker: AtomicWaker,
}

impl<const IS_CLIENT: bool> WebSocketReplier<IS_CLIENT> {
  pub(crate) fn new() -> Self {
    Self { data: AtomicCell::new((false, None)), waker: AtomicWaker::new() }
  }

  /// Awaits `Close` or `Pong` frames sent by the concurrent reader part. It should probably be
  /// called within a loop.
  ///
  /// Received `Close` frames should halt further processing. Returns `None` when the reader
  /// part is dropped.
  #[inline]
  pub async fn reply_frame(&self) -> Option<FrameControlArray<IS_CLIENT>> {
    poll_fn(|cx| {
      let (is_conn_closed, frame) = self.data.update(|el| (el.0, None));
      if let Some((op_code, data, len)) = frame {
        Poll::Ready(Some(FrameControlArray::<IS_CLIENT>::new_fin(
          op_code,
          ArrayVectorU8::from_parts(data, Some(len)),
        )))
      } else if is_conn_closed {
        Poll::Ready(None)
      } else {
        self.waker.register(cx.waker());
        Poll::Pending
      }
    })
    .await
  }

  pub(crate) const fn data(
    &self,
  ) -> &AtomicCell<(bool, Option<(OpCode, [u8; MAX_CONTROL_PAYLOAD_LEN], u8)>)> {
    &self.data
  }

  pub(crate) const fn waker(&self) -> &AtomicWaker {
    &self.waker
  }
}
