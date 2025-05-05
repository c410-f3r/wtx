use crate::client_api_framework::{
  pkg::{data_format::DataFormat, fir::fir_pkg_attr::FirPkgAttr},
  transport_group::TransportGroup,
};
use syn::Path;

#[derive(Debug)]
pub(crate) struct SirPkaAttr<'attrs> {
  pub(crate) data_formats: Vec<DataFormat>,
  pub(crate) id: &'attrs Path,
  pub(crate) transport_groups: Vec<TransportGroup>,
}

impl<'attrs> TryFrom<FirPkgAttr<'attrs>> for SirPkaAttr<'attrs> {
  type Error = crate::Error;

  #[inline]
  fn try_from(fea: FirPkgAttr<'attrs>) -> Result<Self, Self::Error> {
    let data_formats = fea
      .data_formats
      .into_iter()
      .map(TryInto::try_into)
      .collect::<crate::Result<Vec<DataFormat>>>()?;
    if data_formats.is_empty() {
      return Err(crate::Error::MandatoryOuterAttrsAreNotPresent);
    }
    let transport_groups =
      fea.transports.into_iter().map(TryInto::try_into).collect::<crate::Result<_>>()?;
    Ok(Self { data_formats, id: fea.id, transport_groups })
  }
}
