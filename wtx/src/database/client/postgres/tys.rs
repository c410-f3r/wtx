macro_rules! proptest {
  ($name:ident, $ty:ty) => {
    #[cfg(feature = "_proptest")]
    #[cfg(test)]
    #[test_strategy::proptest]
    fn $name(instance: $ty) {
      let mut vec = &mut alloc::vec::Vec::new();
      let mut fbw = FilledBufferWriter::new(0, &mut vec);
      Encode::<Postgres<crate::Error>>::encode(&instance, &mut fbw, &Ty::Any).unwrap();
      let decoded: $ty =
        Decode::<Postgres<crate::Error>>::decode(&DecodeValue::new(fbw._curr_bytes(), &Ty::Any))
          .unwrap();
      assert_eq!(instance, decoded);
      vec.clear();
    }
  };
}

macro_rules! test {
  ($name:ident, $ty:ty, $instance:expr) => {
    #[cfg(test)]
    #[test]
    fn $name() {
      let mut vec = &mut alloc::vec::Vec::new();
      let mut fbw = FilledBufferWriter::new(0, &mut vec);
      let instance: $ty = $instance;
      Encode::<Postgres<crate::Error>>::encode(
        &instance,
        &mut fbw,
        &crate::database::client::postgres::Ty::Any,
      )
      .unwrap();
      let decoded: $ty = Decode::<Postgres<crate::Error>>::decode(&DecodeValue::new(
        fbw._curr_bytes(),
        &crate::database::client::postgres::Ty::Any,
      ))
      .unwrap();
      assert_eq!(instance, decoded);
    }
  };
}

#[cfg(feature = "arrayvec")]
mod arrayvec {
  use crate::{
    database::{
      client::postgres::{DecodeValue, Postgres, Ty},
      Decode, Encode,
    },
    misc::{from_utf8_basic_rslt, FilledBufferWriter},
  };
  use arrayvec::ArrayString;

  impl<E, const N: usize> Decode<'_, Postgres<E>> for ArrayString<N>
  where
    E: From<crate::Error>,
  {
    #[inline]
    fn decode(input: &DecodeValue<'_>) -> Result<Self, E> {
      Ok(from_utf8_basic_rslt(input.bytes()).map_err(Into::into)?.try_into().map_err(Into::into)?)
    }
  }

  impl<E, const N: usize> Encode<Postgres<E>> for ArrayString<N>
  where
    E: From<crate::Error>,
  {
    #[inline]
    fn encode(&self, fbw: &mut FilledBufferWriter<'_>, _: &Ty) -> Result<(), E> {
      fbw._extend_from_slice(self.as_str().as_bytes());
      Ok(())
    }
  }

  test!(array_string, ArrayString<4>, ArrayString::try_from("123").unwrap());
}

#[cfg(feature = "chrono")]
mod chrono {
  use crate::{
    database::{
      client::postgres::{DecodeValue, Postgres, Ty},
      Decode, Encode,
    },
    misc::FilledBufferWriter,
  };
  use chrono::{DateTime, Duration, NaiveDate, NaiveDateTime, TimeZone, Utc};

  impl<E> Decode<'_, Postgres<E>> for DateTime<Utc>
  where
    E: From<crate::Error>,
  {
    #[inline]
    fn decode(input: &DecodeValue<'_>) -> Result<Self, E> {
      let rslt = || {
        let &[a, b, c, d, e, f, g, h] = input.bytes() else {
          return None;
        };
        let timestamp = i64::from_be_bytes([a, b, c, d, e, f, g, h]);
        Some(Utc.from_utc_datetime(&base()?.checked_add_signed(Duration::microseconds(timestamp))?))
      };
      rslt().ok_or_else(|| crate::Error::UnexpectedValueFromBytes { expected: "timestamp" }.into())
    }
  }

  impl<E> Encode<Postgres<E>> for DateTime<Utc>
  where
    E: From<crate::Error>,
  {
    #[inline]
    fn encode(&self, fbw: &mut FilledBufferWriter<'_>, _: &Ty) -> Result<(), E> {
      let time =
        match base().and_then(|el| self.naive_utc().signed_duration_since(el).num_microseconds()) {
          Some(time) => time,
          None => {
            return Err(crate::Error::UnexpectedValueFromBytes { expected: "timestamp" }.into())
          }
        };
      fbw._extend_from_slice(&time.to_be_bytes());
      Ok(())
    }
  }

