macro_rules! _local_write_all {
  ($bytes:expr, $write:expr) => {{
    while !$bytes.is_empty() {
      match $write {
        Err(err) => return Err(err.into()),
        Ok(0) => return { Err(crate::net::NetError::UnexpectedStreamWriteEOF.into()) },
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
        <Self as crate::net::StreamWriter>::write_all($this, single).await?;
      }
      _ => {
        let mut buffer = [std::io::IoSlice::new(&[]); _];
        let mut $io_slices = crate::net::misc::convert_to_io_slices(&mut buffer, $bytes)?;
        while !$io_slices.is_empty() {
          match $write_many {
            Err(err) => return Err(err.into()),
            Ok(0) => return Err(crate::net::NetError::UnexpectedStreamWriteEOF.into()),
            Ok(n) => std::io::IoSlice::advance_slices(&mut $io_slices, n),
          }
        }
      }
    }
  }};
}
