use crate::http2::{
  SettingsFrame, BODY_LEN_DEFAULT, BUFFERED_FRAMES_NUM_DEFAULT, CACHED_HEADERS_LEN_DEFAULT,
  EXPANDED_HEADERS_LEN_DEFAULT, FRAME_LEN_DEFAULT, FRAME_LEN_LOWER_BOUND, FRAME_LEN_UPPER_BOUND,
  INITIAL_WINDOW_LEN_DEFAULT, RAPID_RESETS_NUM_DEFAULT, READ_BUFFER_LEN_DEFAULT,
  STREAMS_NUM_DEFAULT, U31,
};

/// Tells how connections and streams should behave.
#[derive(Debug)]
pub struct Http2Params {
  enable_connect_protocol: bool,
  initial_window_len: u32,
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
  /// Defaults to `false`.
  pub fn enable_connect_protocol(&self) -> bool {
    self.enable_connect_protocol
  }

  /// Initial window length.
  ///
  /// The initial amount of "credit" a counterpart can have for sending data.
  ///
  /// Corresponds to `SETTINGS_INITIAL_WINDOW_SIZE`. Defaults to 512 KiB. Capped within
  /// 0 ~ (2^31 - 1) bytes
  pub fn initial_window_len(&self) -> u32 {
    self.initial_window_len
  }

  /// Maximum request/response body length.
  ///
  /// Or the maximum size allowed for the sum of the length of all data frames. Also servers as an
  /// upper bound for the window size of a stream.
  ///
  /// Defaults to 4 MiB.
  pub fn max_body_len(&self) -> u32 {
    self.max_body_len
  }

  /// Maximum number of buffered frames per stream.
  ///
  /// An implementation detail. Due to the concurrent nature of the HTTP/2 specification, it is
  /// necessary to temporally store received frames into an intermediary structure.
  ///
  /// Defaults to 16 frames.
  pub fn max_buffered_frames_num(&self) -> u8 {
    self.max_buffered_frames_num
  }

  /// Maximum cached headers length.
  ///
  /// Related to HPACK, indicates the maximum length of the structure that holds cached decoded
  /// headers received from a counterpart.
  ///
  /// - The first parameter indicates the local HPACK ***decoder*** length that is externally
  ///   advertised and can become the remote HPACK ***encoder*** length.
  /// - The second parameter indicates the maximum local HPACK ***encoder*** length. In other words,
  ///   it doesn't allow external actors to dictate very large lengths.
  ///
  /// Corresponds to `SETTINGS_HEADER_TABLE_SIZE`. Defaults to 8 KiB.
  pub fn max_cached_headers_len(&self) -> (u32, u32) {
    self.max_cached_headers_len
  }

  /// Maximum expanded headers length.
  ///
  /// Or the maximum length of the final Request/Response header. Contents may or may not originate
  /// from the HPACK structure that holds cached decoded headers.
  ///
  /// Corresponds to `SETTINGS_MAX_HEADER_LIST_SIZE`. Defaults to 4 KiB.
  pub fn max_expanded_headers_len(&self) -> u32 {
    self.max_expanded_headers_len
  }

  /// Maximum frame length.
  ///
  /// Avoids the reading of very large frames sent by external actors.
  ///
  /// Corresponds to `SETTINGS_MAX_FRAME_SIZE`. Defaults to 1 MiB. Capped within 16 KiB ~ 16 MiB
  pub fn max_frame_len(&self) -> u32 {
    self.max_frame_len
  }

  /// Maximum number of rapid resets.
  ///
  /// A rapid reset happens when a peer sends an initial header followed by a RST_STREAM frame. This
  /// parameter is used to avoid CVE-2023-44487.
  pub fn max_rapid_resets_num(&self) -> u8 {
    self.max_rapid_resets_num
  }

  /// Maximum number of active concurrent streams.
  ///
  /// Corresponds to `SETTINGS_MAX_CONCURRENT_STREAMS`. Defaults to 1073741824 streams.
  pub fn max_streams_num(&self) -> u32 {
    self.max_streams_num
  }

  /// Read Buffer Length.
  ///
  /// Allocated space intended to read bytes sent by external actors.
  ///
  /// Defaults to 4 MiB.
  pub fn read_buffer_len(&self) -> u32 {
    self.read_buffer_len
  }

  /// Mutable version of [Self::initial_window_len].
  pub fn set_initial_window_len(&mut self, value: u32) -> &mut Self {
    self.initial_window_len = value.clamp(0, U31::MAX.u32());
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
    self.max_frame_len = value.clamp(FRAME_LEN_LOWER_BOUND, FRAME_LEN_UPPER_BOUND);
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
      initial_window_len: INITIAL_WINDOW_LEN_DEFAULT,
      max_body_len: BODY_LEN_DEFAULT,
      max_buffered_frames_num: BUFFERED_FRAMES_NUM_DEFAULT,
      max_cached_headers_len: (CACHED_HEADERS_LEN_DEFAULT, CACHED_HEADERS_LEN_DEFAULT),
      max_expanded_headers_len: EXPANDED_HEADERS_LEN_DEFAULT,
      max_frame_len: FRAME_LEN_DEFAULT,
      max_rapid_resets_num: RAPID_RESETS_NUM_DEFAULT,
      max_streams_num: STREAMS_NUM_DEFAULT,
      read_buffer_len: READ_BUFFER_LEN_DEFAULT,
    }
  }
}
