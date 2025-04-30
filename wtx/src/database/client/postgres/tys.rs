macro_rules! kani {
  ($name:ident, $ty:ty) => {
    #[cfg(kani)]
    #[kani::proof]
    fn $name(instance: $ty) {
      let mut vec = &mut crate::misc::FilledBuffer::_new();
      {
        let mut sw = crate::misc::FilledBufferWriter::new(0, &mut vec);
        let mut ew = EncodeValue::new(&mut sw);
        Encode::<Postgres<crate::Error>>::encode(&instance, &mut ew).unwrap();
        let decoded: $ty = Decode::<Postgres<crate::Error>>::decode(&DecodeValue::new(
          ew.sw()._curr_bytes(),
          crate::database::client::postgres::Ty::Any,
        ))
        .unwrap();
        assert_eq!(instance, decoded);
      }
      vec._clear();
    }
  };
}

macro_rules! test {
  ($name:ident, $ty:ty, $instance:expr) => {
    #[cfg(test)]
    #[test]
    fn $name() {
      let vec = &mut crate::misc::FilledBuffer::_new();
      let mut sw = crate::misc::SuffixWriter::_new(0, vec._vector_mut());
      let mut ew = EncodeWrapper::new(&mut sw);
      let instance: $ty = $instance;
      Encode::<Postgres<crate::Error>>::encode(&instance, &mut (), &mut ew).unwrap();
      let decoded: $ty = Decode::<Postgres<crate::Error>>::decode(
        &mut (),
        &mut DecodeWrapper::new(
          ew.buffer()._curr_bytes(),
          crate::database::client::postgres::Ty::Any,
        ),
      )
      .unwrap();
      assert_eq!(instance, decoded);
    }
  };
}

#[cfg(feature = "chrono")]
mod chrono;
#[cfg(feature = "rust_decimal")]
mod rust_decimal;
#[cfg(feature = "serde_json")]
mod serde_json;
#[cfg(feature = "uuid")]
mod uuid;

mod array {
  use crate::{
    collection::ArrayString,
    database::{
      Typed,
      client::postgres::{DecodeWrapper, EncodeWrapper, Postgres, Ty},
    },
    misc::{Decode, Encode, from_utf8_basic},
  };

  impl<E, const N: usize> Decode<'_, Postgres<E>> for ArrayString<N>
  where
    E: From<crate::Error>,
  {
    #[inline]
    fn decode(_: &mut (), dw: &mut DecodeWrapper<'_>) -> Result<Self, E> {
      Ok(from_utf8_basic(dw.bytes()).map_err(Into::into)?.try_into()?)
    }
  }
  impl<E, const N: usize> Encode<Postgres<E>> for ArrayString<N>
  where
    E: From<crate::Error>,
  {
    #[inline]
    fn encode(&self, _: &mut (), ew: &mut EncodeWrapper<'_, '_>) -> Result<(), E> {
      ew.buffer().extend_from_slice(self.as_str().as_bytes())?;
      Ok(())
    }
  }
  impl<E, const N: usize> Typed<Postgres<E>> for ArrayString<N>
  where
    E: From<crate::Error>,
  {
    #[inline]
    fn runtime_ty(&self) -> Option<Ty> {
      <Self as Typed<Postgres<E>>>::static_ty()
    }

    #[inline]
    fn static_ty() -> Option<Ty> {
      Some(Ty::Text)
    }
  }

  test!(array_string, ArrayString<4>, ArrayString::try_from("123").unwrap());
}

mod collections {
  use crate::{
    database::{
      Typed,
      client::postgres::{DecodeWrapper, EncodeWrapper, Postgres, Ty},
    },
    misc::{Decode, Encode, from_utf8_basic},
  };
  use alloc::string::String;

  // &[u8]

  impl<'exec, E> Decode<'exec, Postgres<E>> for &'exec [u8]
  where
    E: From<crate::Error>,
  {
    #[inline]
    fn decode(_: &mut (), dw: &mut DecodeWrapper<'exec>) -> Result<Self, E> {
      Ok(dw.bytes())
    }
  }
  impl<E> Encode<Postgres<E>> for &[u8]
  where
    E: From<crate::Error>,
  {
    #[inline]
    fn encode(&self, _: &mut (), ew: &mut EncodeWrapper<'_, '_>) -> Result<(), E> {
      ew.buffer().extend_from_slice(self)?;
      Ok(())
    }
  }
  impl<E> Typed<Postgres<E>> for &[u8]
  where
    E: From<crate::Error>,
  {
    #[inline]
    fn runtime_ty(&self) -> Option<Ty> {
      <Self as Typed<Postgres<E>>>::static_ty()
    }

    #[inline]
    fn static_ty() -> Option<Ty> {
      Some(Ty::ByteaArray)
    }
  }
  test!(bytes, &[u8], &[1, 2, 3, 4]);

