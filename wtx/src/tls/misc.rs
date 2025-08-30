use crate::{
  de::Decode,
  tls::{TlsError, de::De},
};

#[inline]
pub(crate) fn u16_chunk<'de, T>(
  dw: &mut &'de [u8],
  err: TlsError,
  cb: impl FnOnce(&mut &'de [u8]) -> crate::Result<T>,
) -> crate::Result<T> {
  let len: u16 = Decode::<'_, De>::decode(&mut (), dw)?;
  let Some((mut before, after)) = dw.split_at_checked(len.into()) else {
    return Err(err.into());
  };
  let rslt = cb(&mut before)?;
  if !before.is_empty() {
    return Err(err.into());
  }
  *dw = after;
  Ok(rslt)
}
