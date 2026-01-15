use crate::{
  de::{Decode, Encode},
  misc::SuffixWriterMut,
  tls::de::De,
};

create_enum! {
  #[derive(Clone, Copy, Debug, Eq, PartialEq)]
  pub enum ServerHelloExtensionTy<u16> {
    PreSharedKey = (41),
    SupportedVersions = (43),
    KeyShare = (51),
  }
}

impl<'de> Decode<'de, De> for ServerHelloExtensionTy {
  #[inline]
  fn decode(dw: &mut &'de [u8]) -> crate::Result<Self> {
    let tag: u16 = Decode::<'_, De>::decode(dw)?;
    Self::try_from(tag)
  }
}

impl Encode<De> for ServerHelloExtensionTy {
  #[inline]
  fn encode(&self, sw: &mut SuffixWriterMut<'_>) -> crate::Result<()> {
    <u16 as Encode<De>>::encode(&u16::from(*self), sw)
  }
}