  // str

  impl<E> Encode<Postgres<E>> for str
  where
    E: From<crate::Error>,
  {
    #[inline]
    fn encode(&self, _: &mut (), ew: &mut EncodeWrapper<'_, '_>) -> Result<(), E> {
      ew.buffer().extend_from_slice(self.as_bytes())?;
      Ok(())
    }
  }
  impl<E> Typed<Postgres<E>> for str
  where
    E: From<crate::Error>,
  {
    #[inline]
    fn runtime_ty(&self) -> Option<Ty> {
      <&Self as Typed<Postgres<E>>>::static_ty()
    }
  }

  // &str

  impl<'exec, E> Decode<'exec, Postgres<E>> for &'exec str
  where
    E: From<crate::Error>,
  {
    #[inline]
    fn decode(_: &mut (), dw: &mut DecodeWrapper<'exec>) -> Result<Self, E> {
      Ok(from_utf8_basic(dw.bytes()).map_err(crate::Error::from)?)
    }
  }
  impl<E> Encode<Postgres<E>> for &str
  where
    E: From<crate::Error>,
  {
    #[inline]
    fn encode(&self, _: &mut (), ew: &mut EncodeWrapper<'_, '_>) -> Result<(), E> {
      ew.buffer().extend_from_slice(self.as_bytes())?;
      Ok(())
    }
  }
  impl<E> Typed<Postgres<E>> for &str
  where
    E: From<crate::Error>,
  {
    #[inline]
    fn runtime_ty(&self) -> Option<Ty> {
      <Self as Typed<Postgres<E>>>::static_ty()
    }

    #[inline]
    fn static_ty() -> Option<Ty> {
      Some(Ty::Text)
    }
  }
  test!(str, &str, "1234");

  // String

  impl<E> Decode<'_, Postgres<E>> for String
  where
    E: From<crate::Error>,
  {
    #[inline]
    fn decode(_: &mut (), dw: &mut DecodeWrapper<'_>) -> Result<Self, E> {
      match from_utf8_basic(dw.bytes()).map_err(crate::Error::from) {
        Ok(elem) => Ok(elem.into()),
        Err(err) => Err(err.into()),
      }
    }
  }
  impl<E> Encode<Postgres<E>> for String
  where
    E: From<crate::Error>,
  {
    #[inline]
    fn encode(&self, _: &mut (), ew: &mut EncodeWrapper<'_, '_>) -> Result<(), E> {
      ew.buffer().extend_from_slice(self.as_bytes())?;
      Ok(())
    }
  }
  impl<E> Typed<Postgres<E>> for String
  where
    E: From<crate::Error>,
  {
    #[inline]
    fn runtime_ty(&self) -> Option<Ty> {
      <Self as Typed<Postgres<E>>>::static_ty()
    }

    #[inline]
    fn static_ty() -> Option<Ty> {
      Some(Ty::Text)
    }
  }
  kani!(string, String);
}

mod ip {
  use crate::{
    database::{
      Typed,
      client::postgres::{DecodeWrapper, EncodeWrapper, Postgres, PostgresError, Ty},
    },
    misc::{Decode, Encode},
  };
  use core::net::{IpAddr, Ipv4Addr, Ipv6Addr};

  impl<'exec, E> Decode<'exec, Postgres<E>> for IpAddr
  where
    E: From<crate::Error>,
  {
    #[inline]
    fn decode(aux: &mut (), dw: &mut DecodeWrapper<'exec>) -> Result<Self, E> {
      Ok(match dw.bytes() {
        [2, ..] => IpAddr::V4(Ipv4Addr::decode(aux, dw)?),
        [3, ..] => IpAddr::V6(Ipv6Addr::decode(aux, dw)?),
        _ => return Err(E::from(PostgresError::InvalidIpFormat.into())),
      })
    }
  }
  impl<E> Encode<Postgres<E>> for IpAddr
  where
    E: From<crate::Error>,
  {
    #[inline]
    fn encode(&self, aux: &mut (), ew: &mut EncodeWrapper<'_, '_>) -> Result<(), E> {
      match self {
        IpAddr::V4(ipv4_addr) => ipv4_addr.encode(aux, ew),
        IpAddr::V6(ipv6_addr) => ipv6_addr.encode(aux, ew),
      }
    }
  }
  impl<E> Typed<Postgres<E>> for IpAddr
  where
    E: From<crate::Error>,
  {
    #[inline]
    fn runtime_ty(&self) -> Option<Ty> {
      <Self as Typed<Postgres<E>>>::static_ty()
    }

    #[inline]
    fn static_ty() -> Option<Ty> {
      Some(Ty::Inet)
    }
  }
  test!(ipaddr_v4, IpAddr, IpAddr::V4(Ipv4Addr::new(1, 2, 3, 4)));
  test!(ipaddr_v6, IpAddr, IpAddr::V6(Ipv6Addr::new(1, 2, 3, 4, 5, 6, 7, 8)));

