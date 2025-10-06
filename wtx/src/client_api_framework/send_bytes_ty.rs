/// The source used to send requests.
#[derive(Clone, Copy, Debug)]
pub enum SendBytesSource<'bytes> {
  /// Uses this parameter to perform a request.
  Param(&'bytes [u8]),
  /// Uses the buffer of the `PkgsAux` structure.
  PkgsAux,
}

impl<'bytes> SendBytesSource<'bytes> {
  pub(crate) const fn bytes<'bb, 'rslt>(self, byte_buffer: &'bb [u8]) -> &'rslt [u8]
  where
    'bb: 'rslt,
    'bytes: 'rslt,
  {
    match self {
      SendBytesSource::Param(elem) => elem,
      SendBytesSource::PkgsAux => byte_buffer,
    }
  }
}
