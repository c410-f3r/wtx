/// The result of a HTTP request or response frame parse
#[derive(Debug)]
pub enum ParseStatus {
  /// The completed result.
  Complete(usize),
  /// A partial result.
  Partial,
}

impl From<httparse::Status<usize>> for ParseStatus {
  #[inline]
  fn from(from: httparse::Status<usize>) -> Self {
    match from {
      httparse::Status::Complete(elem) => Self::Complete(elem),
      httparse::Status::Partial => Self::Partial,
    }
  }
}
