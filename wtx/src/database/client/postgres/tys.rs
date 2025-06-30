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
      Encode::<Postgres<crate::Error>>::encode(&instance, &mut (), &mut ew).unwrap();
      let decoded: $ty = Decode::<Postgres<crate::Error>>::decode(
        &mut (),
        &mut DecodeWrapper::new(
          ew.buffer().curr_bytes(),
          crate::database::client::postgres::Ty::Any,
        ),
      )
      .unwrap();
      assert_eq!(instance, decoded);
    }
  };
}

mod arguments;
mod array;
mod calendar;
mod collection;
mod ip;
#[cfg(feature = "rust_decimal")]
mod pg_numeric;
mod primitives;
#[cfg(feature = "rust_decimal")]
mod rust_decimal;
#[cfg(feature = "serde_json")]
mod serde_json;
#[cfg(feature = "uuid")]
mod uuid;
