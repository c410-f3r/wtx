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
        &mut DecodeWrapper::new(
          ew.buffer(),
          "",
          crate::database::client::mysql::TyParams::empty(crate::database::client::mysql::Ty::Tiny),
        ),
      )
      .unwrap();
      assert_eq!(instance, decoded);
    }
  };
}

mod calendar;
mod collection;
mod primitives;
