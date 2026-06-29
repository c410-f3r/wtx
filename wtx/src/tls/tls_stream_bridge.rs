use crate::{
  misc::Either,
  sync::{Arc, AtomicCell, AtomicWaker},
};
use core::{future::poll_fn, task::Poll};

/// The RFC requires all parties (Client or Server) to send back some types of TLS records.
///
/// `WTX` automatically enforces such behavior in sequential code but how is the reader part
/// going to access the writer part in concurrent scenarios? In fact, there are numerous ways
/// to approach this and the choice is yours to make.
///
/// You can see this structure as a bridge between the reader and the writer. Examples about
/// possible utilizations are available in the `wtx-examples` directory.
#[derive(Clone, Debug)]
pub struct TlsStreamBridge<const IS_CLIENT: bool> {
  inner: Arc<(AtomicCell<(bool, Option<TlsStreamBridgeData>)>, AtomicWaker)>,
}

impl<const IS_CLIENT: bool> TlsStreamBridge<IS_CLIENT> {
  pub(crate) fn new() -> Self {
    Self { inner: Arc::new((AtomicCell::new((false, None)), AtomicWaker::new())) }
  }

  /// Awaits special records sent by the concurrent reader part. It should probably be called
  /// within a loop.
  ///
  /// Returns `None` when the reader part is dropped.
  ///
  /// The future returned by this method is cancel-safe in the sense that it does not owns
  /// temporary internal data.
  #[inline]
  pub async fn listen(&self) -> Option<TlsStreamBridgeData> {
    poll_fn(|cx| {
      self.inner.1.register(cx.waker());
      let (is_conn_closed, data) = self.inner.0.update(|el| (el.0, None));
      if let Some(elem) = data {
        Poll::Ready(Some(elem))
      } else if is_conn_closed {
        Poll::Ready(None)
      } else {
        Poll::Pending
      }
    })
    .await
  }

  pub(crate) fn data(&self) -> &AtomicCell<(bool, Option<TlsStreamBridgeData>)> {
    &self.inner.0
  }

  pub(crate) fn waker(&self) -> &AtomicWaker {
    &self.inner.1
  }
}

/// Data returned by the [`TlsStreamBridge::listen`] method. Should be handed to the writer part.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TlsStreamBridgeData {
  pub(crate) inner: Either<[u8; 2], [u8; 1]>,
}
