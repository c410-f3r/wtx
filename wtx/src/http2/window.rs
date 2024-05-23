use crate::{
  http2::{
    http2_params_send::Http2ParamsSend, misc::write_array, Http2Error, Http2Params,
    WindowUpdateFrame, U31,
  },
  misc::Stream,
};

/// A "credit" system used to restrain the exchange of data.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Window {
  applied: i32,
  total: i32,
}

impl Window {
  #[inline]
  pub(crate) const fn new(total: i32) -> Self {
    Self { applied: total, total }
  }

  #[inline]
  pub(crate) fn deposit(&mut self, value: i32) {
    self.applied = self.applied.wrapping_add(value);
  }

  #[inline]
  pub(crate) const fn diff(&self) -> i32 {
    self.total.wrapping_sub(self.applied)
  }

  pub(crate) fn set(&mut self, value: i32) -> crate::Result<()> {
    if value < self.total {
      return Err(crate::Error::http2_go_away_generic(Http2Error::WindowSizeCanNotBeReduced));
    };
    self.total = value;
    Ok(())
  }

  #[inline]
  const fn is_invalid(&self) -> bool {
    self.applied < 0
  }

  #[inline]
  fn withdrawn(&mut self, value: i32) {
    self.applied = self.applied.wrapping_sub(value);
  }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Windows {
  /// Parameters used to received data. It is defined locally.
  pub(crate) recv: Window,
  /// Parameters used to send data. It is initially defined locally with default parameters
  /// and then defined by a remote peer.
  pub(crate) send: Window,
}

impl Windows {
  #[inline]
  pub(crate) const fn new() -> Self {
    Self { recv: Window::new(0), send: Window::new(0) }
  }

  /// Used in initial connections. Sending parameters are only known when a settings frame is received.
  #[inline]
  pub(crate) const fn conn(hp: &Http2Params) -> Self {
    Self {
      recv: Window::new(hp.initial_window_len().i32()),
      send: Window::new(initial_window_len!()),
    }
  }

  /// Used in initial streams.
  #[inline]
  pub(crate) const fn stream(hp: &Http2Params, hps: &Http2ParamsSend) -> Self {
    Self {
      recv: Window::new(hp.initial_window_len().i32()),
      send: Window::new(hps.initial_window_len.i32()),
    }
  }
}

#[derive(Debug)]
pub(crate) struct WindowsPair<'any> {
  pub(crate) conn: &'any mut Windows,
  pub(crate) stream: &'any mut Windows,
}

impl<'any> WindowsPair<'any> {
  pub(crate) fn new(conn: &'any mut Windows, stream: &'any mut Windows) -> Self {
    Self { conn, stream }
  }

  #[inline]
  pub(crate) async fn manage_recv<S>(
    &mut self,
    is_conn_open: bool,
    stream: &mut S,
    stream_id: U31,
    value: U31,
  ) -> crate::Result<()>
  where
    S: Stream,
  {
    self.conn.recv.withdrawn(value.i32());
    self.stream.recv.withdrawn(value.i32());
    match (self.conn.recv.is_invalid(), self.stream.recv.is_invalid()) {
      (false, false) => {}
      (false, true) => {
        let conn_diff = self.conn.recv.diff();
        self.conn.recv.deposit(conn_diff);
        write_array(
          [&WindowUpdateFrame::new(U31::from_i32(conn_diff), U31::ZERO)?.bytes()],
          is_conn_open,
          stream,
        )
        .await?;
      }
      (true, false) => {
        let stream_diff = self.stream.recv.diff();
        self.stream.recv.deposit(stream_diff);
        write_array(
          [&WindowUpdateFrame::new(U31::from_i32(stream_diff), stream_id)?.bytes()],
          is_conn_open,
          stream,
        )
        .await?;
      }
      (true, true) => {
        let conn_diff = self.conn.recv.diff();
        let stream_diff = self.stream.recv.diff();
        self.conn.recv.deposit(conn_diff);
        self.stream.recv.deposit(stream_diff);
        write_array(
          [
            &WindowUpdateFrame::new(U31::from_i32(conn_diff), U31::ZERO)?.bytes(),
            &WindowUpdateFrame::new(U31::from_i32(stream_diff), stream_id)?.bytes(),
          ],
          is_conn_open,
          stream,
        )
        .await?;
      }
    }
    Ok(())
  }

  #[inline]
  pub(crate) fn manage_send(&mut self, value: U31) -> bool {
    self.conn.send.withdrawn(value.i32());
    self.stream.send.withdrawn(value.i32());
    if !self.conn.send.is_invalid() && !self.stream.send.is_invalid() {
      true
    } else {
      self.conn.send.deposit(value.i32());
      self.stream.send.deposit(value.i32());
      false
    }
  }

  // Sending data based on received parameters
  #[inline]
  pub(crate) fn is_invalid_send(&self) -> bool {
    self.conn.send.is_invalid() || self.stream.send.is_invalid()
  }
}
