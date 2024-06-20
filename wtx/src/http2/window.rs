use crate::{
  http2::{
    http2_params_send::Http2ParamsSend, misc::write_array, Http2Error, Http2ErrorCode, Http2Params,
    WindowUpdateFrame, U31,
  },
  misc::Stream,
};

/// A "credit" system used to restrain the exchange of data.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Window {
  available: i32,
  original: i32,
}

impl Window {
  #[inline]
  pub(crate) const fn new(available: i32) -> Self {
    Self { available, original: available }
  }

  #[inline]
  pub(crate) const fn available(&self) -> i32 {
    self.available
  }

  #[inline]
  pub(crate) fn deposit(&mut self, stream_id: Option<U31>, value: i32) -> crate::Result<()> {
    'block: {
      let Some(added) = self.available.checked_add(value) else {
        break 'block;
      };
      if added > U31::MAX.i32() {
        break 'block;
      }
      self.available = added;
      return Ok(());
    };
    if let Some(elem) = stream_id {
      Err(crate::Error::Http2ErrorReset(
        Http2ErrorCode::FlowControlError,
        Some(Http2Error::InvalidWindowUpdateSize),
        elem.u32(),
      ))
    } else {
      Err(crate::Error::Http2ErrorGoAway(
        Http2ErrorCode::FlowControlError,
        Some(Http2Error::InvalidWindowUpdateSize),
      ))
    }
  }

  pub(crate) fn update(&mut self, value: i32) {
    let diff = self.original.wrapping_sub(value);
    self.available = self.original.wrapping_sub(self.used()).wrapping_sub(diff);
    self.original = value;
  }

  #[inline]
  const fn is_invalid(&self) -> bool {
    self.available < 0
  }

  #[inline]
  fn used(&self) -> i32 {
    self.original.wrapping_sub(self.available)
  }

  #[inline]
  fn withdrawn(&mut self, value: i32) {
    self.available = self.available.wrapping_sub(value);
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
      recv: Window::new(U31::from_u32(hp.initial_window_len()).i32()),
      send: Window::new(initial_window_len!()),
    }
  }

  /// Used in initial streams.
  #[inline]
  pub(crate) const fn stream(hp: &Http2Params, hps: &Http2ParamsSend) -> Self {
    Self {
      recv: Window::new(U31::from_u32(hp.initial_window_len()).i32()),
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

  /// Available - Send
  #[inline]
  pub(crate) fn available_send(&self) -> i32 {
    self.stream.send.available()
  }

  /// withdrawn - Receive
  ///
  /// Controls window sizes received from external sources. Invalid or negative values trigger a
  /// frame dispatch to return to the default window size.
  #[inline]
  pub(crate) async fn withdrawn_recv<S>(
    &mut self,
    hp: &Http2Params,
    is_conn_open: bool,
    stream: &mut S,
    stream_id: U31,
    value: U31,
  ) -> crate::Result<()>
  where
    S: Stream,
  {
    let iwl = U31::from_u32(hp.initial_window_len()).i32();
    self.conn.recv.withdrawn(value.i32());
    self.stream.recv.withdrawn(value.i32());
    match (self.conn.recv.is_invalid(), self.stream.recv.is_invalid()) {
      (false, false) => {}
      (false, true) => {
        let conn_value = self.conn.recv.available().abs().wrapping_add(iwl);
        self.conn.recv.deposit(Some(stream_id), conn_value)?;
        write_array(
          [&WindowUpdateFrame::new(U31::from_i32(conn_value), U31::ZERO)?.bytes()],
          is_conn_open,
          stream,
        )
        .await?;
      }
      (true, false) => {
        let stream_value = self.stream.recv.available().abs().wrapping_add(iwl);
        self.stream.recv.deposit(Some(stream_id), stream_value)?;
        write_array(
          [&WindowUpdateFrame::new(U31::from_i32(stream_value), stream_id)?.bytes()],
          is_conn_open,
          stream,
        )
        .await?;
      }
      (true, true) => {
        let conn_value = self.conn.recv.available().abs().wrapping_add(iwl);
        let stream_value = self.stream.recv.available().abs().wrapping_add(iwl);
        self.conn.recv.deposit(Some(stream_id), conn_value)?;
        self.stream.recv.deposit(Some(stream_id), stream_value)?;
        write_array(
          [
            &WindowUpdateFrame::new(U31::from_i32(conn_value), U31::ZERO)?.bytes(),
            &WindowUpdateFrame::new(U31::from_i32(stream_value), stream_id)?.bytes(),
          ],
          is_conn_open,
          stream,
        )
        .await?;
      }
    }
    Ok(())
  }

  /// Withdrawn - Send
  ///
  /// Withdraws connection and sending parameters
  #[inline]
  pub(crate) fn withdrawn_send(&mut self, value: U31) {
    self.conn.send.withdrawn(value.i32());
    self.stream.send.withdrawn(value.i32());
  }
}
