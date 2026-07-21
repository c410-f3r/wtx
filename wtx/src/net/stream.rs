use crate::net::{StreamReader, StreamWriter};

/// A stream of values produced asynchronously.
pub trait Stream: StreamReader + StreamWriter {
  /// Connects the reader and the writer.
  type BridgeOwned;
  /// See [`StreamReader`].
  type ReadHalfOwned: StreamReader;
  /// See [`StreamWriter`].
  type WriteHalfOwned: StreamWriter;

  /// Splits this instance into owned parts that can be used in concurrent scenarios.
  fn into_split(
    self,
  ) -> crate::Result<(Self::BridgeOwned, Self::ReadHalfOwned, Self::WriteHalfOwned)>;
}

impl Stream for () {
  type BridgeOwned = ();
  type ReadHalfOwned = ();
  type WriteHalfOwned = ();

  #[inline]
  fn into_split(
    self,
  ) -> crate::Result<(Self::BridgeOwned, Self::ReadHalfOwned, Self::WriteHalfOwned)> {
    Ok(((), (), ()))
  }
}
