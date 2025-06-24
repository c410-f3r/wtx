use crate::misc::{Lease, LeaseMut};

/// Wrapper used to work around coherence rules.
#[derive(Debug)]
pub struct Wrapper<T>(
  /// Element
  pub T,
);

impl<T> Lease<T> for Wrapper<T> {
  #[inline]
  fn lease(&self) -> &T {
    &self.0
  }
}

impl<T> Lease<Wrapper<T>> for Wrapper<T> {
  #[inline]
  fn lease(&self) -> &Wrapper<T> {
    self
  }
}

impl<T> LeaseMut<T> for Wrapper<T> {
  #[inline]
  fn lease_mut(&mut self) -> &mut T {
    &mut self.0
  }
}

impl<T> LeaseMut<Wrapper<T>> for Wrapper<T> {
  #[inline]
  fn lease_mut(&mut self) -> &mut Wrapper<T> {
    self
  }
}

#[cfg(feature = "serde")]
mod serde {
  use crate::{de::serde_collect_seq_rslt, misc::Wrapper};
  use core::fmt::Display;
  use serde::{Serialize, Serializer};

  impl<ELEM, ERR, T> Serialize for Wrapper<T>
  where
    ERR: Display,
    T: Clone + Iterator<Item = Result<ELEM, ERR>>,
    ELEM: Serialize,
  {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
      S: Serializer,
    {
      serde_collect_seq_rslt(serializer, self.0.clone())
    }
  }
}
