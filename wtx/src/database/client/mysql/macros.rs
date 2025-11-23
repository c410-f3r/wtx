macro_rules! impl_primitive {
  ($instance:expr, [$($elem:ident),+], $ty:ident, $pg_ty:expr) => {
    impl<E> Decode<'_, Mysql<E>> for $ty
    where
      E: From<crate::Error>,
    {
      #[inline]
      fn decode(dw: &mut DecodeWrapper<'_, '_>) -> Result<Self, E> {
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
      fn encode(&self, ew: &mut EncodeWrapper<'_>) -> Result<(), E> {
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
      Encode::<Mysql<crate::Error>>::encode(&instance, &mut ew).unwrap();
      let decoded: $ty = Decode::<Mysql<crate::Error>>::decode(&mut DecodeWrapper::new(
        ew.buffer(),
        "",
        crate::database::client::mysql::TyParams::empty(crate::database::client::mysql::Ty::Tiny),
      ))
      .unwrap();
      assert_eq!(instance, decoded);
    }
  };
}
