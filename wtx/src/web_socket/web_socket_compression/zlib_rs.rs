use crate::{
  codec::{Compression, CompressionFlush, Decompression, DecompressionFlush, FromRadix10 as _},
  collections::{ArrayVectorCopy, Vector},
  http::GenericHeader,
  misc::bytes_split1,
  web_socket::{
    DeflateConfig, WebSocketError, WsCompression,
    web_socket_compression::{
      NegotiatedWsCompression, WebSocketCompression, WebSocketDecompression,
    },
  },
};
use core::fmt::Debug;
use zlib_rs::{Deflate, Inflate};

/// Initial compression extension
#[derive(Clone, Debug)]
pub struct ZlibRs {
  /// See [`DeflateConfig`].
  pub dc: DeflateConfig,
}

impl From<DeflateConfig> for ZlibRs {
  #[inline]
  fn from(dc: DeflateConfig) -> Self {
    Self { dc }
  }
}

impl<const IS_CLIENT: bool> WsCompression<IS_CLIENT> for ZlibRs {
  type NegotiatedCompression = Option<NegotiatedZlibRs>;

  #[inline]
  fn negotiate(
    self,
    headers: impl Iterator<Item = impl GenericHeader>,
  ) -> crate::Result<Self::NegotiatedCompression> {
    let swe_bytes = b"Sec-WebSocket-Extensions";
    let mut final_client_max_window_bits = false;
    let mut final_client_no_context_takeover = false;
    let mut final_dc = None;
    let mut final_server_max_window_bits = false;
    let mut final_server_no_context_takeover = false;

    'outer: for swe in headers.filter(|el| el.name().eq_ignore_ascii_case(swe_bytes)) {
      for permessage_deflate_option in bytes_split1(swe.value(), b',') {
        let mut client_max_window_bits_flag = false;
        let mut client_no_context_takeover_flag = false;
        let mut dc = self.dc;
        let mut is_permessage_deflate = false;
        let mut is_valid = true;
        let mut server_max_window_bits_flag = false;
        let mut server_no_context_takeover_flag = false;

        for param in bytes_split1(permessage_deflate_option, b';').map(<[u8]>::trim_ascii) {
          if param == b"permessage-deflate" {
            is_permessage_deflate = true;
          } else if param == b"client_no_context_takeover" {
            manage_header_uniqueness(&mut client_no_context_takeover_flag, || Ok(()))?;
          } else if param == b"server_no_context_takeover" {
            manage_header_uniqueness(&mut server_no_context_takeover_flag, || Ok(()))?;
          } else if let Some(after_cmwb) = param.strip_prefix(b"client_max_window_bits") {
            manage_header_uniqueness(&mut client_max_window_bits_flag, || {
              if let Some(value) = num_after_eq(after_cmwb) {
                dc.client_max_window_bits = value.try_into()?;
              }
              Ok(())
            })?;
          } else if let Some(after_smwb) = param.strip_prefix(b"server_max_window_bits") {
            manage_header_uniqueness(&mut server_max_window_bits_flag, || {
              if let Some(value) = num_after_eq(after_smwb) {
                dc.server_max_window_bits = value.try_into()?;
              }
              Ok(())
            })?;
          } else {
            is_valid = false;
            break;
          }
        }

        if is_permessage_deflate && is_valid {
          final_client_max_window_bits = client_max_window_bits_flag;
          final_client_no_context_takeover = client_no_context_takeover_flag;
          final_dc = Some(dc);
          final_server_max_window_bits = server_max_window_bits_flag;
          final_server_no_context_takeover = server_no_context_takeover_flag;
          break 'outer;
        }
      }
    }

    let Some(dc) = final_dc else {
      return Ok(None);
    };

    let decoder_nct =
      if IS_CLIENT { final_server_no_context_takeover } else { final_client_no_context_takeover };
    let encoder_nct =
      if IS_CLIENT { final_client_no_context_takeover } else { final_server_no_context_takeover };

