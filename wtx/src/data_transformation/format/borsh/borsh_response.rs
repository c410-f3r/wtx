use crate::{
  data_transformation::dnsn::{Deserialize, Serialize},
  misc::Vector,
};

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
#[doc = generic_data_format_doc!("Borsh response")]
pub struct BorshResponse<D> {
  /// Actual data
  pub data: D,
}

impl<'de, D> Deserialize<'de, ()> for BorshResponse<D>
where
  D: Default,
{
  #[inline]
  fn from_bytes(_: &'de [u8], _: &mut ()) -> crate::Result<Self> {
    Ok(Self { data: D::default() })
  }

  #[inline]
  fn seq_from_bytes(_: &'de [u8], _: &mut ()) -> impl Iterator<Item = crate::Result<Self>> {
    [].into_iter()
  }
}

impl<D> Serialize<()> for BorshResponse<D> {
  #[inline]
  fn to_bytes(&mut self, _: &mut Vector<u8>, _: &mut ()) -> crate::Result<()> {
    Ok(())
  }
}

#[cfg(feature = "borsh")]
mod borsh {
  use crate::data_transformation::{dnsn::Borsh, format::BorshResponse};
  use borsh::BorshDeserialize;

  impl<'de, D> crate::data_transformation::dnsn::Deserialize<'de, Borsh> for BorshResponse<D>
  where
    D: BorshDeserialize,
  {
    fn from_bytes(mut bytes: &'de [u8], _: &mut Borsh) -> crate::Result<Self> {
      Ok(Self { data: D::deserialize(&mut bytes)? })
    }

    fn seq_from_bytes(_: &'de [u8], _: &mut Borsh) -> impl Iterator<Item = crate::Result<Self>> {
      [].into_iter()
    }
  }
}
