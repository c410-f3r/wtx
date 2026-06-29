//! Abstractions over different types of data streams.

macro_rules! _local_write_all {
  ($bytes:expr, $write:expr) => {{
    while !$bytes.is_empty() {
      match $write {
        Err(err) => return Err(err.into()),
        Ok(0) => return { Err(crate::Error::UnexpectedStreamWriteEOF) },
        Ok(n) => $bytes = $bytes.get(n..).unwrap_or_default(),
      }
    }
  }};
}

macro_rules! _local_write_all_vectored {
  ($bytes:expr, $this:ident, |$io_slices:ident| $write_many:expr) => {{
    match $bytes {
      [] => return Ok(()),
      [single] => {
        <Self as crate::stream::StreamWriter>::write_all($this, single).await?;
      }
      _ => {
        let mut buffer = [std::io::IoSlice::new(&[]); _];
        let mut $io_slices = crate::stream::convert_to_io_slices(&mut buffer, $bytes)?;
        while !$io_slices.is_empty() {
          match $write_many {
            Err(err) => return Err(err.into()),
            Ok(0) => return Err(crate::Error::UnexpectedStreamWriteEOF),
            Ok(n) => std::io::IoSlice::advance_slices(&mut $io_slices, n),
          }
        }
      }
    }
  }};
}

mod buf_stream_reader;
mod bytes_stream;
#[cfg(feature = "embassy-net")]
mod embassy_net;
#[cfg(feature = "std")]
mod std;
mod stream_common;
mod stream_reader;
mod stream_writer;
#[cfg(feature = "tokio")]
mod tokio;

pub use buf_stream_reader::{BufStreamReader, BufStreamReaderError, StreamReadItem};
pub use bytes_stream::BytesStream;
pub use stream_common::StreamCommon;
pub use stream_reader::StreamReader;
pub use stream_writer::StreamWriter;

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

#[cfg(feature = "std")]
fn convert_to_io_slices<'buffer, 'bytes>(
  buffer: &'buffer mut [::std::io::IoSlice<'bytes>; 8],
  elems: &[&'bytes [u8]],
) -> crate::Result<&'buffer mut [::std::io::IoSlice<'bytes>]> {
  if elems.len() > 8 {
    return crate::misc::unlikely_elem(Err(crate::Error::VectoredWriteOverflow));
  }
  for (elem, io_slice) in elems.iter().zip(&mut *buffer) {
    *io_slice = ::std::io::IoSlice::new(elem);
  }
  Ok(buffer.get_mut(..elems.len()).unwrap_or_default())
}
