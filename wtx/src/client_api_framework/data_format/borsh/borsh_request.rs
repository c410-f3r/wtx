use crate::{client_api_framework::dnsn::Serialize, misc::Vector};

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
#[doc = generic_data_format_doc!("Borsh request")]
pub struct BorshRequest<D> {
  /// Actual data
  pub data: D,
}

impl<D> Serialize<()> for BorshRequest<D> {
  #[inline]
  fn to_bytes(&mut self, _: &mut Vector<u8>, _: &mut ()) -> crate::Result<()> {
    Ok(())
  }
}

#[cfg(feature = "borsh")]
mod borsh {
  use crate::{
    client_api_framework::{data_format::BorshRequest, dnsn::Borsh},
    misc::Vector,
  };
  use borsh::BorshSerialize;

  impl<D> crate::client_api_framework::dnsn::Serialize<Borsh> for BorshRequest<D>
  where
    D: BorshSerialize,
  {
    fn to_bytes(&mut self, bytes: &mut Vector<u8>, _: &mut Borsh) -> crate::Result<()> {
      self.data.serialize(bytes)?;
      Ok(())
    }
  }
}
