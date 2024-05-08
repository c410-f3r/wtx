use crate::http2::{
  SettingsFrame, MAX_BODY_LEN, MAX_BUFFERED_FRAMES_NUM, MAX_CACHED_HEADERS_LEN,
  MAX_EXPANDED_HEADERS_LEN, MAX_FRAME_LEN, MAX_FRAME_LEN_LOWER_BOUND, MAX_FRAME_LEN_UPPER_BOUND,
  MAX_RAPID_RESETS_NUM, MAX_STREAMS_NUM, READ_BUFFER_LEN, U31,
};

/// Indicates to a remote peer the receiving parameters of a connection as well as its streams.
///
/// Also states some configurations for local structures.
#[derive(Debug)]
pub struct Http2Params {
  enable_connect_protocol: bool,
  initial_window_len: U31,
  max_body_len: u32,
  max_buffered_frames_num: u8,
  max_cached_headers_len: (u32, u32),
  max_expanded_headers_len: u32,
  max_frame_len: u32,
  max_rapid_resets_num: u8,
  max_streams_num: u32,
  read_buffer_len: u32,
}

impl Http2Params {
  /// Enable connect protocol
  ///
  /// Allows the execution of other protocols like WebSockets within HTTP/2 connections. At the
  /// current time this parameter has no effect.
  ///
  /// Corresponds to `SETTINGS_ENABLE_CONNECT_PROTOCOL`. Defaults to `false`.
  pub fn enable_connect_protocol(&self) -> bool {
    self.enable_connect_protocol
  }

  /// Initial window length
  ///
  /// The initial amount of "credit" a counterpart can have for sending data.
  ///
  /// Corresponds to `SETTINGS_INITIAL_WINDOW_SIZE`. Capped within 0 ~ (2^31 - 1) bytes. Defaults
  /// to
  #[doc = concat!(initial_window_len!())]
  /// bytes.
  pub fn initial_window_len(&self) -> U31 {
    self.initial_window_len
  }

  /// Maximum request/response body length
  ///
  /// Or the maximum size allowed for the sum of the length of all data frames.
  ///
  /// Defaults to
  #[doc = concat!(max_body_len!())]
  /// bytes.
  pub fn max_body_len(&self) -> u32 {
    self.max_body_len
  }

  /// Maximum number of buffered frames per stream
  ///
  /// An implementation detail. Due to the concurrent nature of the HTTP/2 specification, it is
  /// necessary to temporally store received frames into an intermediary structure.
  ///
  /// Defaults to
  #[doc = concat!(max_buffered_frames_num!())]
  /// bytes.
  pub fn max_buffered_frames_num(&self) -> u8 {
    self.max_buffered_frames_num
  }

  /// Maximum cached headers length
  ///
  /// Related to HPACK, indicates the maximum length of the structure that holds cached decoded
  /// headers received from a counterpart.
  ///
  /// - The first parameter indicates the local HPACK ***decoder*** length that is externally
  ///   advertised and can become the remote HPACK ***encoder*** length.
  /// - The second parameter indicates the maximum local HPACK ***encoder*** length. In other words,
  ///   it doesn't allow external actors to dictate very large lengths.
  ///
  /// Corresponds to `SETTINGS_HEADER_TABLE_SIZE`. Defaults to
  #[doc = concat!(max_cached_headers_len!())]
  /// bytes.
  pub fn max_cached_headers_len(&self) -> (u32, u32) {
    self.max_cached_headers_len
  }

  /// Maximum expanded headers length
  ///
  /// Or the maximum length of the final Request/Response header. Contents may or may not originate
  /// from the HPACK structure that holds cached decoded headers.
  ///
  /// Corresponds to `SETTINGS_MAX_HEADER_LIST_SIZE`. Defaults to
  #[doc = concat!(max_expanded_headers_len!())]
  /// bytes.
  pub fn max_expanded_headers_len(&self) -> u32 {
    self.max_expanded_headers_len
  }

  /// Maximum frame length
  ///
  /// Avoids the reading of very large frames sent by external actors.
  ///
  /// Corresponds to `SETTINGS_MAX_FRAME_SIZE`. Capped within
  #[doc = concat!(max_frame_len_lower_bound!())]
  /// ~
  #[doc = concat!(max_frame_len_upper_bound!())]
  /// bytes. Defaults to
  #[doc = concat!(max_frame_len!())]
  /// bytes.
  pub fn max_frame_len(&self) -> u32 {
    self.max_frame_len
  }

