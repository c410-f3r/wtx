// https://datatracker.ietf.org/doc/html/rfc8446#section-4.2.2

use crate::{
  codec::{Decode, Encode},
  misc::Lease,
  tls::{
    TlsError, de::De, misc::u16_chunk, tls_decode_wrapper::TlsDecodeWrapper,
    tls_encode_wrapper::TlsEncodeWrapper,
  },
};

/// Handshake state
#[expect(unused, reason = "future-proof")]
#[derive(Clone, Debug, Default)]
pub(crate) struct Cookie<B> {
  /// Opaque bytes
  pub(crate) cookie: B,
}

impl<'de, B> Decode<'de, De> for Cookie<B>
where
  B: Lease<[u8]> + TryFrom<&'de [u8]>,
  B::Error: Into<crate::Error>,
{
  #[inline]
  fn decode(dw: &mut TlsDecodeWrapper<'de>) -> crate::Result<Self> {
    Ok(Self {
      cookie: u16_chunk(dw, TlsError::InvalidCookie, |local_dw| Ok(local_dw.bytes()))?
        .try_into()
        .map_err(Into::into)?,
    })
  }
}

impl<B> Encode<De> for Cookie<B>
where
  B: Lease<[u8]>,
{
  #[inline]
  fn encode(&self, ew: &mut TlsEncodeWrapper<'_>) -> crate::Result<()> {
    let cookie = self.cookie.lease();
    let array = [&u16::try_from(cookie.len())?.to_be_bytes(), cookie];
    let _ = ew.buffer().extend_from_copyable_slices(array)?;
    Ok(())
  }
}
