use crate::misc::{LeaseMut, memset_slice_volatile, mlock_slice, munlock_slice};
use core::{
  fmt::Debug,
  ops::{Deref, DerefMut},
};

/// Bytes that are zeroed when dropped. See `Secret` for a more confidential container.
pub struct SensitiveBytes<B>
where
  B: LeaseMut<[u8]>,
{
  bytes: B,
  is_locked: bool,
}

impl<B> SensitiveBytes<B>
where
  B: LeaseMut<[u8]>,
{
  /// New instance ***with*** `mlock`ed bytes
  #[inline]
  pub fn new_locked(mut bytes: B) -> crate::Result<Self> {
    mlock_slice(bytes.lease_mut())?;
    Ok(Self { bytes, is_locked: true })
  }

  /// New instance ***without*** `mlock`ed bytes
  #[inline]
  pub const fn new_unlocked(bytes: B) -> Self {
    Self { bytes, is_locked: false }
  }
}

impl<B> Debug for SensitiveBytes<B>
where
  B: LeaseMut<[u8]>,
{
  #[inline]
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    f.debug_tuple("SensitiveBytes").finish()
  }
}

impl<B> Deref for SensitiveBytes<B>
where
  B: LeaseMut<[u8]>,
{
  type Target = B;

  #[inline]
  fn deref(&self) -> &Self::Target {
    &self.bytes
  }
}

impl<B> DerefMut for SensitiveBytes<B>
where
  B: LeaseMut<[u8]>,
{
  #[inline]
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.bytes
  }
}

impl<B> Drop for SensitiveBytes<B>
where
  B: LeaseMut<[u8]>,
{
  #[inline]
  fn drop(&mut self) {
    memset_slice_volatile(self.bytes.lease_mut(), 0);
    if self.is_locked {
      let rslt = munlock_slice(self.bytes.lease_mut());
      debug_assert!(rslt.is_ok());
    }
  }
}

#[cfg(feature = "database")]
mod database {
  use crate::{
    codec::{Decode, Encode},
    database::{Database, Typed},
    misc::{LeaseMut, SensitiveBytes},
  };

  impl<'de, B, DB> Decode<'de, DB> for SensitiveBytes<B>
  where
    B: Decode<'de, DB> + LeaseMut<[u8]>,
    DB: Database,
  {
    fn decode(dw: &mut DB::DecodeWrapper<'de, '_, '_>) -> Result<Self, DB::Error> {
      let data: B = Decode::<'_, DB>::decode(dw)?;
      Ok(Self::new_locked(data)?)
    }
  }

  impl<B, DB> Encode<DB> for SensitiveBytes<B>
  where
    B: Encode<DB> + LeaseMut<[u8]>,
    DB: Database,
  {
    fn encode(&self, ew: &mut DB::EncodeWrapper<'_, '_, '_>) -> Result<(), DB::Error> {
      <B as Encode<DB>>::encode(self, ew)
    }
  }

  impl<B, DB> Typed<DB> for SensitiveBytes<B>
  where
    B: LeaseMut<[u8]> + Typed<DB>,
    DB: Database,
  {
    fn runtime_ty(&self) -> Option<DB::Ty> {
      self.bytes.runtime_ty()
    }

    fn static_ty() -> Option<DB::Ty> {
      B::static_ty()
    }
  }
}
