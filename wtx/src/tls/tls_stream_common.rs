use crate::{
  sync::{AtomicBool, AtomicCell, AtomicU8, AtomicWaker},
  tls::protocol::key_update::KeyUpdate,
};

#[derive(Debug)]
pub(crate) struct TlsStreamCommon {
  pub(crate) can_reply_key_update: AtomicBool,
  pub(crate) connection_state: AtomicU8,
  pub(crate) key_update_reply: AtomicCell<Option<KeyUpdate>>,
  pub(crate) reader_waker: AtomicWaker,
}
