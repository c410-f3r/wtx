use crate::{
  codec::{Decode, Encode},
  collections::ArrayVectorCopy,
  database::{
    Typed,
    client::postgres::{
      Postgres, PostgresDecodeWrapper, PostgresEncodeWrapper, PostgresError, Ty,
      tys::pg_numeric::{PgNumeric, Sign},
    },
  },
};
use rust_decimal::{Decimal, MathematicalOps as _};

impl<E> Decode<'_, Postgres<E>> for Decimal
where
  E: From<crate::Error>,
{
  #[inline]
  fn decode(dw: &mut PostgresDecodeWrapper<'_, '_>) -> Result<Self, E> {
    Ok(PgNumeric::decode(dw)?.try_into()?)
  }
}

impl<E> Encode<Postgres<E>> for Decimal
where
  E: From<crate::Error>,
{
  #[inline]
  fn encode(&self, ew: &mut PostgresEncodeWrapper<'_>) -> Result<(), E> {
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
        digits: ArrayVectorCopy::new(),
        scale: 0,
        sign: Sign::Positive,
        weight: 0,
      });
    }

    let scale: u16 = value.scale().try_into()?;

    let mut mantissa = value.mantissa().unsigned_abs();
    let diff = scale % 4;
    if diff > 0 {
      let remainder = 4u32.wrapping_sub(u32::from(diff));
      mantissa = mantissa.wrapping_mul(u128::from(10u32.pow(remainder)));
    }

    let mut digits = ArrayVectorCopy::new();
    while mantissa != 0 {
      digits.push(i16::try_from(mantissa % 10_000)?)?;
      mantissa /= 10_000;
    }
    digits.reverse();

    let after_decimal = i16::try_from(scale.wrapping_add(3) / 4)?;
    let weight = i16::from(digits.len()).wrapping_sub(after_decimal).wrapping_sub(1);

    while let Some(&0) = digits.last() {
      let _ = digits.pop();
    }

    Ok(PgNumeric::Number {
      digits,
      scale,
      sign: if value.is_sign_negative() { Sign::Negative } else { Sign::Positive },
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
    let mut num = Decimal::ZERO;
    for digit in digits {
      let mut operations = || {
        let mul = Decimal::from(10_000u16).checked_powi(weight.into())?;
        let part = Decimal::from(digit).checked_mul(mul)?;
        num = num.checked_add(part)?;
        weight = weight.checked_sub(1)?;
        Some(())
      };
      operations().ok_or(PostgresError::OutOfBoundsNumericArithmetic)?;
    }
    match sign {
      Sign::Positive => num.set_sign_positive(true),
      Sign::Negative => num.set_sign_negative(true),
    }
    num.rescale(scale.into());
    Ok(num)
  }
}

kani!(rust_decimal, Decimal);

#[cfg(test)]
mod tests {
  use crate::database::client::postgres::tys::pg_numeric::PgNumeric;
  use rust_decimal::Decimal;

  #[test]
  fn encodes_and_decodes() {
    {
      let original: Decimal = "-12345.67890".try_into().unwrap();
      let encoded = PgNumeric::try_from(original).unwrap();
      let decoded = Decimal::try_from(encoded).unwrap();
      assert_eq!(decoded, original);
    }

    {
      let original: Decimal = "-10000".try_into().unwrap();
      let encoded = PgNumeric::try_from(original).unwrap();
      let decoded = Decimal::try_from(encoded).unwrap();
      assert_eq!(decoded, original);
    }

    {
      let original: Decimal = Decimal::ZERO;
      let encoded = PgNumeric::try_from(original).unwrap();
      let decoded = Decimal::try_from(encoded).unwrap();
      assert_eq!(decoded, original);
    }

    {
      let original: Decimal = "0.0000".try_into().unwrap();
      let encoded = PgNumeric::try_from(original).unwrap();
      let decoded = Decimal::try_from(encoded).unwrap();
      assert_eq!(decoded, original);
    }

    {
      let original: Decimal = "0.0001".try_into().unwrap();
      let encoded = PgNumeric::try_from(original).unwrap();
      let decoded = Decimal::try_from(encoded).unwrap();
      assert_eq!(decoded, original);
    }

    {
      let original: Decimal = "0.00000001".try_into().unwrap();
      let encoded = PgNumeric::try_from(original).unwrap();
      let decoded = Decimal::try_from(encoded).unwrap();
      assert_eq!(decoded, original);
    }

    {
      let original: Decimal = "123.4500".try_into().unwrap();
      let encoded = PgNumeric::try_from(original).unwrap();
      let decoded = Decimal::try_from(encoded).unwrap();
      assert_eq!(original, decoded);
    }

    {
      let original: Decimal = "12345.67890".try_into().unwrap();
      let encoded = PgNumeric::try_from(original).unwrap();
      let decoded = Decimal::try_from(encoded).unwrap();
      assert_eq!(decoded, original);
    }

    {
      let original: Decimal = "10000".try_into().unwrap();
      let encoded = PgNumeric::try_from(original).unwrap();
      let decoded = Decimal::try_from(encoded).unwrap();
      assert_eq!(decoded, original);
    }

    {
      let original: Decimal = "100000".try_into().unwrap();
      let encoded = PgNumeric::try_from(original).unwrap();
      let decoded = Decimal::try_from(encoded).unwrap();
      assert_eq!(decoded, original);
    }

    {
      let original: Decimal = "10000000000".try_into().unwrap();
      let encoded = PgNumeric::try_from(original).unwrap();
      let decoded = Decimal::try_from(encoded).unwrap();
      assert_eq!(decoded, original);
    }
  }
}
