use core::future::Future;

/// Simulates `impl for<'any> FnMut(&'any ..) -> impl Future + 'any` due to the lack of compiler
/// support.
///
/// If applied as a function parameter, then callers should create their own async functions
/// instead of using closures.
///
/// Credits to `Daniel Henry-Mantilla`.
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
