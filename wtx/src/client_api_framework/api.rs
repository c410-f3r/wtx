use crate::misc::AsyncBounds;
use core::{fmt::Display, future::Future};

/// Api definitions group different packages into a common namespace and define custom additional
/// logical through hooks.
pub trait Api {
  /// Any custom error structure that can be constructed from [crate::Error].
  type Error: Display + From<crate::Error>;

  /// Fallible hook that is automatically called after sending any related request.
  #[inline]
  fn after_sending(&mut self) -> impl AsyncBounds + Future<Output = Result<(), Self::Error>> {
    async { Ok(()) }
  }

  /// Fallible hook that is automatically called before sending any related request.
  #[inline]
  fn before_sending(&mut self) -> impl AsyncBounds + Future<Output = Result<(), Self::Error>> {
    async { Ok(()) }
  }
}

impl Api for () {
  type Error = crate::Error;
}

impl<T> Api for &mut T
where
  T: Api + AsyncBounds,
{
  type Error = T::Error;

  #[inline]
  async fn after_sending(&mut self) -> Result<(), Self::Error> {
    (**self).before_sending().await
  }

  #[inline]
  async fn before_sending(&mut self) -> Result<(), Self::Error> {
    (**self).before_sending().await
  }
}
