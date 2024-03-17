use crate::client_api_framework::dnsn::Serialize;
use alloc::vec::Vec;

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
#[doc = generic_data_format_doc!("Borsh request")]
pub struct BorshRequest<D> {
  /// Actual data
  pub data: D,
}

impl<D> Serialize<()> for BorshRequest<D> {
  #[inline]
  fn to_bytes(&mut self, _: &mut Vec<u8>, _: &mut ()) -> crate::Result<()> {
    Ok(())
  }
}

#[cfg(feature = "borsh")]
mod borsh {
  use crate::client_api_framework::{data_format::BorshRequest, dnsn::Borsh};
  use alloc::vec::Vec;
  use borsh::BorshSerialize;

  impl<D> crate::client_api_framework::dnsn::Serialize<Borsh> for BorshRequest<D>
  where
    D: BorshSerialize,
  {
    fn to_bytes(&mut self, bytes: &mut Vec<u8>, _: &mut Borsh) -> crate::Result<()> {
      self.data.serialize(bytes)?;
      Ok(())
    }
  }
}
