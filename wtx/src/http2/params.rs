use crate::http2::{SettingsFrame, StreamId};
use core::time::Duration;

#[derive(Debug)]
pub struct AcceptParams {
  initial_target_connection_window_size: Option<u32>,
  max_send_buffer_size: usize,
  pending_accept_reset_stream_max: usize,
  reset_stream_duration: Duration,
  reset_stream_max: usize,
  settings: SettingsFrame,
}

impl Default for AcceptParams {
  fn default() -> Self {
    Self {
      initial_target_connection_window_size: None,
      max_send_buffer_size: 1,
      pending_accept_reset_stream_max: 1,
      reset_stream_duration: Duration::from_secs(1),
      reset_stream_max: 1,
      settings: SettingsFrame::default(),
    }
  }
}

#[derive(Debug)]
pub struct ConnectParams {
  pub(crate) max_send_buffer_size: usize,
  pub(crate) pending_accept_reset_stream_max: usize,
  pub(crate) reset_stream_duration: Duration,
  pub(crate) reset_stream_max: usize,
  pub(crate) settings: SettingsFrame,
  pub(crate) stream_id: StreamId,
}

impl ConnectParams {
  /// This setting indicates the sender's initial window size (in units of octets) for stream-level
  /// flow control.
  ///
  /// https://datatracker.ietf.org/doc/html/rfc9113#SETTINGS_INITIAL_WINDOW_SIZE
  pub fn initial_window_size(&mut self, size: u32) -> &mut Self {
    self.settings.set_initial_window_size(Some(size));
    self
  }

  /// This setting indicates the size of the largest frame payload that the sender is willing to
  /// receive, in units of octets.
  ///
  /// https://datatracker.ietf.org/doc/html/rfc9113#SETTINGS_MAX_FRAME_SIZE
  pub fn max_frame_size(&mut self, max: u32) -> &mut Self {
    self.settings.set_max_frame_size(Some(max));
    self
  }

  /// This advisory setting informs a peer of the maximum field section size that the sender is
  /// prepared to accept, in units of octets.
  ///
  /// https://datatracker.ietf.org/doc/html/rfc9113#SETTINGS_MAX_HEADER_LIST_SIZE
  pub fn max_header_list_size(&mut self, max: u32) -> &mut Self {
    self.settings.set_max_header_list_size(Some(max));
    self
  }

  /// This setting indicates the maximum number of concurrent streams that the sender will allow.
  ///
  /// https://datatracker.ietf.org/doc/html/rfc9113#SETTINGS_MAX_CONCURRENT_STREAMS
  pub fn max_concurrent_streams(&mut self, max: u32) -> &mut Self {
    self.settings.set_max_concurrent_streams(Some(max));
    self
  }

  /// Sets the maximum number of pending-accept remotely-reset streams.
  pub fn max_pending_accept_reset_streams(&mut self, max: usize) -> &mut Self {
    self.pending_accept_reset_stream_max = max;
    self
  }

  /// This setting allows the sender to inform the remote endpoint of the maximum size of the
  /// compression table used to decode field blocks, in units of octets.
  ///
  /// https://datatracker.ietf.org/doc/html/rfc9113#SETTINGS_HEADER_TABLE_SIZE
  pub fn header_table_size(&mut self, size: u32) -> &mut Self {
    self.settings.set_header_table_size(Some(size));
    self
  }
}

impl Default for ConnectParams {
  fn default() -> Self {
    Self {
      max_send_buffer_size: 0,
      pending_accept_reset_stream_max: 0,
      reset_stream_duration: Duration::from_secs(0),
      reset_stream_max: 0,
      settings: Default::default(),
      stream_id: 1.into(),
    }
  }
}
