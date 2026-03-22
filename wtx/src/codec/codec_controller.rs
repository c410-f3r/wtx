use crate::misc::Lease;

/// Decode/Encode Controller
pub trait CodecController {
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

impl CodecController for () {
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

impl<T> CodecController for &T
where
  T: CodecController,
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

impl<T> CodecController for &mut T
where
  T: CodecController,
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
