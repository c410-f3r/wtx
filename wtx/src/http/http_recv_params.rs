use crate::http::{
  DEFAULT_INITIAL_WINDOW_LEN, DEFAULT_MAX_CONCURRENT_STREAMS_NUM, DEFAULT_MAX_FRAME_LEN,
  DEFAULT_MAX_HEADERS_LEN, DEFAULT_MAX_HPACK_LEN, MAX_FRAME_LEN_LOWER_BOUND,
  MAX_FRAME_LEN_UPPER_BOUND, u31::U31,
};

/// Indicates to a remote peer the receiving parameters of a connection as well as its streams.
#[derive(Clone, Copy, Debug)]
pub struct HttpRecvParams {
  enable_connect_protocol: bool,
  initial_window_len: U31,
  max_body_len: u32,
  max_concurrent_streams_num: u32,
  max_frame_len: u32,
  max_headers_len: u32,
  max_hpack_len: (u32, u32),
  max_recv_streams_num: u32,
}

impl HttpRecvParams {
  /// New instance with the default parameters declared by the official specification.
  ///
  /// These parameters are outdated and don't reflect reality nowadays.
  #[inline]
  pub const fn with_default_params() -> Self {
    Self {
      enable_connect_protocol: false,
      initial_window_len: U31::from_u32(DEFAULT_INITIAL_WINDOW_LEN),
      max_body_len: 1024 * 1024,
      max_concurrent_streams_num: DEFAULT_MAX_CONCURRENT_STREAMS_NUM,
      max_frame_len: DEFAULT_MAX_FRAME_LEN,
      max_headers_len: DEFAULT_MAX_HEADERS_LEN,
      max_hpack_len: (DEFAULT_MAX_HPACK_LEN, DEFAULT_MAX_HPACK_LEN),
      max_recv_streams_num: 32,
    }
  }

  /// New instance with optioned default parameters that allows the interaction with most
  /// modern websites.
  #[inline]
  pub const fn with_optioned_params() -> Self {
    Self {
      enable_connect_protocol: false,
      initial_window_len: U31::from_i32(4 * 1024 * 1024),
      max_body_len: 64 * 1024 * 1024,
      max_concurrent_streams_num: 256,
      max_frame_len: 64 * 1024,
      max_headers_len: 64 * 1024,
      max_hpack_len: (128 * 1024, 128 * 1024),
      max_recv_streams_num: 256,
    }
  }

  /// New instance without constraints. Not recommended for production but potentially useful for testing.
  #[inline]
  pub const fn with_permissive_params() -> Self {
    Self {
      enable_connect_protocol: false,
      initial_window_len: U31::MAX,
      max_body_len: u32::MAX,
      max_concurrent_streams_num: u32::MAX,
      max_frame_len: u32::MAX,
      max_headers_len: u32::MAX,
      max_hpack_len: (u32::MAX, u32::MAX),
      max_recv_streams_num: u32::MAX,
    }
  }

  /// Enable connect protocol
  ///
  /// Servers only. Allows the execution of other protocols like WebSockets within HTTP/2
  /// connections.
  ///
  /// Corresponds to `SETTINGS_ENABLE_CONNECT_PROTOCOL`. Defaults to `false`.
  #[inline]
  pub const fn enable_connect_protocol(&self) -> bool {
    self.enable_connect_protocol
  }

  /// Initial window length
  ///
  /// The initial amount of "credit" a counterpart can have for sending data.
  ///
  /// Corresponds to `SETTINGS_INITIAL_WINDOW_SIZE`. Capped within 0 ~ (2^31 - 1) bytes.
  #[inline]
  pub const fn initial_window_len(&self) -> u32 {
    self.initial_window_len.u32()
  }

  /// Maximum request/response body length
  ///
  /// Or the maximum size allowed for the sum of the length of all data frames.
  #[inline]
  pub const fn max_body_len(&self) -> u32 {
    self.max_body_len
  }

