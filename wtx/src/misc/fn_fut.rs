use core::future::Future;

/// Simulates `impl for<'any> Fn(&'any ..) -> impl Future + 'any` due to the lack of compiler
/// support.
///
/// If applied as a function parameter, then callers must create their own async functions
/// instead of using closures.
///
/// Credits to `Daniel Henry-Mantilla`.
pub trait FnFut<P, R>: Fn(P) -> Self::Future {
  /// Returning future.
  type Future: Future<Output = R>;
}

impl<P, F, FUT, R> FnFut<P, R> for F
where
  F: Fn(P) -> FUT,
  FUT: Future<Output = R>,
{
  type Future = FUT;
}

/// Like [`FnFut`] but for [`FnMut`].
pub trait FnMutFut<P, R>: FnMut(P) -> Self::Future {
  /// Returning future.
  type Future: Future<Output = R>;
}

impl<P, F, FUT, R> FnMutFut<P, R> for F
where
  F: FnMut(P) -> FUT,
  FUT: Future<Output = R>,
{
  type Future = FUT;
}

/// Like [`FnFut`] but for [`FnOnce`].
pub trait FnOnceFut<P, R>: FnOnce(P) -> Self::Future {
  /// Returning future.
  type Future: Future<Output = R>;
}

impl<P, F, FUT, R> FnOnceFut<P, R> for F
where
  F: FnOnce(P) -> FUT,
  FUT: Future<Output = R>,
{
  type Future = FUT;
}
