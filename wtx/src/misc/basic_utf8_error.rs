/// Basic string error that doesn't contain any information.
#[derive(Debug)]
pub struct BasicUtf8Error;

impl From<BasicUtf8Error> for crate::Error {
  #[inline]
  fn from(_: BasicUtf8Error) -> Self {
    Self::InvalidUTF8
  }
}
