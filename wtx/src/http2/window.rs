use crate::{
  http2::{
    Http2Error, Http2ErrorCode, Http2Params, http2_params_send::Http2ParamsSend, misc::write_array,
    u31::U31, window_update_frame::WindowUpdateFrame,
  },
  stream::StreamWriter,
  sync::AtomicBool,
};

/// A "credit" system used to restrain the exchange of data in a single direction (receive or send).
#[derive(Clone, Copy, Debug)]
pub struct Window {
  available: i32,
}

impl Window {
  pub(crate) const fn new(available: i32) -> Self {
    Self { available }
  }

  /// The amount of data available to send or receive. Depends if you are a client or a server.
  pub(crate) const fn available(self) -> i32 {
    self.available
  }

  pub(crate) const fn deposit(&mut self, stream_id: Option<U31>, value: i32) -> crate::Result<()> {
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
      Err(crate::Error::Http2FlowControlError(Http2Error::InvalidWindowUpdateSize, elem.u32()))
    } else {
      Err(crate::Error::Http2ErrorGoAway(
        Http2ErrorCode::FlowControlError,
        Http2Error::InvalidWindowUpdateSize,
      ))
    }
  }

  pub(crate) const fn withdrawn(
    &mut self,
    stream_id: Option<U31>,
    value: i32,
  ) -> crate::Result<()> {
    let Some(diff) = self.available.checked_sub(value) else {
      return if let Some(elem) = stream_id {
        Err(crate::Error::Http2FlowControlError(Http2Error::InvalidWindowUpdateSize, elem.u32()))
      } else {
        Err(crate::Error::Http2ErrorGoAway(
          Http2ErrorCode::FlowControlError,
          Http2Error::InvalidWindowUpdateSize,
        ))
      };
    };
    self.available = diff;
    Ok(())
  }

  const fn is_invalid(self) -> bool {
    self.available <= 0
  }
}

/// A "credit" system used to restrain the exchange of data for both receiving and sending parts.
#[derive(Clone, Copy, Debug)]
pub struct Windows {
  /// Parameters used to received data. It is defined locally.
  recv: Window,
  /// Parameters used to send data. It is initially defined locally with default parameters
  /// and then defined by a remote peer.
  send: Window,
}

impl Windows {
  /// Used in initial connections/streams.
  pub(crate) const fn initial(hp: &Http2Params, hps: &Http2ParamsSend) -> Self {
    Self {
      recv: Window::new(U31::from_u32(hp.initial_window_len()).i32()),
      send: Window::new(hps.initial_window_len.i32()),
    }
  }

  pub(crate) const fn new() -> Self {
    Self { recv: Window::new(0), send: Window::new(0) }
  }

  /// Parameters used to received data. It is defined locally.
  pub const fn recv(&self) -> Window {
    self.recv
  }

  /// Parameters used to send data. It is initially defined locally with default parameters
  /// and then defined by a remote peer.
  pub const fn send(&self) -> Window {
    self.send
  }

  pub(crate) const fn send_mut(&mut self) -> &mut Window {
    &mut self.send
  }
}

#[derive(Debug)]
pub(crate) struct WindowsPair<'any> {
  pub(crate) conn: &'any mut Windows,
  pub(crate) stream: &'any mut Windows,
}

impl<'any> WindowsPair<'any> {
  pub(crate) const fn new(conn: &'any mut Windows, stream: &'any mut Windows) -> Self {
    Self { conn, stream }
  }

  /// Available - Send
  pub(crate) const fn available_send(&self) -> i32 {
    self.stream.send.available()
  }

  /// Withdrawn - Receive
  ///
  /// Controls window sizes received from external sources. Invalid or negative values trigger a
  /// frame dispatch to return to the default window size.
  pub(crate) async fn withdrawn_recv<SW>(
    &mut self,
    hp: &Http2Params,
    is_conn_open: &AtomicBool,
    stream_writer: &mut SW,
    stream_id: U31,
    value: U31,
  ) -> crate::Result<()>
  where
    SW: StreamWriter,
  {
    let iwl = U31::from_u32(hp.initial_window_len()).i32();
    self.conn.recv.withdrawn(None, value.i32())?;
    self.stream.recv.withdrawn(Some(stream_id), value.i32())?;
    match (self.conn.recv.is_invalid(), self.stream.recv.is_invalid()) {
      (false, false) => {}
      (false, true) => {
        let stream_value = self.stream.recv.available().abs().wrapping_add(iwl);
        self.stream.recv.deposit(Some(stream_id), stream_value)?;
        write_array(
          [&WindowUpdateFrame::new(U31::from_i32(stream_value), stream_id)?.bytes()],
          is_conn_open,
          stream_writer,
        )
        .await?;
      }
      (true, false) => {
        let conn_value = self.conn.recv.available().abs().wrapping_add(iwl);
        self.conn.recv.deposit(Some(stream_id), conn_value)?;
        write_array(
          [&WindowUpdateFrame::new(U31::from_i32(conn_value), U31::ZERO)?.bytes()],
          is_conn_open,
          stream_writer,
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
          stream_writer,
        )
        .await?;
      }
    }

    Ok(())
  }

  /// Withdrawn - Send
  ///
  /// Used when sending data frames
  pub(crate) fn withdrawn_send(&mut self, stream_id: Option<U31>, value: U31) -> crate::Result<()> {
    self.conn.send.withdrawn(None, value.i32())?;
    self.stream.send.withdrawn(stream_id, value.i32())?;
    Ok(())
  }
}
