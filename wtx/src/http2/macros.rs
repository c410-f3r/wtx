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
      crate::http2::misc::send_go_away(error_code, &self.inner).await;
    }
  };
}
