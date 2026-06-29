use crate::{
  codec::{Decode, Encode},
  crypto::AEAD_TAG_LEN,
  tls::{
    TlsError, de::De, key_schedule::KeyScheduleState,
    protocol::record_content_type::RecordContentType, tls_decode_wrapper::TlsDecodeWrapper,
    tls_encode_wrapper::TlsEncodeWrapper,
  },
};

create_enum! {
  /// The `KeyUpdate` handshake message is used to indicate that the sender is updating its
  /// sending cryptographic keys.
  #[derive(Clone, Copy, Debug, Eq, PartialEq)]
  pub(crate) enum KeyUpdateRequest<u8> {
    UpdateNotRequested = (0),
    UpdateRequested = (1),
  }
}

pub(crate) struct KeyUpdate {
  pub(crate) request_update: KeyUpdateRequest,
}

impl KeyUpdate {
  pub(crate) fn new(request_update: KeyUpdateRequest) -> Self {
    Self { request_update }
  }

  pub(crate) fn data_bytes(&self) -> [u8; 1] {
    [u8::from(self.request_update)]
  }

  pub(crate) fn record_bytes(
    [a0]: [u8; 1],
    kss: &mut KeyScheduleState,
  ) -> crate::Result<[u8; 5 + 1 + 1 + 16]> {
    let header = [RecordContentType::ApplicationData.into(), 3, 3, 0, 1];
    let mut encrypted = [a0, RecordContentType::ApplicationData.into()];
    let nonce = kss.nonce();
    let secret = kss.cipher_key();
    let tag = kss.cipher_suite().aes_encrypt(&header, &mut encrypted, nonce, secret)?;
    let [b0, b1, b2, b3, b4] = header;
    let [b5, b6] = encrypted;
    let mut rslt = [b0, b1, b2, b3, b4, b5, b6, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    if let Some(elem) = rslt.last_chunk_mut::<AEAD_TAG_LEN>() {
      elem.copy_from_slice(&tag);
    }
    Ok(rslt)
  }
}

impl<'de> Decode<'de, De> for KeyUpdate {
  #[inline]
  fn decode(dw: &mut TlsDecodeWrapper<'de>) -> crate::Result<Self> {
    let [b0, rest @ ..] = dw.bytes() else {
      return Err(TlsError::InvalidMaxFragmentLength.into());
    };
    *dw.bytes_mut() = rest;
    Ok(KeyUpdate { request_update: KeyUpdateRequest::try_from(*b0)? })
  }
}

impl Encode<De> for KeyUpdate {
  #[inline]
  fn encode(&self, ew: &mut TlsEncodeWrapper<'_>) -> crate::Result<()> {
    ew.buffer().inner_mut().push(u8::from(self.request_update))
  }
}
