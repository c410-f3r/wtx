// https://datatracker.ietf.org/doc/html/rfc7301

use crate::{
  collection::ArrayVectorU8,
  codec::{Decode, Encode},
  misc::counter_writer::{CounterWriterBytesTy, CounterWriterIterTy, u16_write, u16_write_iter},
  tls::{
    MAX_ALPN_LEN, TlsError, de::De, decode_wrapper::DecodeWrapper, encode_wrapper::EncodeWrapper,
    misc::u16_chunk,
  },
};

#[derive(Debug, Default)]
pub struct Alpn<'any> {
  pub protocol_name_list: ArrayVectorU8<&'any [u8], MAX_ALPN_LEN>,
}

impl<'de> Decode<'de, De> for Alpn<'de> {
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'de>) -> crate::Result<Self> {
    let mut protocol_name_list = ArrayVectorU8::new();
    u16_chunk(dw, TlsError::InvalidOfferedPsks, |local_dw| {
      while !local_dw.bytes().is_empty() {
        protocol_name_list.push(local_dw.bytes())?;
      }
      Ok(())
    })?;
    Ok(Self { protocol_name_list })
  }
}

impl Encode<De> for Alpn<'_> {
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_>) -> crate::Result<()> {
    u16_write_iter(
      CounterWriterIterTy::Bytes(CounterWriterBytesTy::IgnoresLen),
      self.protocol_name_list.iter(),
      None,
      ew,
      |elem, local_ew| {
        u16_write(CounterWriterBytesTy::IgnoresLen, None, local_ew, |local_local_sw| {
          local_local_sw.buffer().extend_from_slice(elem)?;
          crate::Result::Ok(())
        })
      },
    )?;
    Ok(())
  }
}
