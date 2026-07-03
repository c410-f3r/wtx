// https://datatracker.ietf.org/doc/html/rfc8446#section-4

use crate::{
  codec::{Decode, Encode},
  misc::counter_writer::{CounterWriterBytesTy, u24_write},
  tls::{
    TlsError, de::De, misc::u24_chunk, tls_decode_wrapper::TlsDecodeWrapper,
    tls_encode_wrapper::TlsEncodeWrapper,
  },
};

create_enum! {
  /// <https://datatracker.ietf.org/doc/html/rfc8446#appendix-B.3>
  #[derive(Copy, Clone, Debug, PartialEq)]
  pub(crate) enum HandshakeType<u8> {
    ClientHello = (1),
    ServerHello = (2),
    NewSessionTicket = (4),
    EndOfEarlyData = (5),
    EncryptedExtensions = (8),
    Certificate = (11),
    CertificateRequest = (13),
    CertificateVerify = (15),
    Finished = (20),
    KeyUpdate = (24),
    MessageHash = (254),
  }
}

pub(crate) struct Handshake<T> {
  pub(crate) msg_type: HandshakeType,
  pub(crate) data: T,
}

impl<T> Handshake<T> {
  #[inline]
  pub(crate) const fn new(msg_type: HandshakeType, data: T) -> Self {
    Self { msg_type, data }
  }
}

impl<'de, T> Decode<'de, De> for Handshake<T>
where
  T: Decode<'de, De>,
{
  #[inline]
  fn decode(dw: &mut TlsDecodeWrapper<'de>) -> crate::Result<Self> {
    let msg_type = HandshakeType::try_from(<u8 as Decode<De>>::decode(dw)?)?;
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
