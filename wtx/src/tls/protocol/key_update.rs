use crate::{
  codec::{Decode, Encode},
  crypto::AEAD_TAG_LEN,
  tls::{
    RECORD_HEADER_LEN, TlsError,
    de::De,
    key_schedule::KeyScheduleState,
    protocol::{handshake_ty::HandshakeTy, record_content_ty::RecordContentTy},
    tls_decode_wrapper::TlsDecodeWrapper,
    tls_encode_wrapper::TlsEncodeWrapper,
  },
};

/// The `KeyUpdate` handshake message is used to indicate that the sender is updating its
/// sending cryptographic keys.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum KeyUpdateRequest {
  UpdateNotRequested = 0,
  UpdateRequested = 1,
}

impl From<KeyUpdateRequest> for u8 {
  #[inline]
  fn from(value: KeyUpdateRequest) -> Self {
    match value {
      KeyUpdateRequest::UpdateNotRequested => 0,
      KeyUpdateRequest::UpdateRequested => 1,
    }
  }
}

impl TryFrom<u8> for KeyUpdateRequest {
  type Error = crate::Error;

  #[inline]
  fn try_from(value: u8) -> crate::Result<Self> {
    Ok(match value {
      0 => KeyUpdateRequest::UpdateNotRequested,
      1 => KeyUpdateRequest::UpdateRequested,
      _ => return Err(TlsError::UnknownKeyUpdateRequest.into()),
    })
  }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct KeyUpdate {
  pub(crate) request_update: KeyUpdateRequest,
}

impl KeyUpdate {
  pub(crate) fn new(request_update: KeyUpdateRequest) -> Self {
    Self { request_update }
  }

  pub(crate) fn record_bytes(
    self,
    kss: &mut KeyScheduleState,
  ) -> crate::Result<[u8; RECORD_HEADER_LEN + 5 + 1 + 16]> {
    let [a0] = self.data_bytes();
    let header =
      [RecordContentTy::ApplicationData.into(), 3, 3, 0, RecordContentTy::Handshake.into()];
    let mut encrypted =
      [HandshakeTy::KeyUpdate.into(), 0, 0, 1, a0, RecordContentTy::Handshake.into()];
    let nonce = kss.nonce();
    let secret = kss.cipher_key();
    let tag = kss.cipher_suite().aes_encrypt(&header, &mut encrypted, nonce, secret)?;
    let [b0, b1, b2, b3, b4] = header;
    let [b5, b6, b7, b8, b9, b10] = encrypted;
    let mut rslt =
      [b0, b1, b2, b3, b4, b5, b6, b7, b8, b9, b10, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    if let Some(elem) = rslt.last_chunk_mut::<AEAD_TAG_LEN>() {
      elem.copy_from_slice(&tag);
    }
    kss.increment_counter();
    Ok(rslt)
  }

  fn data_bytes(self) -> [u8; 1] {
    [u8::from(self.request_update)]
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
    ew.buffer().push(u8::from(self.request_update))
  }
}
