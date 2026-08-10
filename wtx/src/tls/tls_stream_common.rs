use crate::sync::{AtomicBool, AtomicU8, AtomicWaker};

#[derive(Debug)]
pub(crate) struct TlsStreamCommon {
  pub(crate) can_reply_key_update: AtomicBool,
  pub(crate) connection_state: AtomicU8,
  pub(crate) reader_waker: AtomicWaker,
}
