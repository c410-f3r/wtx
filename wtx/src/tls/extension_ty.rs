use crate::{
  de::{Decode, Encode},
  misc::SuffixWriterMut,
  tls::de::De,
};

create_enum! {
  #[derive(Clone, Copy, Debug, Eq, PartialEq)]
  pub enum ExtensionTy<u16> {
    ServerName = (0),
    MaxFragmentLength = (1),
    StatusRequest = (5),
    SupportedGroups = (10),
    SignatureAlgorithms = (13),
    UseSrtp = (14),
    Heartbeat = (15),
    ApplicationLayerProtocolNegotiation = (16),
    SignedCertificateTimestamp = (18),
    ClientCertificateType = (19),
    ServerCertificateType = (20),
    Padding = (21),
    PreSharedKey = (41),
    EarlyData = (42),
    SupportedVersions = (43),
    Cookie = (44),
    PskKeyExchangeModes = (45),
    CertificateAuthorities = (47),
    OidFilters = (48),
    PostHandshakeAuth = (49),
    SignatureAlgorithmsCert = (50),
    KeyShare = (51),
  }
}

impl<'de> Decode<'de, De> for ExtensionTy {
  #[inline]
  fn decode(aux: &mut (), dw: &mut &'de [u8]) -> crate::Result<Self> {
    let tag: u16 = Decode::<'_, De>::decode(aux, dw)?;
    Self::try_from(tag)
  }
}

impl Encode<De> for ExtensionTy {
  #[inline]
  fn encode(&self, aux: &mut (), sw: &mut SuffixWriterMut<'_>) -> crate::Result<()> {
    <u16 as Encode<De>>::encode(&u16::from(*self), aux, sw)
  }
}
