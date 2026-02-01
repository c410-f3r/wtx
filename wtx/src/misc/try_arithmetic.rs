/// Try Arithmetic Error
#[derive(Clone, Copy, Debug)]
pub enum TryArithmeticError {
  /// The result of an addition is greater than the underlying capacity
  AddOverflow,
  /// Division by zero or an overflow involving signed numbers
  DivOverflow,
  /// The result of a multiplication is greater than the underlying capacity
  MulOverflow,
  /// Remainder by zero or an overflow involving signed numbers
  RemOverflow,
  /// The result of a subtraction is lesser or greater than the underlying capacity
  SubOverflow,
}

/// Abstracts over fallible arithmetic operations
pub trait TryArithmetic<Rhs = Self> {
  /// The resulting type.
  type Output;

  /// Performs the `+` operation.
  fn try_add(&self, rhs: Rhs) -> crate::Result<Self::Output>;

  /// Performs the `/` operation.
  fn try_div(&self, rhs: Rhs) -> crate::Result<Self::Output>;

  /// Performs the `*` operation.
  fn try_mul(&self, rhs: Rhs) -> crate::Result<Self::Output>;

  /// Performs the `%` operation.
  fn try_rem(&self, rhs: Rhs) -> crate::Result<Self::Output>;

  /// Performs the `-` operation.
  fn try_sub(&self, rhs: Rhs) -> crate::Result<Self::Output>;
}

#[cfg(feature = "rust_decimal")]
impl TryArithmetic<rust_decimal::Decimal> for rust_decimal::Decimal {
  type Output = rust_decimal::Decimal;

  #[inline]
  fn try_add(&self, rhs: rust_decimal::Decimal) -> crate::Result<Self::Output> {
    Ok(self.checked_add(rhs).ok_or(TryArithmeticError::AddOverflow)?)
  }

  #[inline]
  fn try_div(&self, rhs: rust_decimal::Decimal) -> crate::Result<Self::Output> {
    Ok(self.checked_div(rhs).ok_or(TryArithmeticError::AddOverflow)?)
  }

  #[inline]
  fn try_mul(&self, rhs: rust_decimal::Decimal) -> crate::Result<Self::Output> {
    Ok(self.checked_mul(rhs).ok_or(TryArithmeticError::AddOverflow)?)
  }

  #[inline]
  fn try_rem(&self, rhs: rust_decimal::Decimal) -> crate::Result<Self::Output> {
    Ok(self.checked_rem(rhs).ok_or(TryArithmeticError::AddOverflow)?)
  }

  #[inline]
  fn try_sub(&self, rhs: rust_decimal::Decimal) -> crate::Result<Self::Output> {
    Ok(self.checked_sub(rhs).ok_or(TryArithmeticError::AddOverflow)?)
  }
}

macro_rules! impl_float {
  ($($ty:ty)*) => {
    $(
      impl TryArithmetic<$ty> for $ty {
        type Output = $ty;

        #[inline]
        fn try_add(&self, rhs: $ty) -> crate::Result<Self::Output> {
          Ok(self + rhs)
        }

        #[inline]
        fn try_div(&self, rhs: $ty) -> crate::Result<Self::Output> {
          Ok(self / rhs)
        }

        #[inline]
        fn try_mul(&self, rhs: $ty) -> crate::Result<Self::Output> {
          Ok(self * rhs)
        }

        #[inline]
        fn try_rem(&self, rhs: $ty) -> crate::Result<Self::Output> {
          Ok(self % rhs)
        }

        #[inline]
        fn try_sub(&self, rhs: $ty) -> crate::Result<Self::Output> {
          Ok(self - rhs)
        }
      }
    )*
  };
}
macro_rules! impl_integer {
  ($($ty:ty)*) => {
    $(
      impl TryArithmetic<$ty> for $ty {
        type Output = $ty;

        #[inline]
        fn try_add(&self, rhs: $ty) -> crate::Result<Self::Output> {
          Ok(self.checked_add(rhs).ok_or(TryArithmeticError::AddOverflow)?)
        }

        #[inline]
        fn try_div(&self, rhs: $ty) -> crate::Result<Self::Output> {
          Ok(self.checked_div(rhs).ok_or(TryArithmeticError::DivOverflow)?)
        }

        #[inline]
        fn try_mul(&self, rhs: $ty) -> crate::Result<Self::Output> {
          Ok(self.checked_mul(rhs).ok_or(TryArithmeticError::MulOverflow)?)
        }

        #[inline]
        fn try_rem(&self, rhs: $ty) -> crate::Result<Self::Output> {
          Ok(self.checked_rem(rhs).ok_or(TryArithmeticError::RemOverflow)?)
        }

        #[inline]
        fn try_sub(&self, rhs: $ty) -> crate::Result<Self::Output> {
          Ok(self.checked_sub(rhs).ok_or(TryArithmeticError::SubOverflow)?)
        }
      }
    )*
  };
}

impl_float!(f32 f64);
impl_integer!(i8 i16 i32 i64 i128 isize u8 u16 u32 u64 u128 usize);
