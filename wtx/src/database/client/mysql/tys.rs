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
      let mut vec = crate::collection::Vector::new();
      let mut ew = EncodeWrapper::new(&mut vec);
      let instance: $ty = $instance;
      Encode::<Mysql<crate::Error>>::encode(&instance, &mut (), &mut ew).unwrap();
      let decoded: $ty = Decode::<Mysql<crate::Error>>::decode(
        &mut (),
        &mut DecodeWrapper::new(ew.buffer(), crate::database::client::mysql::Ty::Tiny),
      )
      .unwrap();
      assert_eq!(instance, decoded);
    }
  };
}

#[cfg(feature = "calendar")]
mod calendar;

mod collections {
  use crate::{
    collection::ArrayString,
    database::{
      Typed,
      client::mysql::{DecodeWrapper, EncodeWrapper, Mysql, Ty, TyParams, misc::encoded_len},
    },
    misc::{Decode, Encode, Usize, from_utf8_basic},
  };
  use alloc::string::String;

  // &[u8]

  impl<'exec, E> Decode<'exec, Mysql<E>> for &'exec [u8]
  where
    E: From<crate::Error>,
  {
    #[inline]
    fn decode(_: &mut (), dw: &mut DecodeWrapper<'exec>) -> Result<Self, E> {
      Ok(dw.bytes())
    }
  }
  impl<E> Encode<Mysql<E>> for &[u8]
  where
    E: From<crate::Error>,
  {
    #[inline]
    fn encode(&self, _: &mut (), ew: &mut EncodeWrapper<'_>) -> Result<(), E> {
      let len = encoded_len(*Usize::from(self.len()))?;
      let _ = ew.buffer().extend_from_copyable_slices([len.as_slice(), self])?;
      Ok(())
    }
  }
  impl<E> Typed<Mysql<E>> for &[u8]
  where
    E: From<crate::Error>,
  {
    #[inline]
    fn runtime_ty(&self) -> Option<TyParams> {
      <Self as Typed<Mysql<E>>>::static_ty()
    }

    #[inline]
    fn static_ty() -> Option<TyParams> {
      Some(TyParams::binary(Ty::Blob))
    }
  }

  // String

  impl<E, const N: usize> Decode<'_, Mysql<E>> for ArrayString<N>
  where
    E: From<crate::Error>,
  {
    #[inline]
    fn decode(aux: &mut (), dw: &mut DecodeWrapper<'_>) -> Result<Self, E> {
      Ok(<&str as Decode<Mysql<E>>>::decode(aux, dw)?.try_into()?)
    }
  }
  impl<E, const N: usize> Encode<Mysql<E>> for ArrayString<N>
  where
    E: From<crate::Error>,
  {
    #[inline]
    fn encode(&self, aux: &mut (), ew: &mut EncodeWrapper<'_>) -> Result<(), E> {
      <&str as Encode<Mysql<E>>>::encode(&self.as_str(), aux, ew)
    }
  }
  impl<E, const N: usize> Typed<Mysql<E>> for ArrayString<N>
  where
    E: From<crate::Error>,
  {
    #[inline]
    fn runtime_ty(&self) -> Option<TyParams> {
      <Self as Typed<Mysql<E>>>::static_ty()
    }

    #[inline]
    fn static_ty() -> Option<TyParams> {
      Some(TyParams::empty(Ty::VarString))
    }
  }

  impl<'exec, E> Decode<'exec, Mysql<E>> for &'exec str
  where
    E: From<crate::Error>,
  {
    #[inline]
    fn decode(aux: &mut (), dw: &mut DecodeWrapper<'exec>) -> Result<Self, E> {
      Ok(
        from_utf8_basic(<&[u8] as Decode<Mysql<E>>>::decode(aux, dw)?)
          .map_err(crate::Error::from)?,
      )
    }
  }
  impl<E> Encode<Mysql<E>> for &str
  where
    E: From<crate::Error>,
  {
    #[inline]
    fn encode(&self, aux: &mut (), ew: &mut EncodeWrapper<'_>) -> Result<(), E> {
      <&[u8] as Encode<Mysql<E>>>::encode(&self.as_bytes(), aux, ew)
    }
  }
  impl<E> Typed<Mysql<E>> for &str
  where
    E: From<crate::Error>,
  {
    #[inline]
    fn runtime_ty(&self) -> Option<TyParams> {
      <Self as Typed<Mysql<E>>>::static_ty()
    }

    #[inline]
    fn static_ty() -> Option<TyParams> {
      Some(TyParams::empty(Ty::VarString))
    }
  }

  // String

  impl<E> Decode<'_, Mysql<E>> for String
  where
    E: From<crate::Error>,
  {
    #[inline]
    fn decode(aux: &mut (), dw: &mut DecodeWrapper<'_>) -> Result<Self, E> {
      <&str as Decode<Mysql<E>>>::decode(aux, dw).map(String::from)
    }
  }
  impl<E> Encode<Mysql<E>> for String
  where
    E: From<crate::Error>,
  {
    #[inline]
    fn encode(&self, aux: &mut (), ew: &mut EncodeWrapper<'_>) -> Result<(), E> {
      <&str as Encode<Mysql<E>>>::encode(&self.as_str(), aux, ew)
    }
  }
  impl<E> Typed<Mysql<E>> for String
  where
    E: From<crate::Error>,
  {
    #[inline]
    fn runtime_ty(&self) -> Option<TyParams> {
      <Self as Typed<Mysql<E>>>::static_ty()
    }

    #[inline]
    fn static_ty() -> Option<TyParams> {
      Some(TyParams::empty(Ty::VarString))
    }
  }
  kani!(string, String);
}

