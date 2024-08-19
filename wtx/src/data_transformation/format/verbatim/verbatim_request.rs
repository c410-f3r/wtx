use crate::{data_transformation::dnsn::Serialize, misc::Vector};

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
#[doc = generic_data_format_doc!("verbatim request")]
pub struct VerbatimRequest<D> {
  /// Actual data
  pub data: D,
}

impl<D> Serialize<()> for VerbatimRequest<D> {
  #[inline]
  fn to_bytes(&mut self, _: &mut Vector<u8>, _: &mut ()) -> crate::Result<()> {
    Ok(())
  }
}

#[cfg(feature = "rkyv")]
mod rkyv {
  use crate::{
    data_transformation::{
      dnsn::{Rkyv, _InnerSerializer},
      format::VerbatimRequest,
    },
    misc::Vector,
  };

  impl<D> crate::data_transformation::dnsn::Serialize<Rkyv> for VerbatimRequest<D>
  where
    for<'any> D: rkyv::Serialize<_InnerSerializer<'any>>,
  {
    #[inline]
    fn to_bytes(&mut self, bytes: &mut Vector<u8>, drsr: &mut Rkyv) -> crate::Result<()> {
      drsr._serialize(bytes, &self.data)?;
      Ok(())
    }
  }
}
