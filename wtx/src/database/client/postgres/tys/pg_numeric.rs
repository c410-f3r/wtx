use crate::{
  collection::ArrayVector,
  database::{
    DatabaseError,
    client::postgres::{DecodeWrapper, EncodeWrapper, Postgres, PostgresError},
  },
  de::{Decode, Encode},
  misc::Usize,
};

const _DIGITS_CAP: usize = 64;
const SIGN_NAN: u16 = 0xC000;
const SIGN_NEG: u16 = 0x4000;
const SIGN_POS: u16 = 0x0000;

pub(crate) enum PgNumeric {
  NaN,
  Number { digits: ArrayVector<i16, _DIGITS_CAP>, scale: u16, sign: Sign, weight: i16 },
}

impl<E> Decode<'_, Postgres<E>> for PgNumeric
where
  E: From<crate::Error>,
{
  #[inline]
  fn decode(_: &mut (), dw: &mut DecodeWrapper<'_>) -> Result<Self, E> {
    let [a, b, c, d, e, f, g, h, rest @ ..] = dw.bytes() else {
      return Err(E::from(
        DatabaseError::UnexpectedBufferSize {
          expected: 8,
          received: Usize::from(dw.bytes().len()).into_u64().try_into().unwrap_or(u32::MAX),
        }
        .into(),
      ));
    };
    let digits = u16::from_be_bytes([*a, *b]);
    let digits_usize = usize::from(digits);
    let weight = i16::from_be_bytes([*c, *d]);
    let sign = u16::from_be_bytes([*e, *f]);
    let scale = u16::from_be_bytes([*g, *h]);
    let mut curr_slice = rest;
    Ok(if sign == SIGN_NAN {
      PgNumeric::NaN
    } else {
      if digits_usize > _DIGITS_CAP || digits_usize > 0x7FFF {
        return Err(E::from(PostgresError::VeryLargeDecimal.into()));
      }
      let mut array = [0i16; _DIGITS_CAP];
      for elem in array.iter_mut().take(digits_usize) {
        let [i, j, local_rest @ ..] = curr_slice else {
          break;
        };
        *elem = i16::from_be_bytes([*i, *j]);
        curr_slice = local_rest;
      }
      PgNumeric::Number {
        digits: ArrayVector::from_parts(array, Some(digits.into())),
        scale,
        sign: Sign::try_from(sign)?,
        weight,
      }
    })
  }
}
impl<E> Encode<Postgres<E>> for PgNumeric
where
  E: From<crate::Error>,
{
  #[inline]
  fn encode(&self, _: &mut (), ew: &mut EncodeWrapper<'_, '_>) -> Result<(), E> {
    match self {
      PgNumeric::NaN => {
        ew.buffer().extend_from_slice(&0i16.to_be_bytes())?;
        ew.buffer().extend_from_slice(&0i16.to_be_bytes())?;
        ew.buffer().extend_from_slice(&SIGN_NAN.to_be_bytes())?;
        ew.buffer().extend_from_slice(&0u16.to_be_bytes())?;
      }
      PgNumeric::Number { digits, scale, sign, weight } => {
        let len: i16 = digits.len().try_into().map_err(Into::into)?;
        ew.buffer().extend_from_slice(&len.to_be_bytes())?;
        ew.buffer().extend_from_slice(&weight.to_be_bytes())?;
        ew.buffer().extend_from_slice(&u16::from(*sign).to_be_bytes())?;
        ew.buffer().extend_from_slice(&scale.to_be_bytes())?;
        for digit in digits {
          ew.buffer().extend_from_slice(&digit.to_be_bytes())?;
        }
      }
    }
    Ok(())
  }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Sign {
  Negative,
  Positive,
}

impl From<Sign> for u16 {
  #[inline]
  fn from(from: Sign) -> Self {
    match from {
      Sign::Negative => SIGN_NEG,
      Sign::Positive => SIGN_POS,
    }
  }
}

impl TryFrom<u16> for Sign {
  type Error = crate::Error;

  #[inline]
  fn try_from(from: u16) -> Result<Self, Self::Error> {
    Ok(match from {
      SIGN_NAN => return Err(PostgresError::DecimalCanNotBeConvertedFromNaN.into()),
      SIGN_NEG => Self::Negative,
      SIGN_POS => Self::Positive,
      _ => return Err(crate::Error::UnexpectedUint { received: from.into() }),
    })
  }
}
