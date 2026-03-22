use crate::{
  collection::{ExpansionTy, Vector},
  misc::mlock_slice,
  rng::Rng,
  sync::Arc,
};
use alloc::boxed::Box;
use core::fmt::{Debug, Formatter};

const STATIC_KEYS_NUM: usize = 4;
const STATIC_KEYS_SIZE: usize = 4096;

/// Used by `Secret`, can be freely cloned and shared across threads.
#[derive(Clone)]
pub struct SecretContext(pub(crate) Arc<[Box<[u8]>]>);

impl SecretContext {
  /// New instance
  pub fn new<RNG>(rng: &mut RNG) -> crate::Result<Self>
  where
    RNG: Rng,
  {
    let mut pages = Vector::new();
    for _ in 0..STATIC_KEYS_NUM {
      let mut page = Vector::with_capacity(STATIC_KEYS_SIZE)?;
      page.expand(ExpansionTy::Len(STATIC_KEYS_SIZE), 0)?;
      rng.fill_slice(&mut page);
      mlock_slice(&mut page)?;
      pages.push(page.into_vec().into())?;
    }
    Ok(Self(pages.into_vec().into_boxed_slice().into()))
  }
}

impl Debug for SecretContext {
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    f.debug_struct("SecretContext").finish()
  }
}