    let decoder_mwb_flag =
      if IS_CLIENT { final_server_max_window_bits } else { final_client_max_window_bits };
    let encoder_mwb_flag =
      if IS_CLIENT { final_client_max_window_bits } else { final_server_max_window_bits };

    let decoder_mwb_val =
      if IS_CLIENT { dc.server_max_window_bits } else { dc.client_max_window_bits };
    let encoder_mwb_val =
      if IS_CLIENT { dc.client_max_window_bits } else { dc.server_max_window_bits };

    Ok(Some(NegotiatedZlibRs {
      deflate: Deflate::new(u8::from(dc.compression_level).into(), false, encoder_mwb_val.into()),
      dc,
      inflate: Inflate::new(false, decoder_mwb_val.into()),
      max_window_bits: (decoder_mwb_flag, encoder_mwb_flag),
      no_context_takeover: (decoder_nct, encoder_nct),
    }))
  }

  #[inline]
  fn req_headers(&self) -> ArrayVectorCopy<u8, 160> {
    build_headers(self.dc, true, (true, false), (true, false))
  }
}

impl Default for ZlibRs {
  #[inline]
  fn default() -> Self {
    ZlibRs::from(DeflateConfig::default())
  }
}

/// Negotiated Zlib-rs compression state
pub struct NegotiatedZlibRs {
  dc: DeflateConfig,
  deflate: Deflate,
  inflate: Inflate,
  max_window_bits: (bool, bool),
  no_context_takeover: (bool, bool),
}

impl Compression for NegotiatedZlibRs {
  #[inline]
  fn compress(
    &mut self,
    flush: CompressionFlush,
    input: &[u8],
    output: &mut Vector<u8>,
  ) -> crate::Result<usize> {
    <Deflate as Compression>::compress(&mut self.deflate, flush, input, output)
  }

  #[inline]
  fn compress_ub(&self, len: usize) -> usize {
    <Deflate as Compression>::compress_ub(&self.deflate, len)
  }

  #[inline]
  fn reset(&mut self) {
    <Deflate as Compression>::reset(&mut self.deflate);
  }
}

impl Decompression for NegotiatedZlibRs {
  #[inline]
  fn decompress(
    &mut self,
    flush: DecompressionFlush,
    input: &[u8],
    output: &mut Vector<u8>,
  ) -> crate::Result<usize> {
    <Inflate as Decompression>::decompress(&mut self.inflate, flush, input, output)
  }

  #[inline]
  fn reset(&mut self) {
    <Inflate as Decompression>::reset(&mut self.inflate);
  }
}

impl NegotiatedWsCompression for NegotiatedZlibRs {
  type Compression = NegotiatedZlibRsCompression;
  type Decompression = NegotiatedZlibRsDecompression;

  #[inline]
  fn into_split(self) -> (Self::Compression, Self::Decompression) {
    (
      NegotiatedZlibRsCompression {
        deflate: self.deflate,
        no_context_takeover: self.no_context_takeover.1,
      },
      NegotiatedZlibRsDecompression {
        inflate: self.inflate,
        no_context_takeover: self.no_context_takeover.0,
      },
    )
  }

  #[inline]
  fn rsv1(&self) -> u8 {
    0b0100_0000
  }

  #[inline]
  fn res_headers(&self) -> ArrayVectorCopy<u8, 160> {
    build_headers(
      self.dc,
      false,
      (self.max_window_bits.0, self.no_context_takeover.0),
      (self.max_window_bits.0, self.no_context_takeover.1),
    )
  }
}

impl WebSocketCompression for NegotiatedZlibRs {
  #[inline]
  fn no_context_takeover(&self) -> bool {
    self.no_context_takeover.1
  }
}

impl WebSocketDecompression for NegotiatedZlibRs {
  #[inline]
  fn no_context_takeover(&self) -> bool {
    self.no_context_takeover.0
  }
}

