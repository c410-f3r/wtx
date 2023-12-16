/// Basic string error that doesn't contain any information.
pub(crate) struct BasicUtf8Error;

impl From<BasicUtf8Error> for crate::Error {
  #[inline]
  fn from(_: BasicUtf8Error) -> Self {
    Self::InvalidUTF8
  }
}
