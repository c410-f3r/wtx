//! <https://datatracker.ietf.org/doc/html/rfc7692>

mod compression_level;
mod deflate_config;
#[cfg(feature = "flate2")]
mod flate2;
mod window_bits;

#[cfg(feature = "flate2")]
pub use self::flate2::{Flate2, NegotiatedFlate2};
use crate::{http::GenericHeader, misc::FilledBufferWriter};
pub use compression_level::CompressionLevel;
pub use deflate_config::DeflateConfig;
pub use window_bits::WindowBits;

/// Initial compression parameters defined before a handshake.
pub trait Compression<const IS_CLIENT: bool> {
  /// See [`NegotiatedCompression`].
  type NegotiatedCompression: NegotiatedCompression;

  /// Manages the defined parameters with the received parameters to decide which
  /// parameters will be settled.
  fn negotiate(
    self,
    headers: impl Iterator<Item = impl GenericHeader>,
  ) -> crate::Result<Self::NegotiatedCompression>;

  /// Writes headers bytes that will be sent to the server.
  fn write_req_headers(&self, fbw: &mut FilledBufferWriter<'_>);
}

impl<const IS_CLIENT: bool> Compression<IS_CLIENT> for () {
  type NegotiatedCompression = ();

  #[inline]
  fn negotiate(
    self,
    _: impl Iterator<Item = impl GenericHeader>,
  ) -> crate::Result<Self::NegotiatedCompression> {
    Ok(())
  }

  #[inline]
  fn write_req_headers(&self, _: &mut FilledBufferWriter<'_>) {}
}

/// Final compression parameters defined after a handshake.
pub trait NegotiatedCompression {
  /// Compress
  fn compress<O>(
    &mut self,
    input: &[u8],
    output: &mut O,
    begin_cb: impl FnMut(&mut O) -> &mut [u8],
    rem_cb: impl FnMut(&mut O, usize) -> &mut [u8],
  ) -> crate::Result<usize>;

  /// Decompress
  fn decompress<O>(
    &mut self,
    input: &[u8],
    output: &mut O,
    begin_cb: impl FnMut(&mut O) -> &mut [u8],
    rem_cb: impl FnMut(&mut O, usize) -> crate::Result<&mut [u8]>,
  ) -> crate::Result<usize>;

  /// Rsv1 bit
  fn rsv1(&self) -> u8;

  /// Write response headers
  fn write_res_headers(&self, fbw: &mut FilledBufferWriter<'_>);
}

impl<T> NegotiatedCompression for &mut T
where
  T: NegotiatedCompression,
{
  #[inline]
  fn compress<O>(
    &mut self,
    input: &[u8],
    output: &mut O,
    begin_cb: impl FnMut(&mut O) -> &mut [u8],
    rem_cb: impl FnMut(&mut O, usize) -> &mut [u8],
  ) -> crate::Result<usize> {
    (**self).compress(input, output, begin_cb, rem_cb)
  }

  #[inline]
  fn decompress<O>(
    &mut self,
    input: &[u8],
    output: &mut O,
    begin_cb: impl FnMut(&mut O) -> &mut [u8],
    rem_cb: impl FnMut(&mut O, usize) -> crate::Result<&mut [u8]>,
  ) -> crate::Result<usize> {
    (**self).decompress(input, output, begin_cb, rem_cb)
  }

  #[inline]
  fn rsv1(&self) -> u8 {
    (**self).rsv1()
  }

  #[inline]
  fn write_res_headers(&self, fbw: &mut FilledBufferWriter<'_>) {
    (**self).write_res_headers(fbw);
  }
}

impl NegotiatedCompression for () {
  #[inline]
  fn compress<O>(
    &mut self,
    _: &[u8],
    _: &mut O,
    _: impl FnMut(&mut O) -> &mut [u8],
    _: impl FnMut(&mut O, usize) -> &mut [u8],
  ) -> crate::Result<usize> {
    Ok(0)
  }

  #[inline]
  fn decompress<O>(
    &mut self,
    _: &[u8],
    _: &mut O,
    _: impl FnMut(&mut O) -> &mut [u8],
    _: impl FnMut(&mut O, usize) -> crate::Result<&mut [u8]>,
  ) -> crate::Result<usize> {
    Ok(0)
  }

  #[inline]
  fn rsv1(&self) -> u8 {
    0
  }

  #[inline]
  fn write_res_headers(&self, _: &mut FilledBufferWriter<'_>) {}
}

impl<T> NegotiatedCompression for Option<T>
where
  T: NegotiatedCompression,
{
  #[inline]
  fn compress<O>(
    &mut self,
    input: &[u8],
    output: &mut O,
    begin_cb: impl FnMut(&mut O) -> &mut [u8],
    rem_cb: impl FnMut(&mut O, usize) -> &mut [u8],
  ) -> crate::Result<usize> {
    match self {
      Some(el) => el.compress(input, output, begin_cb, rem_cb),
      None => ().compress(input, output, begin_cb, rem_cb),
    }
  }

  #[inline]
  fn decompress<O>(
    &mut self,
    input: &[u8],
    output: &mut O,
    begin_cb: impl FnMut(&mut O) -> &mut [u8],
    rem_cb: impl FnMut(&mut O, usize) -> crate::Result<&mut [u8]>,
  ) -> crate::Result<usize> {
    match self {
      Some(el) => el.decompress(input, output, begin_cb, rem_cb),
      None => ().decompress(input, output, begin_cb, rem_cb),
    }
  }

  #[inline]
  fn rsv1(&self) -> u8 {
    match self {
      Some(el) => el.rsv1(),
      None => ().rsv1(),
    }
  }

  #[inline]
  fn write_res_headers(&self, fbw: &mut FilledBufferWriter<'_>) {
    match self {
      Some(el) => el.write_res_headers(fbw),
      None => ().write_res_headers(fbw),
    }
  }
}
