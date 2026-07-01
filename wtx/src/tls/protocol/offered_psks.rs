use crate::{
  codec::{Decode, Encode},
  collections::ArrayVectorU8,
  misc::{
    Lease,
    counter_writer::{CounterWriterBytesTy, CounterWriterIterTy, u16_write, u16_write_iter},
  },
  tls::{
    TlsError, de::De, misc::u16_chunk, tls_decode_wrapper::TlsDecodeWrapper,
    tls_encode_wrapper::TlsEncodeWrapper,
  },
};

#[derive(Clone, Debug)]
pub(crate) struct OfferedPsks<B> {
  pub(crate) offered_psks: ArrayVectorU8<OfferedPsk<B>, 4>,
}

impl<'de, B> Decode<'de, De> for OfferedPsks<B>
where
  B: Lease<[u8]> + TryFrom<&'de [u8]>,
  B::Error: Into<crate::Error>,
{
  #[inline]
  fn decode(dw: &mut TlsDecodeWrapper<'de>) -> crate::Result<Self> {
    let mut offered_psks = ArrayVectorU8::new();
    u16_chunk(dw, TlsError::InvalidOfferedPsks, |local_dw| {
      while !local_dw.bytes().is_empty() {
        offered_psks.push(OfferedPsk {
          identity: PskIdentity {
            identity: B::try_from(&[]).map_err(Into::into)?,
            obfuscated_ticket_age: 0,
          },
          binder: B::try_from(&[]).map_err(Into::into)?,
        })?;
      }
      Ok(())
    })?;
    Ok(Self { offered_psks })
  }
}

impl<B> Encode<De> for OfferedPsks<B>
where
  B: Lease<[u8]>,
{
  #[inline]
  fn encode(&self, ew: &mut TlsEncodeWrapper<'_>) -> crate::Result<()> {
    u16_write_iter(
      CounterWriterIterTy::Bytes(CounterWriterBytesTy::IgnoresLen),
      self.offered_psks.iter().map(|el| &el.identity),
      None,
      ew,
      |elem, local_ew| {
        u16_write(CounterWriterBytesTy::IgnoresLen, None, local_ew, |local_local_sw| {
          let _ = local_local_sw.buffer().extend_from_copyable_slices([
            elem.identity.lease(),
            &elem.obfuscated_ticket_age.to_be_bytes(),
          ])?;
          crate::Result::Ok(())
        })
      },
    )?;
    u16_write_iter(
      CounterWriterIterTy::Bytes(CounterWriterBytesTy::IgnoresLen),
      self.offered_psks.iter().map(|el| el.binder.lease()),
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

#[derive(Clone, Debug)]
pub(crate) struct OfferedPsk<B> {
  pub(crate) identity: PskIdentity<B>,
  pub(crate) binder: B,
}

#[derive(Clone, Debug)]
pub(crate) struct PskIdentity<B> {
  pub(crate) identity: B,
  pub(crate) obfuscated_ticket_age: u32,
}
