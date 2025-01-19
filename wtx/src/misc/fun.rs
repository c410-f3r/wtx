macro_rules! create_and_implement {
  ($fn_ty:ident, ($($tys:ident),*)) => {
    impl<$($tys,)* FUN, RSLT> Fun<($($tys,)*)> for FUN
    where
      FUN: $fn_ty($($tys,)*) -> RSLT,
    {
      type Output = RSLT;
    }
  };
}

/// Used internally for type inference.
pub trait Fun<A> {
  /// Function output.
  type Output;
}

create_and_implement!(Fn, ());
