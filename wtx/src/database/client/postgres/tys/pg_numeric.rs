use crate::{
  codec::{Decode, Encode},
  collections::ArrayVectorCopy,
  database::{
    DatabaseError,
    client::postgres::{Postgres, PostgresDecodeWrapper, PostgresEncodeWrapper, PostgresError},
  },
  misc::Usize,
};

const CAP: usize = 64;
const SIGN_NAN: u16 = 0b1100_0000_0000_0000;
const SIGN_NEG: u16 = 0b0100_0000_0000_0000;
const SIGN_POS: u16 = 0b0;

#[derive(Debug, PartialEq)]
pub(crate) enum PgNumeric {
  NaN,
  Number { digits: ArrayVectorCopy<i16, CAP>, scale: u16, sign: Sign, weight: i16 },
}

impl<E> Decode<'_, Postgres<E>> for PgNumeric
where
  E: From<crate::Error>,
{
  #[inline]
  fn decode(dw: &mut PostgresDecodeWrapper<'_, '_>) -> Result<Self, E> {
    let [b0, b1, b2, b3, b4, b5, b6, b7, rest @ ..] = dw.bytes() else {
      return Err(E::from(
        DatabaseError::UnexpectedBufferSize {
          expected: 8,
          received: Usize::from(dw.bytes().len()).into_u64().try_into().unwrap_or(u32::MAX),
        }
        .into(),
      ));
    };
    let digits: u8 = u16::from_be_bytes([*b0, *b1]).try_into().map_err(crate::Error::from)?;
    let digits_usize = usize::from(digits);
    let weight = i16::from_be_bytes([*b2, *b3]);
    let sign = u16::from_be_bytes([*b4, *b5]);
    let scale = u16::from_be_bytes([*b6, *b7]);
    if sign == SIGN_NAN {
      return Ok(PgNumeric::NaN);
    }
    if digits_usize > CAP {
      return Err(E::from(PostgresError::VeryLargeDecimal.into()));
    }
    let mut array = [0i16; CAP];
    let (numbers, numbers_rest) = rest.as_chunks::<2>();
    let (true, []) = (numbers.len() == digits_usize, numbers_rest) else {
      return Err(E::from(
        DatabaseError::UnexpectedBufferSize {
          expected: digits.into(),
          received: numbers.len().try_into().unwrap_or(u32::MAX),
        }
        .into(),
      ));
    };
    for (elem, [i, j]) in array.iter_mut().zip(numbers) {
      *elem = i16::from_be_bytes([*i, *j]);
    }
    Ok(PgNumeric::Number {
      digits: ArrayVectorCopy::from_parts(array, Some(digits)),
      scale,
      sign: Sign::try_from(sign)?,
      weight,
    })
  }
}

impl<E> Encode<Postgres<E>> for PgNumeric
where
  E: From<crate::Error>,
{
  #[inline]
  fn encode(&self, ew: &mut PostgresEncodeWrapper<'_>) -> Result<(), E> {
    match self {
      PgNumeric::NaN => {
        let _ = ew.buffer().extend_from_copyable_slices([
          &0i16.to_be_bytes()[..],
          &0i16.to_be_bytes()[..],
          &SIGN_NAN.to_be_bytes()[..],
          &0u16.to_be_bytes()[..],
        ])?;
      }
      PgNumeric::Number { digits, scale, sign, weight } => {
        let len: i16 = digits.len().into();
        let _ = ew.buffer().extend_from_copyable_slices([
          &len.to_be_bytes()[..],
          &weight.to_be_bytes()[..],
          &u16::from(*sign).to_be_bytes()[..],
          &scale.to_be_bytes()[..],
        ])?;
        for digit in digits {
          ew.buffer().extend_from_copyable_slice(&digit.to_be_bytes())?;
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
      _ => {
        return Err(crate::Error::UnexpectedUint {
          identifier: "Sign".try_into().unwrap_or_default(),
          received: from,
        });
      }
    })
  }
}

#[cfg(test)]
mod tests {
  use crate::{
    codec::{Decode, Encode},
    collections::{ArrayVectorCopy, Vector},
    database::client::postgres::{
      Postgres, PostgresDecodeWrapper, PostgresEncodeWrapper, Ty,
      tys::pg_numeric::{CAP, PgNumeric, Sign},
    },
  };

  #[test]
  fn encodes_and_decodes() {
    let original = PgNumeric::Number {
      digits: {
        let mut arr = [0i16; CAP];
        arr[0] = 1234;
        ArrayVectorCopy::from_parts(arr, Some(1))
      },
      scale: 0,
      sign: Sign::Positive,
      weight: 0,
    };
    let mut buffer = Vector::new();
    let mut suffix_pusher = buffer.suffix_pusher();
    let mut ew = PostgresEncodeWrapper::new(suffix_pusher.inner_mut());
    <PgNumeric as Encode<Postgres<crate::Error>>>::encode(&original, &mut ew).unwrap();
    let mut dw = PostgresDecodeWrapper::new(suffix_pusher.curr(), "", Ty::Numeric);
    let decoded = <PgNumeric as Decode<Postgres<crate::Error>>>::decode(&mut dw).unwrap();
    match decoded {
      PgNumeric::Number { digits, scale, sign, weight } => {
        assert_eq!(digits.len(), 1);
        assert_eq!(digits.as_ref()[0], 1234);
        assert_eq!(scale, 0);
        assert_eq!(sign, Sign::Positive);
        assert_eq!(weight, 0);
      }
      PgNumeric::NaN => panic!(),
    }
  }
}
