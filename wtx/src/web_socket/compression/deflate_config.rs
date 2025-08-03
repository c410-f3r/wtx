use crate::web_socket::compression::{CompressionLevel, WindowBits};

/// Configurations for the `permessage-deflate` extension from the IETF RFC 7692
#[derive(Clone, Copy, Debug)]
pub struct DeflateConfig {
  /// LZ77 sliding window size for the client.
  pub client_max_window_bits: WindowBits,
  /// Compression level.
  pub compression_level: CompressionLevel,
  /// LZ77 sliding window size for the server.
  pub server_max_window_bits: WindowBits,
}

impl Default for DeflateConfig {
  #[inline]
  fn default() -> Self {
    DeflateConfig {
      client_max_window_bits: WindowBits::Twelve,
      compression_level: CompressionLevel::default(),
      server_max_window_bits: WindowBits::Twelve,
    }
  }
}
