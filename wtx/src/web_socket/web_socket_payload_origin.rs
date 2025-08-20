use crate::collection::Vector;

/// The payload of a received frame can represent two things:
///
/// 1. Uncompressed data stored in internal structures, which are originated from
///    control frames or single text/binary frames.
/// 2. One or more compressed or uncompressed frame concatenations that are stored in the provided
///    buffer.
///
/// This distinction exists to ensure that there are as few copies as possible. In case of doubt,
/// [`WebSocketPayloadOrigin::Consistent`] should probably be used when the frame payload needs to be
/// sent to a different structure or task.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WebSocketPayloadOrigin {
  /// The payload of the frame may consist of either the first or the second case described at
  /// the top-level documentation.
  ///
  /// ```ignore,rust
  /// // Received a control frame or a single uncompressed text/binary frame
  /// let frame0 = ws.read_frame(&mut buffer, WebSocketPayloadOrigin::Adaptive).await?;
  /// // Payload makes reference to internal bytes. Buffer will always be empty
  /// assert_eq!((frame0.payload().is_empty(), buffer.is_empty()), (false, true));
  ///
  /// // Received a compressed frame or compressed/uncompressed continuation frames
  /// let frame1 = ws.read_frame(&mut buffer, WebSocketPayloadOrigin::Adaptive).await?;
  /// // Payload is simply referencing the passed buffer, they are the same thing!
  /// assert_eq!((frame1.payload().is_empty(), buffer.is_empty()), (false, false));
  /// ```
  Adaptive,
  /// Contents are always copied to the provided buffer. This implies one extra `memcpy` in the
  /// first case described at the top-level documentation.
  ///
  /// ```ignore,rust
  /// // Received a control frame or a single uncompressed text/binary frame
  /// let frame0 = ws.read_frame(&mut buffer, WebSocketPayloadOrigin::Consistent).await?;
  /// // Payload is simply referencing the passed buffer, they are the same thing!
  /// assert_eq!((frame0.payload().is_empty(), buffer.is_empty()), (false, false));
  ///
  /// // Received a compressed frame or compressed/uncompressed continuation frames
  /// let frame1 = ws.read_frame(&mut buffer, WebSocketPayloadOrigin::Consistent).await?;
  /// // Payload is simply referencing the passed buffer, they are the same thing!
  /// assert_eq!((frame1.payload().is_empty(), buffer.is_empty()), (false, false));
  /// ```
  Consistent,
}

impl WebSocketPayloadOrigin {
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
      WebSocketPayloadOrigin::Adaptive => network_buffer,
      WebSocketPayloadOrigin::Consistent => {
        user_buffer.extend_from_copyable_slice(network_buffer)?;
        user_buffer.as_slice_mut()
      }
    })
  }
}
