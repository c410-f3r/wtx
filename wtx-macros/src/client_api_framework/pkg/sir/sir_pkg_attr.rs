use crate::client_api_framework::{
  pkg::{data_format::DataFormat, fir::fir_pkg_attr::FirPkgAttr},
  transport_group::TransportGroup,
};
use syn::Path;

#[derive(Debug)]
pub(crate) struct SirPkaAttr {
  pub(crate) data_formats: Vec<DataFormat>,
  pub(crate) id: Path,
  pub(crate) transport_groups: Vec<TransportGroup>,
}

impl TryFrom<FirPkgAttr> for SirPkaAttr {
  type Error = crate::Error;

  #[inline]
  fn try_from(fea: FirPkgAttr) -> Result<Self, Self::Error> {
    let data_formats =
      fea.data_formats.iter().map(TryInto::try_into).collect::<crate::Result<Vec<DataFormat>>>()?;
    if data_formats.is_empty() {
      return Err(crate::Error::MandatoryOuterAttrsAreNotPresent);
    }
    let transport_groups =
      fea.transports.iter().map(TryInto::try_into).collect::<crate::Result<_>>()?;
    Ok(Self { data_formats, id: fea.id, transport_groups })
  }
}
