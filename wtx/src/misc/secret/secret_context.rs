use crate::{misc::secret::protected::Protected, rng::CryptoRng, sync::Arc};
use core::fmt::{Debug, Formatter};

const CTX_LEN: usize = 1024;

/// Used by `Secret`, can be freely cloned and shared across threads.
#[derive(Clone)]
pub struct SecretContext {
  pub(crate) _inner: Arc<Protected>,
}

impl SecretContext {
  /// New instance
  #[inline]
  pub fn new<RNG>(rng: &mut RNG) -> crate::Result<Self>
  where
    RNG: CryptoRng,
  {
    let mut protected = Protected::zeroed(CTX_LEN);
    rng.fill_slice(&mut protected);
    #[cfg(feature = "libc")]
    crate::misc::mlock_slice(&mut protected)?;
    Ok(Self { _inner: Arc::new(protected) })
  }
}

impl Debug for SecretContext {
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    f.debug_struct("SecretContext").finish()
  }
}
