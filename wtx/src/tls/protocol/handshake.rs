// https://datatracker.ietf.org/doc/html/rfc9846#section-4

use crate::{
  codec::{Decode, Encode},
  misc::counter_writer::{CounterWriterBytesTy, u24_write},
  tls::{
    TlsError, de::De, misc::u24_chunk, protocol::handshake_ty::HandshakeTy,
    tls_decode_wrapper::TlsDecodeWrapper, tls_encode_wrapper::TlsEncodeWrapper,
  },
};

#[derive(Debug)]
pub(crate) struct Handshake<T> {
  pub(crate) msg_type: HandshakeTy,
  pub(crate) data: T,
}

impl<T> Handshake<T> {
  #[inline]
  pub(crate) const fn new(msg_type: HandshakeTy, data: T) -> Self {
    Self { msg_type, data }
  }
}

impl<'de, T> Decode<'de, De> for Handshake<T>
where
  T: Decode<'de, De>,
{
  #[inline]
  fn decode(dw: &mut TlsDecodeWrapper<'de>) -> crate::Result<Self> {
    let msg_type = HandshakeTy::try_from(<u8 as Decode<De>>::decode(dw)?)?;
    let data = u24_chunk(dw, TlsError::InvalidHandshake, |local_dw| T::decode(local_dw))?;
    Ok(Self { msg_type, data })
  }
}

impl<T> Encode<De> for Handshake<T>
where
  T: Encode<De>,
{
  #[inline]
  fn encode(&self, ew: &mut TlsEncodeWrapper<'_>) -> crate::Result<()> {
    ew.buffer().push(u8::from(self.msg_type))?;
    u24_write(CounterWriterBytesTy::IgnoresLen, None, ew, |local_ew| self.data.encode(local_ew))
  }
}
