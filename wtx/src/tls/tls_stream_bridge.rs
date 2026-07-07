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
  inner: Arc<(AtomicCell<Option<TlsStreamBridgeData>>, AtomicWaker)>,
}

impl<const IS_CLIENT: bool> TlsStreamBridge<IS_CLIENT> {
  pub(crate) fn new() -> Self {
    Self { inner: Arc::new((AtomicCell::new(None), AtomicWaker::new())) }
  }

  /// Awaits special records sent by the concurrent reader part. It should probably be called
  /// within a loop.
  ///
  /// The future returned by this method is cancel-safe in the sense that it does not owns
  /// temporary internal data.
  #[inline]
  pub async fn listen(&self) -> TlsStreamBridgeData {
    poll_fn(|cx| {
      self.inner.1.register(cx.waker());
      if let Some(elem) = self.inner.0.update(|_curr| None) {
        Poll::Ready(elem)
      } else {
        Poll::Pending
      }
    })
    .await
  }

  pub(crate) fn update(&self, data: TlsStreamBridgeData) {
    let _ = self.inner.0.update(|_curr| Some(data));
    self.inner.1.wake();
  }
}

/// Data returned by the [`TlsStreamBridge::listen`] method. Should be handed to the writer part.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TlsStreamBridgeData {
  frame: Either<[u8; 2], [u8; 1]>,
}

impl TlsStreamBridgeData {
  #[inline]
  pub(crate) const fn new(frame: Either<[u8; 2], [u8; 1]>) -> Self {
    Self { frame }
  }

  #[inline]
  pub(crate) const fn frame(self) -> Either<[u8; 2], [u8; 1]> {
    self.frame
  }
}
