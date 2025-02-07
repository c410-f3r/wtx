use crate::misc::Lease;

/// Decode/Encode Controller
pub trait DEController {
  /// Auxiliary structure
  type Aux;
  /// Decode wrapper
  type DecodeWrapper<'de>: Lease<[u8]>;
  /// Error
  type Error: From<crate::Error>;
  /// Encode wrapper
  type EncodeWrapper<'inner, 'outer>: Lease<[u8]>
  where
    'inner: 'outer;
}

impl DEController for () {
  type Aux = ();
  type DecodeWrapper<'de> = ();
  type Error = crate::Error;
  type EncodeWrapper<'inner, 'outer>
    = ()
  where
    'inner: 'outer;
}

impl<T> DEController for &T
where
  T: DEController,
{
  type Aux = T::Aux;
  type DecodeWrapper<'de> = T::DecodeWrapper<'de>;
  type Error = T::Error;
  type EncodeWrapper<'inner, 'outer>
    = T::EncodeWrapper<'inner, 'outer>
  where
    'inner: 'outer;
}

impl<T> DEController for &mut T
where
  T: DEController,
{
  type Aux = T::Aux;
  type DecodeWrapper<'de> = T::DecodeWrapper<'de>;
  type Error = T::Error;
  type EncodeWrapper<'inner, 'outer>
    = T::EncodeWrapper<'inner, 'outer>
  where
    'inner: 'outer;
}
