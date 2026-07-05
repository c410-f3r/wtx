use crate::{
  collections::ArrayVectorCopy,
  sync::{Arc, AtomicCell, AtomicWaker},
  tls::{TlsStreamBridge, TlsStreamBridgeData},
  web_socket::{FrameControlArray, MAX_CONTROL_PAYLOAD_LEN, OpCode},
};
use core::{future::poll_fn, pin::pin, task::Poll};

type WsTy =
  (AtomicCell<Option<(OpCode, ArrayVectorCopy<u8, MAX_CONTROL_PAYLOAD_LEN>)>>, AtomicWaker);

/// The RFC requires all parties (Client or Server) to send back some types of frames.
///
/// `WTX` automatically enforces such behavior in sequential code but how is the reader part
/// going to access the writer part in concurrent scenarios? In fact, there are numerous ways
/// to approach this and the choice is yours to make.
///
/// You can see this structure as a bridge between the reader and the writer. Examples about
/// possible utilizations are available in the `wtx-examples` directory.
///
/// #### Noteworthy
///
/// Reply frames sent without the usage of [`WebSocketBridge`] in concurrent scenarios are not
/// RFC-6455 compliant. Moreover, TLS data is also handled by this structure.
#[derive(Clone, Debug)]
pub struct WebSocketBridge<const IS_CLIENT: bool> {
  tls: TlsStreamBridge<IS_CLIENT>,
  ws: Arc<WsTy>,
}

impl<const IS_CLIENT: bool> WebSocketBridge<IS_CLIENT> {
  pub(crate) fn new(tls: TlsStreamBridge<IS_CLIENT>) -> Self {
    Self { tls, ws: Arc::new((AtomicCell::new(None), AtomicWaker::new())) }
  }

  /// Awaits special frames sent by the concurrent reader part. It should probably be
  /// called within a loop.
  ///
  /// Returns `None` when the reader part is dropped.
  ///
  /// The future returned by this method is cancel-safe in the sense that it does not owns
  /// temporary internal data.
  #[inline]
  pub async fn listen(&self) -> Option<WebSocketBridgeData> {
    let mut tls_fut = pin!(self.tls.listen());
    let mut ws_fut = pin!(self.do_listen());
    poll_fn(|cx| {
      let data = match (tls_fut.as_mut().poll(cx), ws_fut.as_mut().poll(cx)) {
        (Poll::Ready(tls), Poll::Ready(ws)) => {
          if tls.is_none() && ws.is_none() {
            None
          } else {
            Some(WebSocketBridgeData { tls, ws })
          }
        }
        (Poll::Ready(tls), Poll::Pending) => {
          tls.map(|el| WebSocketBridgeData { tls: Some(el), ws: None })
        }
        (Poll::Pending, Poll::Ready(ws)) => {
          ws.map(|el| WebSocketBridgeData { tls: None, ws: Some(el) })
        }
        (Poll::Pending, Poll::Pending) => return Poll::Pending,
      };
      Poll::Ready(data)
    })
    .await
  }

  pub(crate) fn update(&self, data: (OpCode, ArrayVectorCopy<u8, MAX_CONTROL_PAYLOAD_LEN>)) {
    let _ = self.ws.0.update(|_prev| Some(data));
    self.ws.1.wake();
  }

  async fn do_listen(&self) -> Option<FrameControlArray> {
    poll_fn(|cx| {
      self.ws.1.register(cx.waker());
      let frame = self.ws.0.update(|_curr| None);
      if let Some((op_code, payload)) = frame {
        Poll::Ready(Some(FrameControlArray::new(true, op_code, payload, 0)))
      } else {
        Poll::Pending
      }
    })
    .await
  }
}

/// Data returned by the [`WebSocketBridge::listen`] method. Should be handed to the writer part.
#[derive(Debug)]
pub struct WebSocketBridgeData {
  pub(crate) tls: Option<TlsStreamBridgeData>,
  pub(crate) ws: Option<FrameControlArray>,
}
