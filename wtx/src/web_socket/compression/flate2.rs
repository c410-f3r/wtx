use crate::{
  http::{GenericHeader, KnownHeaderName},
  misc::{FromRadix10, SuffixWriterFbvm, bytes_split1},
  web_socket::{Compression, DeflateConfig, WebSocketError, compression::NegotiatedCompression},
};
use flate2::{Compress, Decompress, FlushCompress, FlushDecompress};

/// Initial Flate2 compression
#[derive(Debug)]
pub struct Flate2 {
  dc: DeflateConfig,
}

impl From<DeflateConfig> for Flate2 {
  #[inline]
  fn from(dc: DeflateConfig) -> Self {
    Self { dc }
  }
}

impl<const IS_CLIENT: bool> Compression<IS_CLIENT> for Flate2 {
  type NegotiatedCompression = Option<NegotiatedFlate2>;

  #[inline]
  fn negotiate(
    self,
    headers: impl Iterator<Item = impl GenericHeader>,
  ) -> crate::Result<Self::NegotiatedCompression> {
    let mut dc = DeflateConfig {
      client_max_window_bits: self.dc.client_max_window_bits,
      compression_level: self.dc.compression_level,
      server_max_window_bits: self.dc.server_max_window_bits,
    };

    let mut has_extension = false;

    let swe_bytes = KnownHeaderName::SecWebsocketExtensions.into();
    for swe in headers.filter(|el| el.name().eq_ignore_ascii_case(swe_bytes)) {
      for permessage_deflate_option in bytes_split1(swe.value(), b',') {
        dc = DeflateConfig {
          client_max_window_bits: self.dc.client_max_window_bits,
          compression_level: self.dc.compression_level,
          server_max_window_bits: self.dc.server_max_window_bits,
        };
        let mut client_max_window_bits_flag = false;
        let mut permessage_deflate_flag = false;
        let mut server_max_window_bits_flag = false;
        for param in bytes_split1(permessage_deflate_option, b';').map(<[u8]>::trim_ascii) {
          if param == b"client_no_context_takeover" || param == b"server_no_context_takeover" {
          } else if param == b"permessage-deflate" {
            manage_header_uniqueness(&mut permessage_deflate_flag, || Ok(()))?
          } else if let Some(after_cmwb) = param.strip_prefix(b"client_max_window_bits") {
            manage_header_uniqueness(&mut client_max_window_bits_flag, || {
              if let Some(value) = byte_from_bytes(after_cmwb) {
                dc.client_max_window_bits = value.try_into()?;
              }
              Ok(())
            })?;
          } else if let Some(after_smwb) = param.strip_prefix(b"server_max_window_bits") {
            manage_header_uniqueness(&mut server_max_window_bits_flag, || {
              if let Some(value) = byte_from_bytes(after_smwb) {
                dc.server_max_window_bits = value.try_into()?;
              }
              Ok(())
            })?;
          } else {
            return Err(WebSocketError::InvalidCompressionHeaderParameter.into());
          }
        }
        if !permessage_deflate_flag {
          return Err(WebSocketError::InvalidCompressionHeaderParameter.into());
        }
        has_extension = true;
      }
    }

    if !has_extension {
      return Ok(None);
    }

    let decoder_wb = if IS_CLIENT { dc.server_max_window_bits } else { dc.client_max_window_bits };
    let encoder_wb = if IS_CLIENT { dc.client_max_window_bits } else { dc.server_max_window_bits };

    Ok(Some(NegotiatedFlate2 {
      decompress: Decompress::new_with_window_bits(false, decoder_wb.into()),
      compress: Compress::new_with_window_bits(
        dc.compression_level.into(),
        false,
        encoder_wb.into(),
      ),
      dc,
    }))
  }

  #[inline]
  fn write_req_headers(&self, sw: &mut SuffixWriterFbvm<'_>) -> crate::Result<()> {
    write_headers(&self.dc, sw)
  }
}

impl Default for Flate2 {
  #[inline]
  fn default() -> Self {
    Flate2::from(DeflateConfig::default())
  }
}

/// Negotiated Flate2 compression
#[derive(Debug)]
pub struct NegotiatedFlate2 {
  compress: Compress,
  dc: DeflateConfig,
  decompress: Decompress,
}

