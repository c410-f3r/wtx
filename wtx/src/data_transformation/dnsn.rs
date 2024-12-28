//! # `D`eserializatio`N`/`S`erializatio`N`
//!
//! Abstracts different serialization/deserialization frameworks to enhance de-coupling,
//! enable choice and improve experimentation.

#[cfg(all(feature = "client-api-framework", test))]
#[macro_use]
mod tests;

#[cfg(feature = "borsh")]
mod borsh;
mod decode_wrapper;
mod encode_wrapper;
#[cfg(feature = "quick-protobuf")]
mod quick_protobuf;
#[cfg(feature = "serde_json")]
mod serde_json;

#[cfg(feature = "borsh")]
pub use self::borsh::*;
#[cfg(feature = "quick-protobuf")]
pub use self::quick_protobuf::*;
#[cfg(feature = "serde_json")]
pub use self::serde_json::*;
pub use decode_wrapper::DecodeWrapper;
pub use encode_wrapper::EncodeWrapper;

/// `D`eserializatio`N`/`S`erializatio`N`
pub struct Dnsn<DRSR>(core::marker::PhantomData<DRSR>);

impl<DRSR> crate::misc::DEController for Dnsn<DRSR>
where
  for<'any> DRSR: 'any,
{
  type DecodeWrapper<'any, 'de> = DecodeWrapper<'any, 'de, DRSR>;
  type Error = crate::Error;
  type EncodeWrapper<'inner, 'outer>
    = EncodeWrapper<'inner, DRSR>
  where
    'inner: 'outer;
}

impl<DRSR> crate::misc::Decode<'_, Dnsn<DRSR>> for ()
where
  for<'any> DRSR: 'any,
{
  #[inline]
  fn decode(_: &mut DecodeWrapper<'_, '_, DRSR>) -> crate::Result<Self> {
    Ok(())
  }
}

impl<DRSR> crate::misc::DecodeSeq<'_, Dnsn<DRSR>> for ()
where
  for<'any> DRSR: 'any,
{
  #[inline]
  fn decode_seq(
    _: &mut crate::misc::Vector<Self>,
    _: &mut DecodeWrapper<'_, '_, DRSR>,
  ) -> crate::Result<()> {
    Ok(())
  }
}

impl<DRSR> crate::misc::Encode<Dnsn<DRSR>> for ()
where
  for<'any> DRSR: 'any,
{
  #[inline]
  fn encode(&self, _: &mut EncodeWrapper<'_, DRSR>) -> Result<(), crate::Error> {
    Ok(())
  }
}
