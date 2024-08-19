use crate::misc::UriString;

pub(crate) fn _uri() -> UriString {
  use core::sync::atomic::{AtomicU32, Ordering};
  static PORT: AtomicU32 = AtomicU32::new(7000);
  let uri = alloc::format!("http://127.0.0.1:{}", PORT.fetch_add(1, Ordering::Relaxed));
  UriString::new(uri)
}
