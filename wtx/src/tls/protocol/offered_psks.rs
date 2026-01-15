use crate::{
  collection::ArrayVectorU8,
  de::{Decode, Encode},
  misc::{
    SuffixWriterMut,
    counter_writer::{CounterWriterBytesTy, CounterWriterIterTy, u16_write, u16_write_iter},
  },
  tls::{TlsError, de::De, misc::u16_chunk},
};

#[derive(Clone, Debug)]
pub(crate) struct OfferedPsks<'any> {
  pub(crate) offered_psks: ArrayVectorU8<OfferedPsk<'any>, 4>,
}

impl<'de> Decode<'de, De> for OfferedPsks<'de> {
  #[inline]
  fn decode(dw: &mut &'de [u8]) -> crate::Result<Self> {
    let mut offered_psks = ArrayVectorU8::new();
    u16_chunk(dw, TlsError::InvalidOfferedPsks, |bytes| {
      while !bytes.is_empty() {
        offered_psks.push(OfferedPsk {
          identity: PskIdentity { identity: &[], obfuscated_ticket_age: 0 },
          binder: &[],
        })?;
      }
      Ok(())
    })?;
    Ok(Self { offered_psks })
  }
}

impl Encode<De> for OfferedPsks<'_> {
  #[inline]
  fn encode(&self, sw: &mut SuffixWriterMut<'_>) -> crate::Result<()> {
    u16_write_iter(
      CounterWriterIterTy::Bytes(CounterWriterBytesTy::IgnoresLen),
      self.offered_psks.iter().map(|el| &el.identity),
      None,
      sw,
      |elem, local_sw| {
        u16_write(CounterWriterBytesTy::IgnoresLen, None, local_sw, |local_local_sw| {
          local_local_sw
            .extend_from_slices([elem.identity, &elem.obfuscated_ticket_age.to_be_bytes()])?;
          crate::Result::Ok(())
        })
      },
    )?;
    u16_write_iter(
      CounterWriterIterTy::Bytes(CounterWriterBytesTy::IgnoresLen),
      self.offered_psks.iter().map(|el| el.binder),
      None,
      sw,
      |elem, local_sw| {
        u16_write(CounterWriterBytesTy::IgnoresLen, None, local_sw, |local_local_sw| {
          local_local_sw.extend_from_slice(elem)?;
          crate::Result::Ok(())
        })
      },
    )?;
    Ok(())
  }
}

#[derive(Clone, Debug)]
pub(crate) struct OfferedPsk<'any> {
  pub(crate) identity: PskIdentity<'any>,
  pub(crate) binder: &'any [u8],
}

#[derive(Clone, Debug)]
pub(crate) struct PskIdentity<'any> {
  pub(crate) identity: &'any [u8],
  pub(crate) obfuscated_ticket_age: u32,
}
