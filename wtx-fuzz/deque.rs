//! Deque

#![no_main]

use tokio::runtime::Builder;
use wtx::{
  rng::{Xorshift64, simple_seed},
  stream::BytesStream,
  web_socket::{Frame, OpCode, WebSocket, WebSocketBuffer},
};

libfuzzer_sys::fuzz_target!(|data: (u8, u8, u8, Range<usize>)| {
  let (a, b, c, range) = data;
  let mut deque = Deque::with_capacity(32);
  for _ in 0..a.min(2) {
    deque.push_back(1);
  }
  for _ in 0..b.min(2) {
    deque.push_front(2);
  }
  for _ in 0..c.min(2) {
    let _rslt = deque.extend_front_from_copyable_within(range.start.min(36)..range.end.min(36));
  }
});
