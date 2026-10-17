use crate::{
  codec::{Decode, DecodeWrapper, Encode, EncodeWrapper},
  database::{
    Typed,
    client::mock_db::{MockDb, MockTy},
  },
  misc::from_utf8_basic,
};
use alloc::string::String;

impl<'exec> Decode<'exec, MockDb> for String {
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'exec, ()>) -> crate::Result<Self> {
    Ok(String::from(from_utf8_basic(dw.bytes)?))
  }
}

impl Encode<MockDb> for String {
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_, ()>) -> crate::Result<()> {
    ew.buffer.extend_from_copyable_slice(self.as_bytes())
  }
}

impl Typed<MockDb> for String {
  #[inline]
  fn runtime_ty(&self) -> Option<MockTy> {
    Some(MockTy::String)
  }

  #[inline]
  fn static_ty() -> Option<MockTy> {
    Some(MockTy::String)
  }
}

macro_rules! impl_primitive {
  ($($ty:ty => $ty_variant:ident),* $(,)?) => {
    $(
      impl<'exec> Decode<'exec, MockDb> for $ty {
        #[inline]
        fn decode(dw: &mut DecodeWrapper<'exec, ()>) -> crate::Result<Self> {
          let array = dw.bytes.try_into()?;
          Ok(<$ty>::from_be_bytes(array))
        }
      }

      impl Encode<MockDb> for $ty {
        #[inline]
        fn encode(&self, ew: &mut EncodeWrapper<'_, ()>) -> crate::Result<()> {
          ew.buffer.extend_from_copyable_slice(&self.to_be_bytes())
        }
      }

      impl Typed<MockDb> for $ty {
        #[inline]
        fn runtime_ty(&self) -> Option<MockTy> {
          Some(MockTy::$ty_variant)
        }

        #[inline]
        fn static_ty() -> Option<MockTy> {
          Some(MockTy::$ty_variant)
        }
      }
    )*
  };
}

impl_primitive! {
  f32 => F32,
  f64 => F64,
  u8 => U8,
  u16 => U16,
  u32 => U32,
  u64 => U64,
  usize => Usize,
}
