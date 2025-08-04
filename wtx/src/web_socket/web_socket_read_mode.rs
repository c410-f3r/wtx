use crate::collection::{IndexedStorageMut as _, Vector};

/// The payload of a received frame can represent two things:
///
/// 1. Uncompressed data stored in internal structures, which are originated from
///    control frames or single text/binary frames.
/// 2. One or more compressed or uncompressed frame concatenations that are stored in the provided
///    buffer.
///
/// This distinction exists to ensure that there are as few copies as possible.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WebSocketReadMode {
  /// The payload of the frame may consist of either the first or the second case described at
  /// the top-level documentation.
  Adaptive,
  /// Contents are always copied to the provided buffer. This implies one extra `memcpy` in the
  /// first case described at the top-level documentation.
  Consistent,
}

impl WebSocketReadMode {
  pub(crate) fn manage_payload<'nb, 'rslt, 'ub>(
    self,
    network_buffer: &'nb mut [u8],
    user_buffer: &'ub mut Vector<u8>,
  ) -> crate::Result<&'rslt mut [u8]>
  where
    'nb: 'rslt,
    'ub: 'rslt,
  {
    Ok(match self {
      WebSocketReadMode::Adaptive => network_buffer,
      WebSocketReadMode::Consistent => {
        user_buffer.extend_from_copyable_slice(network_buffer)?;
        user_buffer.as_slice_mut()
      }
    })
  }
}
