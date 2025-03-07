// FIXME(STABLE): async closures that don't imply !Send

#![allow(non_snake_case, reason = "macro stuff")]

macro_rules! create_and_implement {
  ($this:ty, $fn_ty:ident, $fn_fut_ty:ident, ($($tys:ident),*)) => {
    impl<$($tys,)* FUN, FUT, RSLT> $fn_fut_ty<($($tys,)*)> for FUN
    where
      FUN: $fn_ty($($tys,)*) -> FUT,
      FUT: Future<Output = RSLT>,
    {
      type Future = FUT;
      type Result = RSLT;
      type Wrapper = FnFutWrapper<($($tys,)*), Self>;

      #[inline]
      fn call(self: $this, ($($tys,)*): ($($tys,)*)) -> Self::Future {
        (self)($($tys,)*)
      }

      #[inline]
      fn into_wrapper(self) -> Self::Wrapper {
        FnFutWrapper(self, PhantomData)
      }
    }
  };
}

use core::marker::PhantomData;

/// A wrapper for function/closures implementations.
#[derive(Debug)]
pub struct FnFutWrapper<A, F>(pub(crate) F, pub(crate) PhantomData<A>);

impl<A, F> From<F> for FnFutWrapper<A, F> {
  #[inline]
  fn from(from: F) -> Self {
    Self(from, PhantomData)
  }
}

/// Simulates `impl for<'any> Fn(&'any ..) -> impl Future + 'any` due to the lack of compiler
/// support.
///
/// If applied as a function parameter, then callers must create their own async functions
/// instead of using closures.
///
/// Credits to `Daniel Henry-Mantilla`.
pub trait FnFut<A> {
  /// Returning future.
  type Future: Future<Output = Self::Result>;
  /// Function result.
  type Result;
  /// A wrapper for implementations.
  type Wrapper;

  /// Calls inner function that returns [`Self::Future`].
  fn call(&self, args: A) -> Self::Future;

  /// Wraps itself with [`Self::Wrapper`].
  fn into_wrapper(self) -> Self::Wrapper;
}

create_and_implement!(&Self, Fn, FnFut, ());
create_and_implement!(&Self, Fn, FnFut, (A));
create_and_implement!(&Self, Fn, FnFut, (A, B));
create_and_implement!(&Self, Fn, FnFut, (A, B, C));
create_and_implement!(&Self, Fn, FnFut, (A, B, C, D));
create_and_implement!(&Self, Fn, FnFut, (A, B, C, D, E));

/// Simulates `impl for<'any> FnMut(&'any ..) -> impl Future + 'any` due to the lack of compiler
/// support.
///
/// If applied as a function parameter, then callers must create their own async functions
/// instead of using closures.
///
/// Credits to `Daniel Henry-Mantilla`.
pub trait FnMutFut<A> {
  /// Returning future.
  type Future: Future<Output = Self::Result>;
  /// Function result.
  type Result;
  /// A wrapper that can be used to work around coherence rules.
  type Wrapper;

  /// Calls inner function that returns [`Self::Future`].
  fn call(&mut self, args: A) -> Self::Future;

  /// Wraps itself with [`Self::Wrapper`].
  fn into_wrapper(self) -> Self::Wrapper;
}

create_and_implement!(&mut Self, FnMut, FnMutFut, ());
create_and_implement!(&mut Self, FnMut, FnMutFut, (A));
create_and_implement!(&mut Self, FnMut, FnMutFut, (A, B));
create_and_implement!(&mut Self, FnMut, FnMutFut, (A, B, C));
create_and_implement!(&mut Self, FnMut, FnMutFut, (A, B, C, D));
create_and_implement!(&mut Self, FnMut, FnMutFut, (A, B, C, D, E));