  test!(datetime_utc, DateTime<Utc>, Utc.from_utc_datetime(&base().unwrap()));

  fn base() -> Option<NaiveDateTime> {
    NaiveDate::from_ymd_opt(2000, 1, 1).unwrap().and_hms_opt(0, 0, 0)
  }
}

mod collections {
  use crate::{
    database::{
      client::postgres::{DecodeValue, Postgres, Ty},
      Decode, Encode,
    },
    misc::{from_utf8_basic_rslt, FilledBufferWriter},
  };
  use alloc::string::String;

  impl<'exec, E> Decode<'exec, Postgres<E>> for &'exec [u8]
  where
    E: From<crate::Error>,
  {
    #[inline]
    fn decode(input: &DecodeValue<'exec>) -> Result<Self, E> {
      Ok(input.bytes())
    }
  }

  impl<E> Encode<Postgres<E>> for &[u8]
  where
    E: From<crate::Error>,
  {
    #[inline]
    fn encode(&self, fbw: &mut FilledBufferWriter<'_>, _: &Ty) -> Result<(), E> {
      fbw._extend_from_slice(self);
      Ok(())
    }
  }

  test!(bytes, &[u8], &[1, 2, 3, 4]);

  impl<'exec, E> Decode<'exec, Postgres<E>> for &'exec str
  where
    E: From<crate::Error>,
  {
    #[inline]
    fn decode(input: &DecodeValue<'exec>) -> Result<Self, E> {
      Ok(from_utf8_basic_rslt(input.bytes()).map_err(crate::Error::from)?)
    }
  }

  impl<E> Encode<Postgres<E>> for &str
  where
    E: From<crate::Error>,
  {
    #[inline]
    fn encode(&self, fbw: &mut FilledBufferWriter<'_>, _: &Ty) -> Result<(), E> {
      fbw._extend_from_slice(self.as_bytes());
      Ok(())
    }
  }

  test!(str, &str, "1234");

  impl<E> Decode<'_, Postgres<E>> for String
  where
    E: From<crate::Error>,
  {
    #[inline]
    fn decode(input: &DecodeValue<'_>) -> Result<Self, E> {
      Ok(from_utf8_basic_rslt(input.bytes()).map_err(crate::Error::from)?.into())
    }
  }

  impl<E> Encode<Postgres<E>> for String
  where
    E: From<crate::Error>,
  {
    #[inline]
    fn encode(&self, fbw: &mut FilledBufferWriter<'_>, _: &Ty) -> Result<(), E> {
      fbw._extend_from_slice(self.as_bytes());
      Ok(())
    }
  }

  proptest!(string, String);
}

mod pg_numeric {
  use crate::{
    database::{
      client::postgres::{DecodeValue, Postgres, Ty},
      Decode, Encode,
    },
    misc::FilledBufferWriter,
  };
  use arrayvec::ArrayVec;

  const DIGITS_CAP: usize = 64;
  const SIGN_NAN: u16 = 0xC000;
  const SIGN_NEG: u16 = 0x4000;
  const SIGN_POS: u16 = 0x0000;

  pub(crate) enum PgNumeric {
    NotANumber,
    Number { digits: ArrayVec<i16, DIGITS_CAP>, scale: u16, sign: Sign, weight: i16 },
  }

