use crate::{
  collection::ArrayVectorU8,
  database::{
    Typed,
    client::postgres::{
      DecodeWrapper, EncodeWrapper, Postgres, PostgresError, Ty,
      tys::pg_numeric::{PgNumeric, Sign},
    },
  },
  de::{Decode, Encode},
};
use rust_decimal::{Decimal, MathematicalOps};

impl<E> Decode<'_, Postgres<E>> for Decimal
where
  E: From<crate::Error>,
{
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'_, '_>) -> Result<Self, E> {
    Ok(PgNumeric::decode(dw)?.try_into()?)
  }
}

impl<E> Encode<Postgres<E>> for Decimal
where
  E: From<crate::Error>,
{
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_, '_>) -> Result<(), E> {
    PgNumeric::try_from(*self)?.encode(ew)
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

impl TryFrom<Decimal> for PgNumeric {
  type Error = crate::Error;

  #[inline]
  fn try_from(value: Decimal) -> Result<Self, Self::Error> {
    if value.is_zero() {
      return Ok(PgNumeric::Number {
        digits: ArrayVectorU8::new(),
        scale: 0,
        sign: Sign::Positive,
        weight: 0,
      });
    }

    let scale = value.scale() as u16;

    let mut mantissa = value.mantissa().unsigned_abs();
    let diff = scale % 4;
    if diff > 0 {
      let remainder = 4u32.wrapping_sub(u32::from(diff));
      mantissa = mantissa.wrapping_mul(u128::from(10u32.pow(remainder)));
    }

    let mut digits = ArrayVectorU8::new();
    while mantissa != 0 {
      digits.push((mantissa % 10_000) as i16)?;
      mantissa /= 10_000;
    }
    digits.reverse();

    let after_decimal = scale.wrapping_add(3) / 4;
    let weight = u16::from(digits.len()).wrapping_sub(after_decimal).wrapping_sub(1) as i16;

    while let Some(&0) = digits.last() {
      let _ = digits.pop();
    }

    Ok(PgNumeric::Number {
      digits,
      scale,
      sign: match value.is_sign_negative() {
        false => Sign::Positive,
        true => Sign::Negative,
      },
      weight,
    })
  }
}

impl TryFrom<PgNumeric> for Decimal {
  type Error = crate::Error;

  #[inline]
  fn try_from(value: PgNumeric) -> Result<Self, Self::Error> {
    let (digits, sign, mut weight, scale) = match value {
      PgNumeric::NaN => {
        return Err(PostgresError::DecimalCanNotBeConvertedFromNaN.into());
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

kani!(rust_decimal, Decimal);

#[cfg(test)]
mod tests {
  use crate::database::client::postgres::tys::pg_numeric::{PgNumeric, Sign};
  use rust_decimal::Decimal;

  #[test]
  fn encodes_and_decodes() {
    let original: Decimal = "12345.67890".try_into().unwrap();
    let encoded = PgNumeric::try_from(original).unwrap();
    assert_eq!(
      encoded,
      PgNumeric::Number {
        sign: Sign::Positive,
        scale: 5,
        weight: 1,
        digits: [1, 2345, 6789].as_ref().try_into().unwrap(),
      }
    );
    let decoded = Decimal::try_from(encoded).unwrap();
    assert_eq!(decoded, original);
  }
}