  /// Maximum number of rapid resets
  ///
  /// A rapid reset happens when a peer sends an initial header followed by a RST_STREAM frame. This
  /// parameter is used to avoid CVE-2023-44487.
  ///
  /// Defaults to
  #[doc = concat!(max_rapid_resets_num!())]
  /// rapid resets.
  pub fn max_rapid_resets_num(&self) -> u8 {
    self.max_rapid_resets_num
  }

  /// Maximum number of active concurrent streams
  ///
  /// Corresponds to `SETTINGS_MAX_CONCURRENT_STREAMS`. Defaults to
  #[doc = concat!(max_streams_num!())]
  /// streams
  pub fn max_streams_num(&self) -> u32 {
    self.max_streams_num
  }

  /// Read Buffer Length.
  ///
  /// Allocated space intended to read bytes sent by external actors.
  ///
  /// Defaults to
  #[doc = concat!(read_buffer_len!())]
  /// streams
  pub fn read_buffer_len(&self) -> u32 {
    self.read_buffer_len
  }

  /// Mutable version of [Self::initial_window_len].
  pub fn set_initial_window_len(&mut self, value: U31) -> &mut Self {
    self.initial_window_len = value;
    self
  }

  /// Mutable version of [Self::max_body_len].
  pub fn set_max_body_len(&mut self, value: u32) -> &mut Self {
    self.max_body_len = value;
    self
  }

  /// Mutable version of [Self::max_buffered_frames_num].
  pub fn set_max_buffered_frames_num(&mut self, value: u8) -> &mut Self {
    self.max_buffered_frames_num = value;
    self
  }

  /// Mutable version of [Self::max_cached_headers_len].
  pub fn set_max_cached_headers_len(&mut self, value: (u32, u32)) -> &mut Self {
    self.max_cached_headers_len = value;
    self
  }

  /// Mutable version of [Self::max_expanded_headers_len].
  pub fn set_max_expanded_headers_len(&mut self, value: u32) -> &mut Self {
    self.max_expanded_headers_len = value;
    self
  }

  /// Mutable version of [Self::max_frame_len].
  pub fn set_max_frame_len(&mut self, value: u32) -> &mut Self {
    self.max_frame_len = value.clamp(MAX_FRAME_LEN_LOWER_BOUND, MAX_FRAME_LEN_UPPER_BOUND);
    self
  }

  /// Mutable version of [Self::max_rapid_resets_num].
  pub fn set_max_rapid_resets_num(&mut self, value: u8) -> &mut Self {
    self.max_rapid_resets_num = value;
    self
  }

  /// Mutable version of [Self::max_streams_num].
  pub fn set_max_streams_num(&mut self, value: u32) -> &mut Self {
    self.max_streams_num = value;
    self
  }

  /// Mutable version of [Self::read_buffer_len].
  pub fn set_read_buffer_len(&mut self, value: u32) -> &mut Self {
    self.read_buffer_len = value;
    self
  }

  pub(crate) fn to_settings_frame(&self) -> SettingsFrame {
    let mut settings_frame = SettingsFrame::empty();
    settings_frame.set_enable_connect_protocol(Some(self.enable_connect_protocol));
    settings_frame.set_header_table_size(Some(self.max_cached_headers_len.0));
    settings_frame.set_initial_window_size(Some(self.initial_window_len));
    settings_frame.set_max_concurrent_streams(Some(self.max_streams_num));
    settings_frame.set_max_frame_size(Some(self.max_frame_len));
    settings_frame.set_max_header_list_size(Some(self.max_expanded_headers_len));
    settings_frame
  }
}

impl Default for Http2Params {
  fn default() -> Self {
    Self {
      enable_connect_protocol: false,
      initial_window_len: U31::from_u32(initial_window_len!()),
      max_body_len: MAX_BODY_LEN,
      max_buffered_frames_num: MAX_BUFFERED_FRAMES_NUM,
      max_cached_headers_len: (MAX_CACHED_HEADERS_LEN, MAX_CACHED_HEADERS_LEN),
      max_expanded_headers_len: MAX_EXPANDED_HEADERS_LEN,
      max_frame_len: MAX_FRAME_LEN,
      max_rapid_resets_num: MAX_RAPID_RESETS_NUM,
      max_streams_num: MAX_STREAMS_NUM,
      read_buffer_len: READ_BUFFER_LEN,
    }
  }
}
