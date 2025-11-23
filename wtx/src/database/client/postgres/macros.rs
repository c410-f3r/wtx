macro_rules! impl_primitive {
  ($instance:expr, [$($elem:ident),+], $ty:ident, $pg_ty:expr) => {
    impl<E> Decode<'_, Postgres<E>> for $ty
    where
      E: From<crate::Error>,
    {
      #[inline]
      fn decode(dw: &mut DecodeWrapper<'_, '_>) -> Result<Self, E> {
        if let &[$($elem,)+] = dw.bytes() {
          return Ok(<Self>::from_be_bytes([$($elem),+]).into());
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
      fn encode(&self, ew: &mut EncodeWrapper<'_, '_>) -> Result<(), E> {
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
      let vec = &mut crate::misc::FilledBuffer::default();
      let mut sw = crate::misc::SuffixWriter::new(0, vec.vector_mut());
      let mut ew = EncodeWrapper::new(&mut sw);
      let instance: $ty = $instance;
      Encode::<Postgres<crate::Error>>::encode(&instance, &mut ew).unwrap();
      let decoded: $ty = Decode::<Postgres<crate::Error>>::decode(&mut DecodeWrapper::new(
        ew.buffer().curr_bytes(),
        "",
        crate::database::client::postgres::Ty::Any,
      ))
      .unwrap();
      assert_eq!(instance, decoded);
    }
  };
}