impl Debug for NegotiatedZlibRs {
  #[inline]
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    f.debug_struct("NegotiatedZlibRs").finish()
  }
}

/// Compression part
pub struct NegotiatedZlibRsCompression {
  deflate: Deflate,
  no_context_takeover: bool,
}

impl Compression for NegotiatedZlibRsCompression {
  #[inline]
  fn compress(
    &mut self,
    flush: CompressionFlush,
    input: &[u8],
    output: &mut Vector<u8>,
  ) -> crate::Result<usize> {
    <Deflate as Compression>::compress(&mut self.deflate, flush, input, output)
  }

  #[inline]
  fn compress_ub(&self, len: usize) -> usize {
    <Deflate as Compression>::compress_ub(&self.deflate, len)
  }

  #[inline]
  fn reset(&mut self) {
    <Deflate as Compression>::reset(&mut self.deflate);
  }
}

impl WebSocketCompression for NegotiatedZlibRsCompression {
  #[inline]
  fn no_context_takeover(&self) -> bool {
    self.no_context_takeover
  }
}

impl Debug for NegotiatedZlibRsCompression {
  #[inline]
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    f.debug_struct("NegotiatedZlibRsCompression").finish()
  }
}

/// Decompression part
pub struct NegotiatedZlibRsDecompression {
  inflate: Inflate,
  no_context_takeover: bool,
}

impl Decompression for NegotiatedZlibRsDecompression {
  #[inline]
  fn decompress(
    &mut self,
    flush: DecompressionFlush,
    input: &[u8],
    output: &mut Vector<u8>,
  ) -> crate::Result<usize> {
    <Inflate as Decompression>::decompress(&mut self.inflate, flush, input, output)
  }

  #[inline]
  fn reset(&mut self) {
    <Inflate as Decompression>::reset(&mut self.inflate);
  }
}

impl WebSocketDecompression for NegotiatedZlibRsDecompression {
  #[inline]
  fn no_context_takeover(&self) -> bool {
    self.no_context_takeover
  }
}

impl Debug for NegotiatedZlibRsDecompression {
  #[inline]
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    f.debug_struct("NegotiatedZlibRsDecompression").finish()
  }
}

#[inline]
fn build_headers(
  dc: DeflateConfig,
  is_request: bool,
  client: (bool, bool),
  server: (bool, bool),
) -> ArrayVectorCopy<u8, 160> {
  let mut array = ArrayVectorCopy::new();
  let _rslt0 = array.extend_from_copyable_slice(b"Sec-WebSocket-Extensions: permessage-deflate");
  if client.0 {
    if is_request {
      let _rslt1 = array.extend_from_copyable_slice(b"; client_max_window_bits");
    } else {
      let _rslt1 = array.extend_from_copyable_slices([
        b"; client_max_window_bits=",
        dc.client_max_window_bits.strings().number.as_bytes(),
      ]);
    }
  }
  if client.1 {
    let _rslt2 = array.extend_from_copyable_slice(b"; client_no_context_takeover");
  }
  if server.0 {
    let _rslt3 = array.extend_from_copyable_slices([
      b"; server_max_window_bits=",
      dc.server_max_window_bits.strings().number.as_bytes(),
    ]);
  }
  if server.1 {
    let _rslt4 = array.extend_from_copyable_slice(b"; server_no_context_takeover");
  }
  let _rslt5 = array.extend_from_copyable_slice(b"\r\n");
  array
}

#[inline]
fn manage_header_uniqueness(
  flag: &mut bool,
  mut cb: impl FnMut() -> crate::Result<()>,
) -> crate::Result<()> {
  if *flag {
    Err(WebSocketError::DuplicatedHeader.into())
  } else {
    cb()?;
    *flag = true;
    Ok(())
  }
}

#[inline]
fn num_after_eq(bytes: &[u8]) -> Option<u8> {
  let after_equals = bytes_split1(bytes, b'=').nth(1)?;
  u8::from_radix_10(after_equals).ok()
}