  /// Maximum number of active concurrent streams
  ///
  /// Corresponds to `SETTINGS_MAX_CONCURRENT_STREAMS`.
  #[inline]
  pub const fn max_concurrent_streams_num(&self) -> u32 {
    self.max_concurrent_streams_num
  }

  /// Maximum headers length
  ///
  /// The final Request/Response header is composed by the sum of headers and trailers. Contents
  /// may or may not originate from the HPACK structure that holds cached decoded headers.
  ///
  /// Corresponds to `SETTINGS_MAX_HEADER_LIST_SIZE`.
  #[inline]
  pub const fn max_headers_len(&self) -> u32 {
    self.max_headers_len
  }

  /// Maximum HPACK length
  ///
  /// Indicates the maximum length of the HPACK structure that holds cached decoded headers
  /// received from a counterpart.
  ///
  /// * The first parameter indicates the local HPACK ***decoder*** length that is externally
  ///   advertised and can become the remote HPACK ***encoder*** length.
  /// * The second parameter indicates the maximum local HPACK ***encoder*** length. In other words,
  ///   it doesn't allow external actors to dictate very large lengths.
  ///
  /// Corresponds to `SETTINGS_HEADER_TABLE_SIZE`.
  #[inline]
  pub const fn max_hpack_len(&self) -> (u32, u32) {
    self.max_hpack_len
  }

  /// Maximum frame ***payload*** length
  ///
  /// Avoids the reading of very large payload frames sent by external actors.
  ///
  /// Corresponds to `SETTINGS_MAX_FRAME_SIZE`.
  #[inline]
  pub const fn max_frame_len(&self) -> u32 {
    self.max_frame_len
  }

  /// Maximum number of receiving streams
  ///
  /// Servers only. Prevents clients from opening more than the specified number of streams.
  #[inline]
  pub const fn max_recv_streams_num(&self) -> u32 {
    self.max_recv_streams_num
  }

  /// Mutable version of [`Self::enable_connect_protocol`].
  #[inline]
  #[must_use]
  pub const fn set_enable_connect_protocol(mut self, value: bool) -> Self {
    self.enable_connect_protocol = value;
    self
  }

  /// Mutable version of [`Self::initial_window_len`].
  #[inline]
  #[must_use]
  pub const fn set_initial_window_len(mut self, value: u32) -> Self {
    self.initial_window_len = U31::from_u32(value);
    self
  }

  /// Mutable version of [`Self::max_body_len`].
  #[inline]
  #[must_use]
  pub const fn set_max_body_len(mut self, value: u32) -> Self {
    self.max_body_len = value;
    self
  }

  /// Mutable version of [`Self::max_concurrent_streams_num`].
  #[inline]
  #[must_use]
  pub const fn set_max_concurrent_streams_num(mut self, value: u32) -> Self {
    self.max_concurrent_streams_num = value;
    self
  }

  /// Mutable version of [`Self::max_headers_len`].
  #[inline]
  #[must_use]
  pub const fn set_max_headers_len(mut self, value: u32) -> Self {
    self.max_headers_len = value;
    self
  }

  /// Mutable version of [`Self::max_hpack_len`].
  #[inline]
  #[must_use]
  pub const fn set_max_hpack_len(mut self, value: (u32, u32)) -> Self {
    self.max_hpack_len = value;
    self
  }

  /// Mutable version of [`Self::max_frame_len`].
  #[inline]
  #[must_use]
  pub const fn set_max_frame_len(mut self, value: u32) -> Self {
    // FIXME(STABLE): Use constant `clamp`
    self.max_frame_len = if value < MAX_FRAME_LEN_LOWER_BOUND {
      MAX_FRAME_LEN_LOWER_BOUND
    } else if value > MAX_FRAME_LEN_UPPER_BOUND {
      MAX_FRAME_LEN_UPPER_BOUND
    } else {
      value
    };
    self
  }

  /// Mutable version of [`Self::max_recv_streams_num`].
  #[inline]
  #[must_use]
  pub const fn set_max_recv_streams_num(mut self, value: u32) -> Self {
    self.max_recv_streams_num = value;
    self
  }
}
