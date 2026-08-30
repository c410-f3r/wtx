use crate::{
  codec::{Decode, Encode},
  misc::counter_writer::{CounterWriterBytesTy, u16_write},
  tls::{
    TlsError,
    misc::u16_chunk,
    protocol::extension_ty::ExtensionTy,
    tls_cc::TlsCc,
    tls_cc_wrappers::{TlsDecodeWrapper, TlsEncodeWrapper},
  },
};

pub(crate) struct Extension<T> {
  data: T,
  extension_ty: ExtensionTy,
}

impl<T> Extension<T> {
  pub(crate) const fn new(extension_ty: ExtensionTy, data: T) -> Self {
    Self { data, extension_ty }
  }
}

impl<'de, T> Decode<'de, TlsCc> for Extension<T>
where
  T: Decode<'de, TlsCc>,
{
  #[inline]
  fn decode(dw: &mut TlsDecodeWrapper<'de>) -> crate::Result<Self> {
    let extension_ty = ExtensionTy::decode(dw)?;
    let data = u16_chunk(dw, TlsError::InvalidExtension, |local_dw| T::decode(local_dw))?;
    Ok(Self { data, extension_ty })
  }
}

impl<T> Encode<TlsCc> for Extension<T>
where
  T: Encode<TlsCc>,
{
  #[inline]
  fn encode(&self, ew: &mut TlsEncodeWrapper<'_>) -> crate::Result<()> {
    self.extension_ty.encode(ew)?;
    u16_write(CounterWriterBytesTy::IgnoresLen, None, ew, |local_ew| {
      self.data.encode(local_ew)?;
      Ok(())
    })
  }
}
