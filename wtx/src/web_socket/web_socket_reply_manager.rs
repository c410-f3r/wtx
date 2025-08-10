use crate::{
  collection::ArrayVectorU8,
  sync::{AtomicCell, AtomicWaker},
  web_socket::{FrameControlArray, MAX_CONTROL_PAYLOAD_LEN, OpCode},
};
use core::{future::poll_fn, task::Poll};

/// This structure is necessary to comply with the rules stated by the RFC. See
/// [`Self::reply_frame`].
#[derive(Debug)]
pub struct WebSocketReplyManager<const IS_CLIENT: bool> {
  data: AtomicCell<(bool, Option<(OpCode, [u8; MAX_CONTROL_PAYLOAD_LEN], u8)>)>,
  waker: AtomicWaker,
}

impl<const IS_CLIENT: bool> WebSocketReplyManager<IS_CLIENT> {
  pub(crate) const fn new() -> Self {
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
      let rslt = self.data.fetch_update(|mut el| Some((el.0, el.1.take())));
      let (is_conn_closed, frame) = match rslt {
        Ok(elem) => elem,
        // The closure always return `Some`, as such, this branch is impossible.
        Err(_elem) => {
          self.waker.register(cx.waker());
          return Poll::Pending;
        }
      };
      if let Some((op_code, data, len)) = frame {
        self.data.store((is_conn_closed, None));
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

  pub(crate) fn data(
    &self,
  ) -> &AtomicCell<(bool, Option<(OpCode, [u8; MAX_CONTROL_PAYLOAD_LEN], u8)>)> {
    &self.data
  }

  pub(crate) fn waker(&self) -> &AtomicWaker {
    &self.waker
  }
}
