macro_rules! lock_pin {
  ($cx:expr, $hd:expr, $lock_pin:expr) => {{
    let lock = core::task::ready!($lock_pin.as_mut().poll($cx));
    $lock_pin.set($hd.lock());
    lock
  }};
}

macro_rules! send_go_away_method {
  () => {
    /// Sends a GOAWAY frame to the peer, which cancels the connection and consequently all ongoing
    /// streams.
    #[inline]
    pub async fn send_go_away(&self, error_code: crate::http2::Http2ErrorCode) {
      crate::http2::misc::send_go_away(error_code, &mut self.hd.lock().await.parts_mut()).await;
    }
  };
}

macro_rules! initial_window_len {
  () => {
    65_535
  };
}
macro_rules! max_body_len {
  () => {
    131_070
  };
}
macro_rules! max_hpack_len {
  () => {
    4_096
  };
}
macro_rules! max_concurrent_streams_num {
  () => {
    32
  };
}
macro_rules! max_headers_len {
  () => {
    4_096
  };
}
macro_rules! max_frame_len {
  () => {
    16_384
  };
}
macro_rules! max_frame_len_lower_bound {
  () => {
    16_384
  };
}
macro_rules! max_frame_len_upper_bound {
  () => {
    16_777_215
  };
}
macro_rules! max_recv_streams_num {
  () => {
    32
  };
}
macro_rules! read_buffer_len {
  () => {
    131_070
  };
}
