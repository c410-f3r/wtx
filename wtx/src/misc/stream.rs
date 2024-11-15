macro_rules! _local_write_all_vectored {
  ($bytes:expr, $this:ident, |$io_slices:ident| $write_many:expr) => {
    if let [single] = $bytes {
      <Self as crate::misc::StreamWriter>::write_all($this, single).await?;
    } else {
      let mut buffer = [std::io::IoSlice::new(&[]); 8];
      let mut $io_slices = crate::misc::stream::convert_to_io_slices(&mut buffer, $bytes);
      while !$io_slices.is_empty() {
        match $write_many {
          Err(e) => return Err(e.into()),
          Ok(0) => return Err(crate::Error::UnexpectedStreamWriteEOF),
          Ok(n) => std::io::IoSlice::advance_slices(&mut $io_slices, n),
        }
      }
    }
  };
}

mod bytes_stream;
#[cfg(feature = "embedded-tls")]
mod embedded_tls;
#[cfg(feature = "std")]
mod std;
mod stream_reader;
mod stream_with_tls;
mod stream_writer;
#[cfg(feature = "tokio")]
mod tokio;
#[cfg(feature = "tokio-rustls")]
mod tokio_rustls;

pub use bytes_stream::BytesStream;
pub use stream_reader::StreamReader;
pub use stream_with_tls::StreamWithTls;
pub use stream_writer::StreamWriter;

/// A stream of values produced asynchronously.
pub trait Stream: StreamReader + StreamWriter {}

impl<T> Stream for T where T: StreamReader + StreamWriter {}

#[cfg(feature = "std")]
#[inline]
fn convert_to_io_slices<'buffer, 'bytes>(
  buffer: &'buffer mut [::std::io::IoSlice<'bytes>; 8],
  elems: &[&'bytes [u8]],
) -> &'buffer mut [::std::io::IoSlice<'bytes>] {
  use ::std::io::IoSlice;
  match elems {
    [a] => {
      buffer[0] = IoSlice::new(a);
      &mut buffer[..1]
    }
    [a, b] => {
      buffer[0] = IoSlice::new(a);
      buffer[1] = IoSlice::new(b);
      &mut buffer[..2]
    }
    [a, b, c] => {
      buffer[0] = IoSlice::new(a);
      buffer[1] = IoSlice::new(b);
      buffer[2] = IoSlice::new(c);
      &mut buffer[..3]
    }
    [a, b, c, d] => {
      buffer[0] = IoSlice::new(a);
      buffer[1] = IoSlice::new(b);
      buffer[2] = IoSlice::new(c);
      buffer[3] = IoSlice::new(d);
      &mut buffer[..4]
    }
    [a, b, c, d, e] => {
      buffer[0] = IoSlice::new(a);
      buffer[1] = IoSlice::new(b);
      buffer[2] = IoSlice::new(c);
      buffer[3] = IoSlice::new(d);
      buffer[4] = IoSlice::new(e);
      &mut buffer[..5]
    }
    [a, b, c, d, e, f] => {
      buffer[0] = IoSlice::new(a);
      buffer[1] = IoSlice::new(b);
      buffer[2] = IoSlice::new(c);
      buffer[3] = IoSlice::new(d);
      buffer[4] = IoSlice::new(e);
      buffer[5] = IoSlice::new(f);
      &mut buffer[..6]
    }
    [a, b, c, d, e, f, g] => {
      buffer[0] = IoSlice::new(a);
      buffer[1] = IoSlice::new(b);
      buffer[2] = IoSlice::new(c);
      buffer[3] = IoSlice::new(d);
      buffer[4] = IoSlice::new(e);
      buffer[5] = IoSlice::new(f);
      buffer[6] = IoSlice::new(g);
      &mut buffer[..7]
    }
    [a, b, c, d, e, f, g, h] => {
      buffer[0] = IoSlice::new(a);
      buffer[1] = IoSlice::new(b);
      buffer[2] = IoSlice::new(c);
      buffer[3] = IoSlice::new(d);
      buffer[4] = IoSlice::new(e);
      buffer[5] = IoSlice::new(f);
      buffer[6] = IoSlice::new(g);
      buffer[7] = IoSlice::new(h);
      &mut buffer[..8]
    }
    #[expect(clippy::panic, reason = "Programming error")]
    _ => panic!("It is not possible to send more than 8 vectorized slices"),
  }
}
