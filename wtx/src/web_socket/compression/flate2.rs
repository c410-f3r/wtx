use crate::{
  http::Http1Header,
  misc::from_utf8_opt,
  web_socket::{compression::NegotiatedCompression, Compression, DeflateConfig},
};
use core::str::FromStr;
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
    headers: impl Iterator<Item = impl Http1Header>,
  ) -> crate::Result<Self::NegotiatedCompression> {
    use crate::{misc::_trim, web_socket::WebSocketError};

    let mut dc = DeflateConfig {
      client_max_window_bits: self.dc.client_max_window_bits,
      compression_level: self.dc.compression_level,
      server_max_window_bits: self.dc.server_max_window_bits,
    };

    let mut has_extension = false;

    for swe in headers.filter(|el| el.name().eq_ignore_ascii_case(b"sec-websocket-extensions")) {
      for permessage_deflate_option in swe.value().split(|el| el == &b',') {
        dc = DeflateConfig {
          client_max_window_bits: self.dc.client_max_window_bits,
          compression_level: self.dc.compression_level,
          server_max_window_bits: self.dc.server_max_window_bits,
        };
        let mut client_max_window_bits_flag = false;
        let mut permessage_deflate_flag = false;
        let mut server_max_window_bits_flag = false;
        for param in permessage_deflate_option.split(|el| el == &b';').map(|elem| _trim(elem)) {
          if param == b"client_no_context_takeover" || param == b"server_no_context_takeover" {
          } else if param == b"permessage-deflate" {
            _manage_header_uniqueness(&mut permessage_deflate_flag, || Ok(()))?
          } else if let Some(after_cmwb) = param.strip_prefix(b"client_max_window_bits") {
            _manage_header_uniqueness(&mut client_max_window_bits_flag, || {
              if let Some(value) = _value_from_bytes::<u8>(after_cmwb) {
                dc.client_max_window_bits = value.try_into()?;
              }
              Ok(())
            })?;
          } else if let Some(after_smwb) = param.strip_prefix(b"server_max_window_bits") {
            _manage_header_uniqueness(&mut server_max_window_bits_flag, || {
              if let Some(value) = _value_from_bytes::<u8>(after_smwb) {
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
  fn write_req_headers<B>(&self, buffer: &mut B)
  where
    B: Extend<u8>,
  {
    write_headers(buffer, &self.dc)
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
  fn compress<O>(
    &mut self,
    input: &[u8],
    output: &mut O,
    begin_cb: impl FnMut(&mut O) -> &mut [u8],
    rem_cb: impl FnMut(&mut O, usize) -> &mut [u8],
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
      rem_cb,
      |this| this.compress.reset(),
      |this| this.compress.total_in(),
      |this| this.compress.total_out(),
    )
  }

  fn decompress<O>(
    &mut self,
    input: &[u8],
    output: &mut O,
    begin_cb: impl FnMut(&mut O) -> &mut [u8],
    rem_cb: impl FnMut(&mut O, usize) -> &mut [u8],
  ) -> crate::Result<usize> {
    compress_or_decompress(
      input,
      self,
      output,
      true,
      begin_cb,
      |this, local_input, output_butes| {
        let _ = this.decompress.decompress(local_input, output_butes, FlushDecompress::Sync);
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
  fn write_res_headers<B>(&self, buffer: &mut B)
  where
    B: Extend<u8>,
  {
    write_headers(buffer, &self.dc)
  }
}

fn compress_or_decompress<NC, O>(
  input: &[u8],
  nc: &mut NC,
  output: &mut O,
  reset: bool,
  mut begin_output_cb: impl FnMut(&mut O) -> &mut [u8],
  mut call_cb: impl FnMut(&mut NC, &[u8], &mut [u8]) -> crate::Result<()>,
  mut expand_output_cb: impl FnMut(&mut O, usize) -> &mut [u8],
  mut reset_cb: impl FnMut(&mut NC),
  mut total_in_cb: impl FnMut(&mut NC) -> u64,
  mut total_out_cb: impl FnMut(&mut NC) -> u64,
) -> crate::Result<usize> {
  call_cb(nc, input, begin_output_cb(output))?;
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
    call_cb(nc, slice, expand_output_cb(output, total_out_sum))?;
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

fn _manage_header_uniqueness(
  flag: &mut bool,
  mut cb: impl FnMut() -> crate::Result<()>,
) -> crate::Result<()> {
  if *flag {
    Err(crate::Error::DuplicatedHeader)
  } else {
    cb()?;
    *flag = true;
    Ok(())
  }
}

fn _value_from_bytes<T>(bytes: &[u8]) -> Option<T>
where
  T: FromStr,
{
  let after_equals = bytes.split(|byte| byte == &b'=').nth(1)?;
  from_utf8_opt(after_equals)?.parse::<T>().ok()
}

#[inline]
fn write_headers<B>(buffer: &mut B, dc: &DeflateConfig)
where
  B: Extend<u8>,
{
  buffer.extend(*b"Sec-Websocket-Extensions: ");

  buffer.extend(*b"permessage-deflate; ");

  buffer.extend(*b"client_max_window_bits=");
  buffer.extend(<&str>::from(dc.client_max_window_bits).as_bytes().iter().copied());
  buffer.extend(*b"; ");

  buffer.extend(*b"server_max_window_bits=");
  buffer.extend(<&str>::from(dc.server_max_window_bits).as_bytes().iter().copied());

  buffer.extend(*b"; client_no_context_takeover; server_no_context_takeover\r\n");
}
