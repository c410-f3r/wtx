use crate::{
  collection::ArrayVector,
  database::{
    Typed,
    client::postgres::{
      DecodeWrapper, EncodeWrapper, Postgres, PostgresError, Ty,
      tys::pg_numeric::{PgNumeric, Sign},
    },
  },
  misc::{Decode, Encode},
};
use rust_decimal::{Decimal, MathematicalOps};

impl<E> Decode<'_, Postgres<E>> for Decimal
where
  E: From<crate::Error>,
{
  #[inline]
  fn decode(aux: &mut (), dw: &mut DecodeWrapper<'_>) -> Result<Self, E> {
    let pg_numeric = PgNumeric::decode(aux, dw)?;
    let (digits, sign, mut weight, scale) = match pg_numeric {
      PgNumeric::NaN => {
        return Err(E::from(PostgresError::DecimalCanNotBeConvertedFromNaN.into()));
      }
      PgNumeric::Number { digits, sign, weight, scale } => (digits, sign, weight, scale),
    };
    if digits.is_empty() {
      return Ok(0u64.into());
    }
    let mut value = Decimal::ZERO;
    for digit in digits.into_iter() {
      let mut operations = || {
        let mul = Decimal::from(10_000u16).checked_powi(weight.into())?;
        let part = Decimal::from(digit).checked_mul(mul)?;
        value = value.checked_add(part)?;
        weight = weight.checked_sub(1)?;
        Some(())
      };
      operations().ok_or_else(|| crate::Error::OutOfBoundsArithmetic)?;
    }
    match sign {
      Sign::Positive => value.set_sign_positive(true),
      Sign::Negative => value.set_sign_negative(true),
    }
    value.rescale(scale.into());
    Ok(value)
  }
}

impl<E> Encode<Postgres<E>> for Decimal
where
  E: From<crate::Error>,
{
  #[inline]
  fn encode(&self, aux: &mut (), ew: &mut EncodeWrapper<'_, '_>) -> Result<(), E> {
    if self.is_zero() {
      let rslt =
        PgNumeric::Number { digits: ArrayVector::new(), scale: 0, sign: Sign::Positive, weight: 0 };
      rslt.encode(aux, ew)?;
      return Ok(());
    }

    let scale = self.scale() as u16;

    let mut mantissa = u128::from_le_bytes(self.serialize());
    mantissa >>= 32;
    let diff = scale % 4;
    if diff > 0 {
      let remainder = 4u32.wrapping_sub(u32::from(diff));
      mantissa = mantissa.wrapping_mul(u128::from(10u32.pow(remainder)));
    }

    let mut digits = ArrayVector::new();
    while mantissa != 0 {
      digits.push((mantissa % 10_000) as i16)?;
      mantissa /= 10_000;
    }
    digits.reverse();

    let after_decimal = scale.wrapping_add(3) / 4;
    let weight = digits.len().wrapping_sub(after_decimal.into()).wrapping_sub(1) as i16;

    while let Some(&0) = digits.last() {
      let _ = digits.pop();
    }

    let rslt = PgNumeric::Number {
      digits,
      scale,
      sign: match self.is_sign_negative() {
        false => Sign::Positive,
        true => Sign::Negative,
      },
      weight,
    };
    rslt.encode(aux, ew)?;
    Ok(())
  }
}

impl<E> Typed<Postgres<E>> for Decimal
where
  E: From<crate::Error>,
{
  #[inline]
  fn runtime_ty(&self) -> Option<Ty> {
    <Self as Typed<Postgres<E>>>::static_ty()
  }

  #[inline]
  fn static_ty() -> Option<Ty> {
    Some(Ty::Numeric)
  }
}

kani!(rust_decimal, Decimal);
