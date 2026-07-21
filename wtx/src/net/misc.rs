#[cfg(feature = "std")]
pub(crate) fn convert_to_io_slices<'buffer, 'bytes>(
  buffer: &'buffer mut [::std::io::IoSlice<'bytes>; 8],
  elems: &[&'bytes [u8]],
) -> crate::Result<&'buffer mut [::std::io::IoSlice<'bytes>]> {
  if elems.len() > 8 {
    return crate::misc::unlikely_elem(Err(crate::net::NetError::VectoredWriteOverflow.into()));
  }
  for (elem, io_slice) in elems.iter().zip(&mut *buffer) {
    *io_slice = ::std::io::IoSlice::new(elem);
  }
  Ok(buffer.get_mut(..elems.len()).unwrap_or_default())
}
