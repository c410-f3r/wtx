macro_rules! kani {
  ($name:ident, $ty:ty) => {
    #[cfg(kani)]
    #[kani::proof]
    fn $name(instance: $ty) {
      let mut vec = &mut crate::misc::FilledBuffer::_new();
      {
        let mut sw = crate::misc::FilledBufferWriter::new(0, &mut vec);
        let mut ew = EncodeValue::new(&mut sw);
        Encode::<Mysql<crate::Error>>::encode(&instance, &mut ew).unwrap();
        let decoded: $ty = Decode::<Mysql<crate::Error>>::decode(&DecodeValue::new(
          ew.sw()._curr_bytes(),
          crate::database::client::mysql::Ty::Null,
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
      let mut vec = crate::misc::Vector::new();
      let mut ew = EncodeWrapper::new(&mut vec);
      let instance: $ty = $instance;
      Encode::<Mysql<crate::Error>>::encode(&instance, &mut (), &mut ew).unwrap();
      let decoded: $ty = Decode::<Mysql<crate::Error>>::decode(
        &mut (),
        &mut DecodeWrapper::new(ew.sw(), crate::database::client::mysql::Ty::Tiny),
      )
      .unwrap();
      assert_eq!(instance, decoded);
    }
  };
}

#[cfg(feature = "chrono")]
mod chrono;

mod collections {
  use crate::{
    database::{
      Typed,
      client::mysql::{DecodeWrapper, EncodeWrapper, Mysql, Ty},
    },
    misc::{ArrayString, Decode, Encode, from_utf8_basic},
  };
  use alloc::string::String;

  // &[u8]

  impl<'exec, E> Decode<'exec, Mysql<E>> for &'exec [u8]
  where
    E: From<crate::Error>,
  {
    #[inline]
    fn decode(_: &mut (), dv: &mut DecodeWrapper<'exec>) -> Result<Self, E> {
      Ok(dv.bytes())
    }
  }
  impl<E> Encode<Mysql<E>> for &[u8]
  where
    E: From<crate::Error>,
  {
    #[inline]
    fn encode(&self, _: &mut (), ew: &mut EncodeWrapper<'_>) -> Result<(), E> {
      ew.sw().extend_from_copyable_slice(self)?;
      Ok(())
    }
  }
  impl<E> Typed<Mysql<E>> for &[u8]
  where
    E: From<crate::Error>,
  {
    const TY: Ty = Ty::Blob;
  }
  test!(bytes, &[u8], &[1, 2, 3, 4]);

  // String

  impl<E, const N: usize> Decode<'_, Mysql<E>> for ArrayString<N>
  where
    E: From<crate::Error>,
  {
    #[inline]
    fn decode(_: &mut (), dv: &mut DecodeWrapper<'_>) -> Result<Self, E> {
      Ok(from_utf8_basic(dv.bytes()).map_err(Into::into)?.try_into()?)
    }
  }
  impl<E, const N: usize> Encode<Mysql<E>> for ArrayString<N>
  where
    E: From<crate::Error>,
  {
    #[inline]
    fn encode(&self, _: &mut (), ew: &mut EncodeWrapper<'_>) -> Result<(), E> {
      ew.sw().extend_from_copyable_slice(self.as_str().as_bytes())?;
      Ok(())
    }
  }
  impl<E, const N: usize> Typed<Mysql<E>> for ArrayString<N>
  where
    E: From<crate::Error>,
  {
    const TY: Ty = Ty::VarString;
  }
  test!(array_string, ArrayString<4>, ArrayString::try_from("123").unwrap());

  impl<'exec, E> Decode<'exec, Mysql<E>> for &'exec str
  where
    E: From<crate::Error>,
  {
    #[inline]
    fn decode(_: &mut (), dv: &mut DecodeWrapper<'exec>) -> Result<Self, E> {
      Ok(from_utf8_basic(dv.bytes()).map_err(crate::Error::from)?)
    }
  }
  impl<E> Encode<Mysql<E>> for &str
  where
    E: From<crate::Error>,
  {
    #[inline]
    fn encode(&self, _: &mut (), ew: &mut EncodeWrapper<'_>) -> Result<(), E> {
      ew.sw().extend_from_copyable_slice(self.as_bytes())?;
      Ok(())
    }
  }
  impl<E> Typed<Mysql<E>> for &str
  where
    E: From<crate::Error>,
  {
    const TY: Ty = Ty::VarString;
  }
  test!(str, &str, "1234");

  // String

  impl<E> Decode<'_, Mysql<E>> for String
  where
    E: From<crate::Error>,
  {
    #[inline]
    fn decode(_: &mut (), dv: &mut DecodeWrapper<'_>) -> Result<Self, E> {
      match from_utf8_basic(dv.bytes()).map_err(crate::Error::from) {
        Ok(elem) => Ok(elem.into()),
        Err(err) => Err(err.into()),
      }
    }
  }
  impl<E> Encode<Mysql<E>> for String
  where
    E: From<crate::Error>,
  {
    #[inline]
    fn encode(&self, _: &mut (), ew: &mut EncodeWrapper<'_>) -> Result<(), E> {
      ew.sw().extend_from_copyable_slice(self.as_bytes())?;
      Ok(())
    }
  }
  impl<E> Typed<Mysql<E>> for String
  where
    E: From<crate::Error>,
  {
    const TY: Ty = Ty::VarString;
  }
  kani!(string, String);
}

mod primitives {
  use crate::{
    database::{
      DatabaseError, Typed,
      client::mysql::{DecodeWrapper, EncodeWrapper, Mysql, Ty},
    },
    misc::{Decode, Encode, Usize},
  };

  // bool

  impl<E> Decode<'_, Mysql<E>> for bool
  where
    E: From<crate::Error>,
  {
    #[inline]
    fn decode(_: &mut (), dv: &mut DecodeWrapper<'_>) -> Result<Self, E> {
      let &[byte] = dv.bytes() else {
        return Err(E::from(
          DatabaseError::UnexpectedBufferSize {
            expected: 1,
            received: Usize::from(dv.bytes().len()).into_u32().unwrap_or(u32::MAX),
          }
          .into(),
        ));
      };
      Ok(byte != 0)
    }
  }
  impl<E> Encode<Mysql<E>> for bool
  where
    E: From<crate::Error>,
  {
    #[inline]
    fn encode(&self, _: &mut (), ew: &mut EncodeWrapper<'_>) -> Result<(), E> {
      ew.sw().push((*self).into())?;
      Ok(())
    }
  }
  impl<E> Typed<Mysql<E>> for bool
  where
    E: From<crate::Error>,
  {
    const TY: Ty = Ty::Tiny;
  }

  macro_rules! impl_integer_from_array {
      ($instance:expr, [$($elem:ident),+], ($signed:ident, $signed_pg_ty:expr), ($unsigned:ident, $unsigned_pg_ty:expr)) => {
        impl_primitive_from_array!($instance, [$($elem),+], $signed, $signed_pg_ty);

        impl<E> Decode<'_, Mysql<E>> for $unsigned
        where
          E: From<crate::Error>,
        {
          #[inline]
          fn decode(_: &mut (), dv: &mut DecodeWrapper<'_>) -> Result<Self, E> {
            if let &[$($elem,)+] = dv.bytes() {
                return Ok(<Self>::from_be_bytes([$($elem),+]));
              }
              Err(E::from(DatabaseError::UnexpectedBufferSize {
                expected: Usize::from(size_of::<Self>()).into_u32().unwrap_or(u32::MAX),
                received: Usize::from(dv.bytes().len()).into_u32().unwrap_or(u32::MAX)
              }.into()))
          }
        }
        impl<E> Encode<Mysql<E>> for $unsigned
        where
          E: From<crate::Error>,
        {
          #[inline]
          fn encode(&self, _: &mut (), ew: &mut EncodeWrapper<'_>) -> Result<(), E> {
            ew.sw().extend_from_copyable_slice(&self.to_be_bytes()).map_err(Into::into)?;
            Ok(())
          }
        }
        impl<E> Typed<Mysql<E>> for $unsigned
        where
          E: From<crate::Error>
        {
          const TY: Ty = $unsigned_pg_ty;
        }

        test!($unsigned, $unsigned, $instance);
      };
    }

  macro_rules! impl_primitive_from_array {
      ($instance:expr, [$($elem:ident),+], $ty:ident, $pg_ty:expr) => {
        impl<E> Decode<'_, Mysql<E>> for $ty
        where
          E: From<crate::Error>,
        {
          #[inline]
          fn decode(_: &mut (), dv: &mut DecodeWrapper<'_>) -> Result<Self, E> {
            if let &[$($elem,)+] = dv.bytes() {
              return Ok(<Self>::from_be_bytes([$($elem),+]));
            }
            Err(E::from(DatabaseError::UnexpectedBufferSize {
              expected: Usize::from(size_of::<Self>()).into_u32().unwrap_or(u32::MAX),
              received: Usize::from(dv.bytes().len()).into_u32().unwrap_or(u32::MAX)
            }.into()))
          }
        }

        impl<E> Encode<Mysql<E>> for $ty
        where
          E: From<crate::Error>,
        {
          #[inline]
          fn encode(&self, _: &mut (), ew: &mut EncodeWrapper<'_>) -> Result<(), E> {
            ew.sw().extend_from_copyable_slice(&self.to_be_bytes()).map_err(Into::into)?;
            Ok(())
          }
        }

        impl<E> Typed<Mysql<E>> for $ty
        where
          E: From<crate::Error>
        {
          const TY: Ty = $pg_ty;
        }

        test!($ty, $ty, $instance);
      }
    }

  impl_integer_from_array!(37, [a], (i8, Ty::Tiny), (u8, Ty::Tiny));
  impl_integer_from_array!(37, [a, b], (i16, Ty::Short), (u16, Ty::Short));
  impl_integer_from_array!(37, [a, b, c, d], (i32, Ty::Long), (u32, Ty::Long));
  impl_integer_from_array!(37, [a, b, c, d, e, f, g, h], (i64, Ty::LongLong), (u64, Ty::LongLong));

  impl_primitive_from_array!(37.0, [a, b, c, d], f32, Ty::Float);
  impl_primitive_from_array!(37.0, [a, b, c, d, e, f, g, h], f64, Ty::Double);
}
