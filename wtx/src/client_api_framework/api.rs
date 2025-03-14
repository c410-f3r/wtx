use core::fmt::Display;

/// Api definitions group different packages into a common namespace and define custom additional
/// logical through hooks.
pub trait Api {
  /// Any custom error structure that can be constructed from [`crate::Error`].
  type Error: Display + From<crate::Error>;
  /// Identifier
  type Id: ApiId;

  /// Fallible hook that is automatically called after sending any related request.
  #[inline]
  fn after_sending(&mut self) -> impl Future<Output = Result<(), Self::Error>> {
    async { Ok(()) }
  }

  /// Fallible hook that is automatically called before sending any related request.
  #[inline]
  fn before_sending(&mut self) -> impl Future<Output = Result<(), Self::Error>> {
    async { Ok(()) }
  }
}

impl Api for () {
  type Error = crate::Error;
  type Id = ();
}

impl<T> Api for &mut T
where
  T: Api,
{
  type Error = T::Error;
  type Id = T::Id;

  #[inline]
  async fn after_sending(&mut self) -> Result<(), Self::Error> {
    (**self).after_sending().await
  }

  #[inline]
  async fn before_sending(&mut self) -> Result<(), Self::Error> {
    (**self).before_sending().await
  }
}

/// Identification of an API.
pub trait ApiId {
  /// Associated API
  type Api<T>: Api;
}

impl ApiId for () {
  type Api<T> = ();
}
