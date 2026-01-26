// https://datatracker.ietf.org/doc/html/rfc8446#section-5.1

use crate::{
  de::Encode,
  misc::{
    SuffixWriterMut,
    counter_writer::{CounterWriterBytesTy, u16_write},
  },
  tls::{
    de::De, encode_wrapper::EncodeWrapper, protocol::{protocol_version::ProtocolVersion, record_content_type::RecordContentType}
  },
};

#[derive(Debug)]
pub(crate) struct Record<T> {
  ty: RecordContentType,
  legacy_record_version: ProtocolVersion,
  opaque: T,
}

impl<T> Record<T> {
  pub(crate) fn new(ty: RecordContentType, opaque: T) -> Self {
    Self { ty, legacy_record_version: ProtocolVersion::Tls12, opaque }
  }
}

impl<T> Encode<De> for Record<T>
where
  T: Encode<De>,
{
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_>) -> crate::Result<()> {
    ew.buffer().extend_from_slices([
      &[u8::from(self.ty)][..],
      &u16::from(self.legacy_record_version).to_be_bytes(),
    ])?;
    u16_write(CounterWriterBytesTy::IgnoresLen, None, ew, |local_ew| self.opaque.encode(local_ew))
  }
}