mod primitives {
  use crate::{
    database::{
      DatabaseError, Typed,
      client::mysql::{DecodeWrapper, EncodeWrapper, Mysql, Ty, TyParams},
    },
    misc::{Decode, Encode, Usize},
  };

  impl<E> Decode<'_, Mysql<E>> for ()
  where
    E: From<crate::Error>,
  {
    #[inline]
    fn decode(_: &mut (), _: &mut DecodeWrapper<'_>) -> Result<Self, E> {
      Ok(())
    }
  }

  // bool

  impl<E> Decode<'_, Mysql<E>> for bool
  where
    E: From<crate::Error>,
  {
    #[inline]
    fn decode(_: &mut (), dw: &mut DecodeWrapper<'_>) -> Result<Self, E> {
      let &[byte] = dw.bytes() else {
        return Err(E::from(
          DatabaseError::UnexpectedBufferSize {
            expected: 1,
            received: Usize::from(dw.bytes().len()).into_saturating_u32(),
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
      ew.buffer().push((*self).into())?;
      Ok(())
    }
  }
  impl<E> Typed<Mysql<E>> for bool
  where
    E: From<crate::Error>,
  {
    #[inline]
    fn runtime_ty(&self) -> Option<TyParams> {
      <Self as Typed<Mysql<E>>>::static_ty()
    }

    #[inline]
    fn static_ty() -> Option<TyParams> {
      Some(TyParams::unsigned(Ty::Tiny))
    }
  }

  macro_rules! impl_integer_from_array {
      ($instance:expr, [$($elem:ident),+], ($signed:ident, $signed_pg_ty:expr), ($unsigned:ident, $unsigned_pg_ty:expr)) => {
        impl_primitive_from_array!($instance, [$($elem),+], $signed, $signed_pg_ty);

        impl<E> Decode<'_, Mysql<E>> for $unsigned
        where
          E: From<crate::Error>,
        {
          #[inline]
          fn decode(_: &mut (), dw: &mut DecodeWrapper<'_>) -> Result<Self, E> {
            if let &[$($elem,)+] = dw.bytes() {
              return Ok(<Self>::from_le_bytes([$($elem),+]));
            }
            Err(E::from(DatabaseError::UnexpectedBufferSize {
              expected: Usize::from(size_of::<Self>()).into_saturating_u32(),
              received: Usize::from(dw.bytes().len()).into_saturating_u32()
            }.into()))
          }
        }
        impl<E> Encode<Mysql<E>> for $unsigned
        where
          E: From<crate::Error>,
        {
          #[inline]
          fn encode(&self, _: &mut (), ew: &mut EncodeWrapper<'_>) -> Result<(), E> {
            ew.buffer().extend_from_copyable_slice(&self.to_le_bytes()).map_err(Into::into)?;
            Ok(())
          }
        }
        impl<E> Typed<Mysql<E>> for $unsigned
        where
          E: From<crate::Error>
        {
          #[inline]
          fn runtime_ty(&self) -> Option<TyParams> {
            <Self as Typed<Mysql<E>>>::static_ty()
          }

          #[inline]
          fn static_ty() -> Option<TyParams> {
            Some($unsigned_pg_ty)
          }
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
          fn decode(_: &mut (), dw: &mut DecodeWrapper<'_>) -> Result<Self, E> {
            if let &[$($elem,)+] = dw.bytes() {
              return Ok(<Self>::from_le_bytes([$($elem),+]));
            }
            Err(E::from(DatabaseError::UnexpectedBufferSize {
              expected: Usize::from(size_of::<Self>()).into_saturating_u32(),
              received: Usize::from(dw.bytes().len()).into_saturating_u32()
            }.into()))
          }
        }

        impl<E> Encode<Mysql<E>> for $ty
        where
          E: From<crate::Error>,
        {
          #[inline]
          fn encode(&self, _: &mut (), ew: &mut EncodeWrapper<'_>) -> Result<(), E> {
            ew.buffer().extend_from_copyable_slice(&self.to_le_bytes()).map_err(Into::into)?;
            Ok(())
          }
        }

        impl<E> Typed<Mysql<E>> for $ty
        where
          E: From<crate::Error>
        {
          #[inline]
          fn runtime_ty(&self) -> Option<TyParams> {
            <Self as Typed<Mysql<E>>>::static_ty()
          }

          #[inline]
          fn static_ty() -> Option<TyParams> {
            Some($pg_ty)
          }
        }

        test!($ty, $ty, $instance);
      }
    }

  impl_integer_from_array!(
    37,
    [a],
    (i8, TyParams::binary(Ty::Tiny)),
    (u8, TyParams::unsigned(Ty::Tiny))
  );
  impl_integer_from_array!(
    37,
    [a, b],
    (i16, TyParams::binary(Ty::Short)),
    (u16, TyParams::unsigned(Ty::Short))
  );
  impl_integer_from_array!(
    37,
    [a, b, c, d],
    (i32, TyParams::binary(Ty::Long)),
    (u32, TyParams::unsigned(Ty::Long))
  );
  impl_integer_from_array!(
    37,
    [a, b, c, d, e, f, g, h],
    (i64, TyParams::binary(Ty::LongLong)),
    (u64, TyParams::unsigned(Ty::LongLong))
  );

  impl_primitive_from_array!(37.0, [a, b, c, d], f32, TyParams::binary(Ty::Float));
  impl_primitive_from_array!(37.0, [a, b, c, d, e, f, g, h], f64, TyParams::binary(Ty::Double));
}
