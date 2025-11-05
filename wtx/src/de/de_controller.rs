use crate::misc::Lease;

/// Decode/Encode Controller
pub trait DEController {
  /// Decode wrapper
  type DecodeWrapper<'inner, 'outer, 'rem>: Lease<[u8]>
  where
    'inner: 'outer;
  /// Error
  type Error: From<crate::Error>;
  /// Encode wrapper
  type EncodeWrapper<'inner, 'outer, 'rem>: Lease<[u8]>
  where
    'inner: 'outer;
}

impl DEController for () {
  type DecodeWrapper<'inner, 'outer, 'rem>
    = ()
  where
    'inner: 'outer;
  type Error = crate::Error;
  type EncodeWrapper<'inner, 'outer, 'rem>
    = ()
  where
    'inner: 'outer;
}

impl<T> DEController for &T
where
  T: DEController,
{
  type DecodeWrapper<'inner, 'outer, 'rem>
    = T::DecodeWrapper<'inner, 'outer, 'rem>
  where
    'inner: 'outer;
  type Error = T::Error;
  type EncodeWrapper<'inner, 'outer, 'rem>
    = T::EncodeWrapper<'inner, 'outer, 'rem>
  where
    'inner: 'outer;
}

impl<T> DEController for &mut T
where
  T: DEController,
{
  type DecodeWrapper<'inner, 'outer, 'rem>
    = T::DecodeWrapper<'inner, 'outer, 'rem>
  where
    'inner: 'outer;
  type Error = T::Error;
  type EncodeWrapper<'inner, 'outer, 'rem>
    = T::EncodeWrapper<'inner, 'outer, 'rem>
  where
    'inner: 'outer;
}
