pub trait FunWithArgs<Args> {
  type Output;

  fn my_method(&self, args: Args) -> Self::Output;
}

impl<FUN, OUT> FunWithArgs<()> for FUN
where
  FUN: Fn() -> OUT,
{
  type Output = OUT;

  #[inline]
  fn my_method(&self, _: ()) -> Self::Output {
    self()
  }
}

impl<FUN, A, OUT> FunWithArgs<(A,)> for FUN
where
  FUN: Fn(A) -> OUT,
{
  type Output = OUT;

  #[inline]
  fn my_method(&self, args: (A,)) -> Self::Output {
    self(args.0)
  }
}

impl<FUN, A, B, OUT> FunWithArgs<(A, B)> for FUN
where
  FUN: Fn(A, B) -> OUT,
{
  type Output = OUT;

  #[inline]
  fn my_method(&self, args: (A, B)) -> Self::Output {
    self(args.0, args.1)
  }
}
