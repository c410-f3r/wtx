macro_rules! create_and_implement {
  ($name:ident, $fn_ty:ident, $fn_fut_ty:ident, $n:literal, ($($tys:ident,)*)) => {
    /// [`
    #[doc = stringify!($fn_fut_ty)]
    /// `] with
    #[doc = $n]
    /// argument(s).
    pub trait $name<$($tys,)*>: $fn_ty($($tys,)*) -> Self::Future + $fn_fut_ty<($($tys,)*)> {}

    impl<$($tys,)* FUN, FUT, RSLT> $fn_fut_ty<($($tys,)*)> for FUN
    where
      FUN: $fn_ty($($tys,)*) -> FUT,
      FUT: Future<Output = RSLT>,
    {
      type Future = FUT;
      type Result = RSLT;
    }

    impl<$($tys,)* TY> $name<$($tys,)*> for TY
    where
      TY: $fn_ty($($tys,)*) -> Self::Future + $fn_fut_ty<($($tys,)*)>
    {}
  };
}

use core::future::Future;

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
}

create_and_implement!(FnFut0, Fn, FnFut, "0", ());
create_and_implement!(FnFut1, Fn, FnFut, "1", (A,));
create_and_implement!(FnFut2, Fn, FnFut, "2", (A, B,));
create_and_implement!(FnFut3, Fn, FnFut, "3", (A, B, C,));

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
}

create_and_implement!(FnMutFut0, FnMut, FnMutFut, "0", ());
create_and_implement!(FnMutFut1, FnMut, FnMutFut, "1", (A,));
create_and_implement!(FnMutFut2, FnMut, FnMutFut, "2", (A, B,));
create_and_implement!(FnMutFut3, FnMut, FnMutFut, "3", (A, B, C,));

/// Simulates `impl for<'any> FnOnce(&'any ..) -> impl Future + 'any` due to the lack of compiler
/// support.
///
/// If applied as a function parameter, then callers must create their own async functions
/// instead of using closures.
///
/// Credits to `Daniel Henry-Mantilla`.
pub trait FnOnceFut<A> {
  /// Returning future.
  type Future: Future<Output = Self::Result>;
  /// Function result.
  type Result;
}

create_and_implement!(FnOnceFut0, FnOnce, FnOnceFut, "0", ());
create_and_implement!(FnOnceFut1, FnOnce, FnOnceFut, "1", (A,));
create_and_implement!(FnOnceFut2, FnOnce, FnOnceFut, "2", (A, B,));
create_and_implement!(FnOnceFut3, FnOnce, FnOnceFut, "3", (A, B, C,));