impl NegotiatedCompression for NegotiatedFlate2 {
  #[inline]
  fn compress<O>(
    &mut self,
    input: &[u8],
    output: &mut O,
    begin_cb: impl FnMut(&mut O) -> crate::Result<&mut [u8]>,
    mut rem_cb: impl FnMut(&mut O, usize) -> crate::Result<&mut [u8]>,
  ) -> crate::Result<usize> {
    compress_or_decompress(
      input,
      self,
      output,
      true,
      begin_cb,
      |this, local_input, output_butes| {
        let _ = this.compress.compress(local_input, output_butes, FlushCompress::Sync);
        Ok(())
      },
      |a, b| rem_cb(a, b),
      |this| this.compress.reset(),
      |this| this.compress.total_in(),
      |this| this.compress.total_out(),
    )
  }

  #[inline]
  fn decompress<O>(
    &mut self,
    input: &[u8],
    output: &mut O,
    begin_cb: impl FnMut(&mut O) -> crate::Result<&mut [u8]>,
    rem_cb: impl FnMut(&mut O, usize) -> crate::Result<&mut [u8]>,
  ) -> crate::Result<usize> {
    compress_or_decompress(
      input,
      self,
      output,
      true,
      begin_cb,
      |this, local_input, output_bytes| {
        let _ = this.decompress.decompress(local_input, output_bytes, FlushDecompress::Sync);
        Ok(())
      },
      rem_cb,
      |this| this.decompress.reset(false),
      |this| this.decompress.total_in(),
      |this| this.decompress.total_out(),
    )
  }

  #[inline]
  fn rsv1(&self) -> u8 {
    0b0100_0000
  }

  #[inline]
  fn write_res_headers(&self, sw: &mut SuffixWriterFbvm<'_>) -> crate::Result<()> {
    write_headers(&self.dc, sw)
  }
}

#[inline]
fn byte_from_bytes(bytes: &[u8]) -> Option<u8> {
  let after_equals = bytes_split1(bytes, b'=').nth(1)?;
  u8::from_radix_10(after_equals).ok()
}

#[inline]
fn compress_or_decompress<NC, O>(
  input: &[u8],
  nc: &mut NC,
  output: &mut O,
  reset: bool,
  mut begin_output_cb: impl FnMut(&mut O) -> crate::Result<&mut [u8]>,
  mut call_cb: impl FnMut(&mut NC, &[u8], &mut [u8]) -> crate::Result<()>,
  mut expand_output_cb: impl FnMut(&mut O, usize) -> crate::Result<&mut [u8]>,
  mut reset_cb: impl FnMut(&mut NC),
  mut total_in_cb: impl FnMut(&mut NC) -> u64,
  mut total_out_cb: impl FnMut(&mut NC) -> u64,
) -> crate::Result<usize> {
  let initial_slice = begin_output_cb(output)?;
  call_cb(nc, input, initial_slice)?;
  let mut total_in_sum = usize::try_from(total_in_cb(nc))?;
  let mut total_out_sum = usize::try_from(total_out_cb(nc))?;
  if total_in_sum == input.len() {
    if reset {
      reset_cb(nc);
    }
    return Ok(total_out_sum);
  }
  let mut prev_total_in_sum = total_in_sum;
  loop {
    let Some(slice) = input.get(total_in_sum..) else {
      return Err(crate::Error::UnexpectedBufferState);
    };
    let curr_slice = expand_output_cb(output, total_out_sum)?;
    call_cb(nc, slice, curr_slice)?;
    total_in_sum = usize::try_from(total_in_cb(nc))?;
    if prev_total_in_sum == total_in_sum {
      return Err(crate::Error::UnexpectedBufferState);
    }
    total_out_sum = usize::try_from(total_out_cb(nc))?;
    if total_in_sum == input.len() {
      if reset {
        reset_cb(nc);
      }
      return Ok(total_out_sum);
    }
    prev_total_in_sum = total_in_sum;
  }
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
fn write_headers(dc: &DeflateConfig, sw: &mut SuffixWriterFbvm<'_>) -> crate::Result<()> {
  sw._extend_from_slices_group_rn(&[
    b"Sec-Websocket-Extensions: ",
    b"permessage-deflate; ",
    b"client_max_window_bits=",
    dc.client_max_window_bits.strings().number.as_bytes(),
    b"; ",
    b"server_max_window_bits=",
    dc.server_max_window_bits.strings().number.as_bytes(),
    b"; client_no_context_takeover; server_no_context_takeover",
  ])
}
