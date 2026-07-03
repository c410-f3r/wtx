// https://datatracker.ietf.org/doc/html/rfc8446#section-4.4.4

use crate::{
  codec::{Decode, Encode},
  collections::ArrayVectorCopy,
  crypto::MAX_HASH_LEN,
  misc::TryArithmetic as _,
  tls::{
    TlsError,
    de::De,
    key_schedule::KeyScheduleState,
    protocol::{handshake::HandshakeType, record_content_type::RecordContentType},
    tls_decode_wrapper::TlsDecodeWrapper,
    tls_encode_wrapper::TlsEncodeWrapper,
  },
};

#[derive(Debug)]
pub(crate) struct Finished<'any> {
  verify_data: &'any [u8],
}

impl<'any> Finished<'any> {
  pub(crate) fn new(verify_data: &'any [u8]) -> Self {
    Self { verify_data }
  }

  pub(crate) const fn verify_data(&self) -> &'any [u8] {
    self.verify_data
  }

  pub(crate) fn record_bytes(
    data_bytes: &[u8],
    kss: &mut KeyScheduleState,
  ) -> crate::Result<ArrayVectorCopy<u8, { 5 + 4 + MAX_HASH_LEN + 1 + 16 }>> {
    let verify_len: u8 = data_bytes.len().try_into()?;
    let payload_len: u8 = verify_len.try_add(21)?;
    let header = [RecordContentType::ApplicationData.into(), 3, 3, 0, payload_len];
    let mut encrypted = ArrayVectorCopy::<u8, { 4 + MAX_HASH_LEN + 1 }>::new();
    let _ = encrypted.extend_from_copyable_slices([
      &[HandshakeType::Finished.into(), 0, 0, verify_len],
      data_bytes,
      &[RecordContentType::Handshake.into()],
    ])?;
    let nonce = kss.nonce();
    let secret = kss.cipher_key();
    let tag = kss.cipher_suite().aes_encrypt(&header, &mut encrypted, nonce, secret)?;
    let mut rslt = ArrayVectorCopy::new();
    let _ = rslt.extend_from_copyable_slices([header.as_slice(), &encrypted, &tag])?;
    kss.increment_counter();
    Ok(rslt)
  }
}

impl<'de> Decode<'de, De> for Finished<'de> {
  #[inline]
  fn decode(dw: &mut TlsDecodeWrapper<'de>) -> crate::Result<Self> {
    let Some((before, after)) = dw.bytes().split_at_checked(dw.cipher_suite().hash_len().into())
    else {
      return Err(TlsError::InvalidFinishedRecord.into());
    };
    *dw.bytes_mut() = after;
    Ok(Finished { verify_data: before })
  }
}

impl Encode<De> for Finished<'_> {
  #[inline]
  fn encode(&self, ew: &mut TlsEncodeWrapper<'_>) -> crate::Result<()> {
    <[u8] as Encode<De>>::encode(self.verify_data, ew)
  }
}