  impl<'exec, E> Decode<'exec, Postgres<E>> for Ipv4Addr
  where
    E: From<crate::Error>,
  {
    #[inline]
    fn decode(_: &mut (), dw: &mut DecodeWrapper<'exec>) -> Result<Self, E> {
      let [2, 32, 0, 4, e, f, g, h] = dw.bytes() else {
        return Err(E::from(PostgresError::InvalidIpFormat.into()));
      };
      Ok(Ipv4Addr::from([*e, *f, *g, *h]))
    }
  }
  impl<E> Encode<Postgres<E>> for Ipv4Addr
  where
    E: From<crate::Error>,
  {
    #[inline]
    fn encode(&self, _: &mut (), ew: &mut EncodeWrapper<'_, '_>) -> Result<(), E> {
      ew.buffer()._extend_from_slices([&[2, 32, 0, 4][..], &self.octets()])?;
      Ok(())
    }
  }
  impl<E> Typed<Postgres<E>> for Ipv4Addr
  where
    E: From<crate::Error>,
  {
    #[inline]
    fn runtime_ty(&self) -> Option<Ty> {
      <Self as Typed<Postgres<E>>>::static_ty()
    }

    #[inline]
    fn static_ty() -> Option<Ty> {
      Some(Ty::Inet)
    }
  }
  test!(ipv4, Ipv4Addr, Ipv4Addr::new(1, 2, 3, 4));

  impl<'exec, E> Decode<'exec, Postgres<E>> for Ipv6Addr
  where
    E: From<crate::Error>,
  {
    #[inline]
    fn decode(_: &mut (), dw: &mut DecodeWrapper<'exec>) -> Result<Self, E> {
      let [3, 128, 0, 16, e, f, g, h, i, j, k, l, m, n, o, p, q, r, s, t] = dw.bytes() else {
        return Err(E::from(PostgresError::InvalidIpFormat.into()));
      };
      Ok(Ipv6Addr::from([*e, *f, *g, *h, *i, *j, *k, *l, *m, *n, *o, *p, *q, *r, *s, *t]))
    }
  }
  impl<E> Encode<Postgres<E>> for Ipv6Addr
  where
    E: From<crate::Error>,
  {
    #[inline]
    fn encode(&self, _: &mut (), ew: &mut EncodeWrapper<'_, '_>) -> Result<(), E> {
      ew.buffer()._extend_from_slices([&[3, 128, 0, 16][..], &self.octets()])?;
      Ok(())
    }
  }
  impl<E> Typed<Postgres<E>> for Ipv6Addr
  where
    E: From<crate::Error>,
  {
    #[inline]
    fn runtime_ty(&self) -> Option<Ty> {
      <Self as Typed<Postgres<E>>>::static_ty()
    }

    #[inline]
    fn static_ty() -> Option<Ty> {
      Some(Ty::Inet)
    }
  }
  test!(ipv6, Ipv6Addr, Ipv6Addr::new(1, 2, 3, 4, 5, 6, 7, 8));
}

#[cfg(feature = "rust_decimal")]
mod pg_numeric {
  use crate::{
    collection::ArrayVector,
    database::{
      DatabaseError,
      client::postgres::{DecodeWrapper, EncodeWrapper, Postgres, PostgresError},
    },
    misc::{Decode, Encode, Usize},
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
}

mod primitives {
  use crate::{
    database::{
      DatabaseError, Typed,
      client::postgres::{DecodeWrapper, EncodeWrapper, Postgres, PostgresError, Ty},
    },
    misc::{Decode, Encode, Usize},
  };

  // bool

  impl<E> Decode<'_, Postgres<E>> for bool
  where
    E: From<crate::Error>,
  {
    #[inline]
    fn decode(_: &mut (), dw: &mut DecodeWrapper<'_>) -> Result<Self, E> {
      let &[byte] = dw.bytes() else {
        return Err(E::from(
          DatabaseError::UnexpectedBufferSize {
            expected: 1,
            received: Usize::from(dw.bytes().len()).into_u64().try_into().unwrap_or(u32::MAX),
          }
          .into(),
        ));
      };
      Ok(byte != 0)
    }
  }
  impl<E> Encode<Postgres<E>> for bool
  where
    E: From<crate::Error>,
  {
    #[inline]
    fn encode(&self, _: &mut (), ew: &mut EncodeWrapper<'_, '_>) -> Result<(), E> {
      ew.buffer()._extend_from_byte((*self).into())?;
      Ok(())
    }
  }
  impl<E> Typed<Postgres<E>> for bool
  where
    E: From<crate::Error>,
  {
    #[inline]
    fn runtime_ty(&self) -> Option<Ty> {
      <Self as Typed<Postgres<E>>>::static_ty()
    }

    #[inline]
    fn static_ty() -> Option<Ty> {
      Some(Ty::Bool)
    }
  }

