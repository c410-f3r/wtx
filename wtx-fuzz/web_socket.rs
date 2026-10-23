//! WebSocket frame construction and validation

#![no_main]

use wtx::web_socket::{Frame, OpCode};

libfuzzer_sys::fuzz_target!(|data: &[u8]| {
  for op_code in OpCode::all() {
    let _rslt0 = Frame::new_fin(op_code, data);
    let _rslt1 = Frame::new_unfin(op_code, data);
  }
});
