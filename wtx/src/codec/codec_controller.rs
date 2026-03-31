use crate::misc::Lease;

/// Decoder/Encoder Controller
///
/// This is a marker trait intended for coordination. When serializing or deserializing
/// implementations aren't expected to be instantiated.
//
// 'inner, 'outer and 'rem exist to allow nested codecs. See PostgreSQL.
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
