use crate::{
  collection::{ArrayString, IndexedStorage, IndexedStorageLen, IndexedStorageMut},
  database::{
    Typed,
    client::mysql::{DecodeWrapper, EncodeWrapper, Mysql, Ty, TyParams, misc::encoded_len},
  },
  de::{Decode, Encode},
  misc::{Usize, from_utf8_basic},
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

// ArrayString

impl<E, L, const N: usize> Decode<'_, Mysql<E>> for ArrayString<L, N>
where
  E: From<crate::Error>,
  L: IndexedStorageLen,
{
  #[inline]
  fn decode(aux: &mut (), dw: &mut DecodeWrapper<'_>) -> Result<Self, E> {
    Ok(<&str as Decode<Mysql<E>>>::decode(aux, dw)?.try_into()?)
  }
}
impl<E, L, const N: usize> Encode<Mysql<E>> for ArrayString<L, N>
where
  E: From<crate::Error>,
  L: IndexedStorageLen,
{
  #[inline]
  fn encode(&self, aux: &mut (), ew: &mut EncodeWrapper<'_>) -> Result<(), E> {
    <&str as Encode<Mysql<E>>>::encode(&self.as_str(), aux, ew)
  }
}
impl<E, L, const N: usize> Typed<Mysql<E>> for ArrayString<L, N>
where
  E: From<crate::Error>,
  L: IndexedStorageLen,
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

// &str

impl<'exec, E> Decode<'exec, Mysql<E>> for &'exec str
where
  E: From<crate::Error>,
{
  #[inline]
  fn decode(aux: &mut (), dw: &mut DecodeWrapper<'exec>) -> Result<Self, E> {
    Ok(from_utf8_basic(<&[u8] as Decode<Mysql<E>>>::decode(aux, dw)?).map_err(crate::Error::from)?)
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
