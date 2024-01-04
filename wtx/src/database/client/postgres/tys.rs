macro_rules! test {
  ($name:ident, $ty:ty, $instance:expr) => {
    #[cfg(test)]
    #[test]
    fn $name() {
      let mut vec = &mut alloc::vec::Vec::new();
      let mut fbw = FilledBufferWriter::new(0, &mut vec);
      let instance: $ty = $instance;
      Encode::<Postgres<crate::Error>>::encode(&instance, &mut fbw).unwrap();
      let decoded: $ty =
        Decode::<Postgres<crate::Error>>::decode(&Value::new(fbw._curr_bytes())).unwrap();
      assert_eq!(instance, decoded);
    }
  };
}

#[cfg(feature = "arrayvec")]
mod arrayvec {
  use crate::{
    database::{
      client::postgres::{Postgres, Value},
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
    fn decode(input: &Value<'_>) -> Result<Self, E> {
      Ok(from_utf8_basic_rslt(input.bytes()).map_err(Into::into)?.try_into().map_err(Into::into)?)
    }
  }

  impl<E, const N: usize> Encode<Postgres<E>> for ArrayString<N>
  where
    E: From<crate::Error>,
  {
    #[inline]
    fn encode(&self, buffer: &mut FilledBufferWriter<'_>) -> Result<(), E> {
      buffer._extend_from_slice(self.as_str().as_bytes());
      Ok(())
    }
  }

  test!(array_string, ArrayString<4>, ArrayString::try_from("123").unwrap());
}

#[cfg(feature = "chrono")]
mod chrono {
  use crate::{
    database::{
      client::postgres::{Postgres, Value},
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
    fn decode(input: &Value<'_>) -> Result<Self, E> {
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
    fn encode(&self, buffer: &mut FilledBufferWriter<'_>) -> Result<(), E> {
      let time =
        match base().and_then(|el| self.naive_utc().signed_duration_since(el).num_microseconds()) {
          Some(time) => time,
          None => {
            return Err(crate::Error::UnexpectedValueFromBytes { expected: "timestamp" }.into())
          }
        };
      buffer._extend_from_slice(&time.to_be_bytes());
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
      client::postgres::{Postgres, Value},
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
    fn decode(input: &Value<'exec>) -> Result<Self, E> {
      Ok(input.bytes())
    }
  }

  impl<E> Encode<Postgres<E>> for &[u8]
  where
    E: From<crate::Error>,
  {
    #[inline]
    fn encode(&self, buffer: &mut FilledBufferWriter<'_>) -> Result<(), E> {
      buffer._extend_from_slice(self);
      Ok(())
    }
  }

  test!(bytes, &[u8], &[1, 2, 3, 4]);

  impl<'exec, E> Decode<'exec, Postgres<E>> for &'exec str
  where
    E: From<crate::Error>,
  {
    #[inline]
    fn decode(input: &Value<'exec>) -> Result<Self, E> {
      Ok(from_utf8_basic_rslt(input.bytes()).map_err(crate::Error::from)?)
    }
  }

  impl<E> Encode<Postgres<E>> for &str
  where
    E: From<crate::Error>,
  {
    #[inline]
    fn encode(&self, buffer: &mut FilledBufferWriter<'_>) -> Result<(), E> {
      buffer._extend_from_slice(self.as_bytes());
      Ok(())
    }
  }

  test!(str, &str, "1234");

  impl<E> Decode<'_, Postgres<E>> for String
  where
    E: From<crate::Error>,
  {
    #[inline]
    fn decode(input: &Value<'_>) -> Result<Self, E> {
      Ok(from_utf8_basic_rslt(input.bytes()).map_err(crate::Error::from)?.into())
    }
  }

  impl<E> Encode<Postgres<E>> for String
  where
    E: From<crate::Error>,
  {
    #[inline]
    fn encode(&self, buffer: &mut FilledBufferWriter<'_>) -> Result<(), E> {
      buffer._extend_from_slice(self.as_bytes());
      Ok(())
    }
  }

  test!(string, String, String::from("1234"));
}

mod primitives {
  use crate::{
    database::{
      client::postgres::{Postgres, Value},
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
    fn decode(input: &Value<'_>) -> Result<Self, E> {
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
    fn encode(&self, buffer: &mut FilledBufferWriter<'_>) -> Result<(), E> {
      buffer._extend_from_byte((*self).into());
      Ok(())
    }
  }

  test!(bool_true, bool, true);
  test!(bool_false, bool, false);

  macro_rules! impl_integer_from_array {
    ($instance:expr, [$($elem:ident),+], $signed:ident, $unsigned:ident) => {
      impl_primitive_from_array!($instance, [$($elem),+], $signed);

      impl<E> Decode<'_, Postgres<E>> for $unsigned
      where
        E: From<crate::Error>,
      {
        #[inline]
        fn decode(input: &Value<'_>) -> Result<Self, E> {
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
        fn encode(&self, buffer: &mut FilledBufferWriter<'_>) -> Result<(), E> {
          if *self >> mem::size_of::<$unsigned>().wrapping_sub(1) == 1 {
            return Err(E::from(crate::Error::InvalidPostgresUint));
          }
          buffer._extend_from_slice(&self.to_be_bytes());
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
        fn decode(input: &Value<'_>) -> Result<Self, E> {
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
        fn encode(&self, buffer: &mut FilledBufferWriter<'_>) -> Result<(), E> {
          buffer._extend_from_slice(&self.to_be_bytes());
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