  impl<E> Decode<'_, Postgres<E>> for PgNumeric
  where
    E: From<crate::Error>,
  {
    #[inline]
    fn decode(input: &DecodeValue<'_>) -> Result<Self, E> {
      let [a, b, c, d, e, f, g, h, rest @ ..] = input.bytes() else {
        return Err(
          crate::Error::UnexpectedBufferSize {
            expected: 8,
            received: input.bytes().len().try_into().map_err(Into::into)?,
          }
          .into(),
        );
      };
      let digits = u16::from_be_bytes([*a, *b]);
      let digits_usize = usize::from(digits);
      let weight = i16::from_be_bytes([*c, *d]);
      let sign = u16::from_be_bytes([*e, *f]);
      let scale = u16::from_be_bytes([*g, *h]);
      let mut curr_slice = rest;
      Ok(if sign == SIGN_NAN {
        PgNumeric::NotANumber
      } else {
        if digits_usize > DIGITS_CAP || digits_usize > 0x7FFF {
          return Err(crate::Error::VeryLargeDecimal.into());
        }
        let mut fbw = [0i16; DIGITS_CAP];
        for elem in fbw.iter_mut().take(digits_usize) {
          let [i, j, local_rest @ ..] = curr_slice else {
            break;
          };
          *elem = i16::from_be_bytes([*i, *j]);
          curr_slice = local_rest;
        }
        PgNumeric::Number {
          digits: fbw.into_iter().take(digits_usize).collect(),
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
    fn encode(&self, fbw: &mut FilledBufferWriter<'_>, _: &Ty) -> Result<(), E> {
      match self {
        PgNumeric::NotANumber => {
          fbw._extend_from_slice(&0i16.to_be_bytes());
          fbw._extend_from_slice(&0i16.to_be_bytes());
          fbw._extend_from_slice(&SIGN_NAN.to_be_bytes());
          fbw._extend_from_slice(&0u16.to_be_bytes());
        }
        PgNumeric::Number { digits, scale, sign, weight } => {
          let len: i16 = digits.len().try_into().map_err(Into::into)?;
          fbw._extend_from_slice(&len.to_be_bytes());
          fbw._extend_from_slice(&weight.to_be_bytes());
          fbw._extend_from_slice(&u16::from(*sign).to_be_bytes());
          fbw._extend_from_slice(&scale.to_be_bytes());
          for digit in digits {
            fbw._extend_from_slice(&digit.to_be_bytes());
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
        SIGN_NAN => return Err(crate::Error::DecimalCanNotBeConvertedFromNaN),
        SIGN_NEG => Self::Negative,
        SIGN_POS => Self::Positive,
        _ => return Err(crate::Error::UnexpectedUint { received: from.into() }),
      })
    }
  }
}

mod primitives {
  use crate::{
    database::{
      client::postgres::{DecodeValue, Postgres, Ty},
      Decode, Encode,
    },
    misc::FilledBufferWriter,
  };
  use core::mem;

  impl<E> Decode<'_, Postgres<E>> for bool
  where
    E: From<crate::Error>,
  {
    #[inline]
    fn decode(input: &DecodeValue<'_>) -> Result<Self, E> {
      let &[byte] = input.bytes() else {
        return Err(
          crate::Error::UnexpectedBufferSize {
            expected: 1,
            received: input.bytes().len().try_into().map_err(Into::into)?,
          }
          .into(),
        );
      };
      Ok(byte != 0)
    }
  }

  impl<E> Encode<Postgres<E>> for bool
  where
    E: From<crate::Error>,
  {
    #[inline]
    fn encode(&self, fbw: &mut FilledBufferWriter<'_>, _: &Ty) -> Result<(), E> {
      fbw._extend_from_byte((*self).into());
      Ok(())
    }
  }

  proptest!(bool_true, bool);
  proptest!(bool_false, bool);

  macro_rules! impl_integer_from_array {
    ($instance:expr, [$($elem:ident),+], $signed:ident, $unsigned:ident) => {
      impl_primitive_from_array!($instance, [$($elem),+], $signed);

      impl<E> Decode<'_, Postgres<E>> for $unsigned
      where
        E: From<crate::Error>,
      {
        #[inline]
        fn decode(input: &DecodeValue<'_>) -> Result<Self, E> {
          Ok(
            <$signed as Decode::<Postgres<E>>>::decode(input)?
              .try_into()
              .map_err(|_err| crate::Error::InvalidPostgresUint)?
          )
        }
      }

      impl<E> Encode<Postgres<E>> for $unsigned
      where
        E: From<crate::Error>,
      {
        #[inline]
        fn encode(&self, fbw: &mut FilledBufferWriter<'_>, _: &Ty) -> Result<(), E> {
          if *self >> mem::size_of::<$unsigned>().wrapping_sub(1) == 1 {
            return Err(E::from(crate::Error::InvalidPostgresUint));
          }
          fbw._extend_from_slice(&self.to_be_bytes());
          Ok(())
        }
      }

      test!($unsigned, $unsigned, $instance);
    };
  }

  macro_rules! impl_primitive_from_array {
    ($instance:expr, [$($elem:ident),+], $ty:ident) => {
      impl<E> Decode<'_, Postgres<E>> for $ty
      where
        E: From<crate::Error>,
      {
        #[inline]
        fn decode(input: &DecodeValue<'_>) -> Result<Self, E> {
          if let &[$($elem),+] = input.bytes() {
            return Ok(<$ty>::from_be_bytes([$($elem),+]));
          }
          Err(crate::Error::UnexpectedBufferSize {
            expected: mem::size_of::<$ty>().try_into().map_err(Into::into)?,
            received: input.bytes().len().try_into().map_err(Into::into)?,
          }.into())
        }
      }

      impl<E> Encode<Postgres<E>> for $ty
      where
        E: From<crate::Error>,
      {
        #[inline]
        fn encode(&self, fbw: &mut FilledBufferWriter<'_>, _: &Ty) -> Result<(), E> {
          fbw._extend_from_slice(&self.to_be_bytes());
          Ok(())
        }
      }

      test!($ty, $ty, $instance);
    }
  }

  impl_integer_from_array!(37, [a], i8, u8);
  impl_integer_from_array!(37, [a, b], i16, u16);
  impl_integer_from_array!(37, [a, b, c, d], i32, u32);
  impl_integer_from_array!(37, [a, b, c, d, e, f, g, h], i64, u64);

  impl_primitive_from_array!(37.0, [a, b, c, d], f32);
  impl_primitive_from_array!(37.0, [a, b, c, d, e, f, g, h], f64);
}

#[cfg(feature = "rust_decimal")]
mod rust_decimal {
  use crate::{
    database::{
      client::postgres::{
        tys::pg_numeric::{PgNumeric, Sign},
        DecodeValue, Postgres, Ty,
      },
      Decode, Encode,
    },
    misc::FilledBufferWriter,
  };
  use arrayvec::ArrayVec;
  use rust_decimal::{Decimal, MathematicalOps};

  impl<E> Decode<'_, Postgres<E>> for Decimal
  where
    E: From<crate::Error>,
  {
    #[inline]
    fn decode(input: &DecodeValue<'_>) -> Result<Self, E> {
      let pg_numeric = PgNumeric::decode(input)?;
      let (digits, sign, mut weight, scale) = match pg_numeric {
        PgNumeric::NotANumber => {
          return Err(crate::Error::DecimalCanNotBeConvertedFromNaN.into());
        }
        PgNumeric::Number { digits, sign, weight, scale } => (digits, sign, weight, scale),
      };
      if digits.is_empty() {
        return Ok(0u64.into());
      }
      let mut value = Decimal::ZERO;
      for digit in digits {
        let mut operations = || {
          let mul = Decimal::from(10_000u16).checked_powi(weight.into())?;
          let part = Decimal::from(digit).checked_mul(mul)?;
          value = value.checked_add(part)?;
          weight = weight.checked_sub(1)?;
          Some(())
        };
        operations().ok_or_else(|| crate::Error::OutOfBoundsArithmetic.into())?;
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
    fn encode(&self, fbw: &mut FilledBufferWriter<'_>, value: &Ty) -> Result<(), E> {
      if self.is_zero() {
        let rslt = PgNumeric::Number {
          digits: ArrayVec::default(),
          scale: 0,
          sign: Sign::Positive,
          weight: 0,
        };
        rslt.encode(fbw, value)?;
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

      let mut digits = ArrayVec::new();
      while mantissa != 0 {
        digits.push((mantissa % 10_000) as i16);
        mantissa /= 10_000;
      }
      digits.reverse();

      let after_decimal = usize::from(scale.wrapping_add(3) / 4);
      let weight = digits.len().wrapping_sub(after_decimal).wrapping_sub(1) as i16;

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
      rslt.encode(fbw, value)?;
      Ok(())
    }
  }

  proptest!(rust_decimal, Decimal);
}
