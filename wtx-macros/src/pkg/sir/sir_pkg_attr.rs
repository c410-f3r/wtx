use crate::{
  pkg::{data_format::DataFormat, fir::fir_pkg_attr::FirPkgAttr},
  transport_group::TransportGroup,
};
use syn::Path;

#[derive(Debug)]
pub(crate) struct SirPkaAttr<'attrs> {
  pub(crate) api: &'attrs Path,
  pub(crate) data_formats: Vec<DataFormat>,
  pub(crate) transport_groups: Vec<TransportGroup>,
}

impl<'attrs> TryFrom<FirPkgAttr<'attrs>> for SirPkaAttr<'attrs> {
  type Error = crate::Error;

  fn try_from(fea: FirPkgAttr<'attrs>) -> Result<Self, Self::Error> {
    let data_formats = fea
      .data_formats
      .into_iter()
      .map(TryInto::try_into)
      .collect::<crate::Result<Vec<DataFormat>>>()?;
    if data_formats.is_empty() {
      return Err(crate::Error::MandatoryOuterAttrsAreNotPresent);
    }
    Ok(Self {
      api: fea.api,
      data_formats,
      transport_groups: fea
        .transports
        .into_iter()
        .map(TryInto::try_into)
        .collect::<crate::Result<_>>()?,
    })
  }
}