  kani!(bool_true, bool);
  kani!(bool_false, bool);

  macro_rules! impl_integer_from_array {
    ($instance:expr, [$($elem:ident),+], ($signed:ident, $signed_pg_ty:expr), ($unsigned:ident, $unsigned_pg_ty:expr)) => {
      impl_primitive_from_array!($instance, [$($elem),+], $signed, $signed_pg_ty);

      impl<E> Decode<'_, Postgres<E>> for $unsigned
      where
        E: From<crate::Error>,
      {
        #[inline]
        fn decode(aux: &mut (), dw: &mut DecodeWrapper<'_>) -> Result<Self, E> {
          <$signed as Decode::<Postgres<E>>>::decode(aux, dw)?
            .try_into()
            .map_err(|_err| E::from(PostgresError::InvalidPostgresUint.into()))
        }
      }
      impl<E> Encode<Postgres<E>> for $unsigned
      where
        E: From<crate::Error>,
      {
        #[inline]
        fn encode(&self, _: &mut (), ew: &mut EncodeWrapper<'_, '_>) -> Result<(), E> {
          if *self >> const { $unsigned::BITS - 1 } == 1 {
            return Err(E::from(PostgresError::InvalidPostgresUint.into()));
          }
          ew.buffer().extend_from_slice(&self.to_be_bytes())?;
          Ok(())
        }
      }
      impl<E> Typed<Postgres<E>> for $unsigned
      where
        E: From<crate::Error>
      {
        #[inline]
        fn runtime_ty(&self) -> Option<Ty> {
          <Self as Typed<Postgres<E>>>::static_ty()
        }

        #[inline]
        fn static_ty() -> Option<Ty> {
          Some($unsigned_pg_ty)
        }
      }

      test!($unsigned, $unsigned, $instance);
    };
  }

  macro_rules! impl_primitive_from_array {
    ($instance:expr, [$($elem:ident),+], $ty:ident, $pg_ty:expr) => {
      impl<E> Decode<'_, Postgres<E>> for $ty
      where
        E: From<crate::Error>,
      {
        #[inline]
        fn decode(_: &mut (), dw: &mut DecodeWrapper<'_>) -> Result<Self, E> {
          if let &[$($elem,)+] = dw.bytes() {
            return Ok(<Self>::from_be_bytes([$($elem),+]));
          }
          Err(E::from(DatabaseError::UnexpectedBufferSize {
            expected: Usize::from(size_of::<$ty>()).into_u64().try_into().unwrap_or(u32::MAX),
            received: Usize::from(dw.bytes().len()).into_u64().try_into().unwrap_or(u32::MAX)
          }.into()))
        }
      }

      impl<E> Encode<Postgres<E>> for $ty
      where
        E: From<crate::Error>,
      {
        #[inline]
        fn encode(&self, _: &mut (), ew: &mut EncodeWrapper<'_, '_>) -> Result<(), E> {
          ew.buffer().extend_from_slice(&self.to_be_bytes())?;
          Ok(())
        }
      }

      impl<E> Typed<Postgres<E>> for $ty
      where
        E: From<crate::Error>
      {
        #[inline]
        fn runtime_ty(&self) -> Option<Ty> {
          <Self as Typed<Postgres<E>>>::static_ty()
        }

        #[inline]
        fn static_ty() -> Option<Ty> {
          Some($pg_ty)
        }
      }

      test!($ty, $ty, $instance);
    }
  }

  impl_integer_from_array!(37, [a], (i8, Ty::Char), (u8, Ty::Bytea));
  impl_integer_from_array!(37, [a, b], (i16, Ty::Int2), (u16, Ty::Int2));
  impl_integer_from_array!(37, [a, b, c, d], (i32, Ty::Int4), (u32, Ty::Int4));
  impl_integer_from_array!(37, [a, b, c, d, e, f, g, h], (i64, Ty::Int8), (u64, Ty::Int8));

  impl_primitive_from_array!(37.0, [a, b, c, d], f32, Ty::Float4);
  impl_primitive_from_array!(37.0, [a, b, c, d, e, f, g, h], f64, Ty::Float8);
}
