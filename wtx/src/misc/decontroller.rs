use crate::misc::Lease;

/// Decode/Encode Controller
pub trait DEController {
  /// Decode wrapper
  type DecodeWrapper<'any, 'de>: Lease<[u8]>;
  /// Error
  type Error: From<crate::Error>;
  /// Encode wrapper
  type EncodeWrapper<'inner, 'outer>: Lease<[u8]>
  where
    'inner: 'outer;
}

impl DEController for () {
  type DecodeWrapper<'any, 'de> = ();
  type Error = crate::Error;
  type EncodeWrapper<'inner, 'outer>
    = ()
  where
    'inner: 'outer;
}
