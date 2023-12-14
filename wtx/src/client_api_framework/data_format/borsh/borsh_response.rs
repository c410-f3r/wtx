use crate::client_api_framework::dnsn::{Deserialize, Serialize};
use alloc::vec::Vec;

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
#[doc = generic_data_format_doc!("Borsh response")]
pub struct BorshResponse<D> {
  /// Actual data
  pub data: D,
}

impl<D> Deserialize<()> for BorshResponse<D>
where
  D: Default,
{
  #[inline]
  fn from_bytes(_: &[u8], _: &mut ()) -> crate::Result<Self> {
    Ok(Self { data: D::default() })
  }

  #[inline]
  fn seq_from_bytes<E>(_: &[u8], _: &mut (), _: impl FnMut(Self) -> Result<(), E>) -> Result<(), E>
  where
    E: From<crate::Error>,
  {
    Ok(())
  }
}

impl<D> Serialize<()> for BorshResponse<D> {
  #[inline]
  fn to_bytes(&mut self, _: &mut Vec<u8>, _: &mut ()) -> crate::Result<()> {
    Ok(())
  }
}

#[cfg(feature = "borsh")]
mod borsh {
  use crate::client_api_framework::{data_format::BorshResponse, dnsn::Borsh};
  use borsh::BorshDeserialize;
  use core::fmt::Display;

  impl<D> crate::client_api_framework::dnsn::Deserialize<Borsh> for BorshResponse<D>
  where
    D: BorshDeserialize,
  {
    fn from_bytes(mut bytes: &[u8], _: &mut Borsh) -> crate::Result<Self> {
      Ok(Self { data: D::deserialize(&mut bytes)? })
    }

    fn seq_from_bytes<E>(
      _: &[u8],
      _: &mut Borsh,
      _: impl FnMut(Self) -> Result<(), E>,
    ) -> Result<(), E>
    where
      E: Display + From<crate::Error>,
    {
      Err(crate::Error::UnsupportedOperation.into())
    }
  }
}
