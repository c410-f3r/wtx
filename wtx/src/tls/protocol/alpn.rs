// https://datatracker.ietf.org/doc/html/rfc7301

use crate::{
  codec::{Decode, Encode},
  collections::ArrayVectorCopy,
  misc::counter_writer::{CounterWriterBytesTy, CounterWriterIterTy, u16_write, u16_write_iter},
  tls::{
    MAX_ALPN_LEN, TlsError, de::De, misc::u16_chunk, tls_decode_wrapper::TlsDecodeWrapper,
    tls_encode_wrapper::TlsEncodeWrapper,
  },
};

/// Application-Layer Protocol Negotiation Extension
///
/// <https://datatracker.ietf.org/doc/html/rfc7301>
#[derive(Clone, Debug, Default)]
pub struct Alpn {
  /// List of names
  pub protocol_name_list: ArrayVectorCopy<ArrayVectorCopy<u8, 8>, MAX_ALPN_LEN>,
}

impl<'de> Decode<'de, De> for Alpn {
  #[inline]
  fn decode(dw: &mut TlsDecodeWrapper<'de>) -> crate::Result<Self> {
    let mut protocol_name_list = ArrayVectorCopy::new();
    u16_chunk(dw, TlsError::InvalidOfferedPsks, |local_dw| {
      while !local_dw.bytes().is_empty() {
        protocol_name_list.push(local_dw.bytes().try_into()?)?;
      }
      Ok(())
    })?;
    Ok(Self { protocol_name_list })
  }
}

impl Encode<De> for Alpn {
  #[inline]
  fn encode(&self, ew: &mut TlsEncodeWrapper<'_>) -> crate::Result<()> {
    u16_write_iter(
      CounterWriterIterTy::Bytes(CounterWriterBytesTy::IgnoresLen),
      self.protocol_name_list.iter(),
      None,
      ew,
      |elem, local_ew| {
        u16_write(CounterWriterBytesTy::IgnoresLen, None, local_ew, |local_local_ew| {
          local_local_ew.buffer().extend_from_copyable_slice(elem)?;
          crate::Result::Ok(())
        })
      },
    )?;
    Ok(())
  }
}
