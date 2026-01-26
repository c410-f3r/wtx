use crate::{
  collection::TryExtend,
  de::Decode,
  misc::{
    net::{PartitionedFilledBuffer, read_header, read_payload},
    unlikely_elem,
  },
  stream::StreamReader,
  tls::{
    TlsError,
    de::De,
    decode_wrapper::DecodeWrapper,
    protocol::{
      protocol_version::ProtocolVersion, record_content_type::RecordContentType, u24::U24,
    },
  },
};

pub(crate) fn duplicated_error(is_some: bool) -> crate::Result<()> {
  if is_some {
    return Err(TlsError::DuplicatedClientHelloParameters.into());
  }
  Ok(())
}

pub(crate) async fn fetch_rec_from_stream<SR>(
  network_buffer: &mut PartitionedFilledBuffer,
  stream_reader: &mut SR,
) -> crate::Result<RecordContentType>
where
  SR: StreamReader,
{
  network_buffer.clear_if_following_is_empty();
  network_buffer.reserve(5)?;
  let mut read = network_buffer.following_len();
  let buffer = network_buffer.following_rest_mut();
  let [a, b, c, d, e] = read_header::<0, 5, SR>(buffer, &mut read, stream_reader).await?;
  let ty = RecordContentType::try_from(a)?;
  let protocol_version = <u16 as Decode<De>>::decode(&mut DecodeWrapper::from_bytes(&[b, c][..]))?;
  if ProtocolVersion::try_from(protocol_version).ok() != Some(ProtocolVersion::Tls12) {
    return unlikely_elem(Err(TlsError::UnsupportedTlsVersion.into()));
  }
  let len = <u16 as Decode<De>>::decode(&mut DecodeWrapper::from_bytes(&[d, e][..]))?;
  read_payload((0, len.into()), network_buffer, &mut read, stream_reader).await?;
  Ok(ty)
}

#[inline]
pub(crate) fn u8_chunk<'de, T>(
  dw: &mut DecodeWrapper<'de>,
  err: TlsError,
  cb: impl FnOnce(&mut DecodeWrapper<'de>) -> crate::Result<T>,
) -> crate::Result<T> {
  chunk::<u8, T>(dw, err, cb)
}

#[inline]
pub(crate) fn u8_list<'de, B, T>(
  buffer: &mut B,
  dw: &mut DecodeWrapper<'de>,
  err: TlsError,
) -> crate::Result<()>
where
  B: TryExtend<[T; 1]>,
  T: Decode<'de, De>,
{
  chunk::<u8, _>(dw, err, |local_dw| {
    while !local_dw.bytes().is_empty() {
      buffer.try_extend([T::decode(local_dw)?])?;
    }
    Ok(())
  })
}

#[inline]
pub(crate) fn u16_chunk<'de, T>(
  dw: &mut DecodeWrapper<'de>,
  err: TlsError,
  cb: impl FnOnce(&mut DecodeWrapper<'de>) -> crate::Result<T>,
) -> crate::Result<T> {
  chunk::<u16, T>(dw, err, cb)
}

#[inline]
pub(crate) fn u16_list<'de, B, T>(
  buffer: &mut B,
  dw: &mut DecodeWrapper<'de>,
  err: TlsError,
) -> crate::Result<()>
where
  B: TryExtend<[T; 1]>,
  T: Decode<'de, De>,
{
  chunk::<u16, _>(dw, err, |dw| {
    while !dw.bytes().is_empty() {
      buffer.try_extend([T::decode(dw)?])?;
    }
    Ok(())
  })
}

#[inline]
pub(crate) fn u16_list_bytes<'de, B>(
  buffer: &mut B,
  dw: &mut DecodeWrapper<'de>,
  err: TlsError,
) -> crate::Result<()>
where
  B: TryExtend<&'de [u8]>,
{
  chunk::<u16, _>(dw, err, |dw| {
    while !dw.bytes().is_empty() {
      let len = <u16 as Decode<'de, De>>::decode(dw)?;
      let Some((lhs, rhs)) = dw.bytes().split_at_checked(len.into()) else {
        return Err(TlsError::InvalidArray.into());
      };
      *dw.bytes_mut() = rhs;
      buffer.try_extend(lhs)?;
    }
    Ok(())
  })
}

#[inline]
pub(crate) fn u24_chunk<'de, T>(
  dw: &mut DecodeWrapper<'de>,
  err: TlsError,
  cb: impl FnOnce(&mut DecodeWrapper<'de>) -> crate::Result<T>,
) -> crate::Result<T>
where
  T: Decode<'de, De>,
{
  chunk::<U24, T>(dw, err, cb)
}

#[inline]
pub(crate) fn u24_list<'de, B, T>(
  buffer: &mut B,
  dw: &mut DecodeWrapper<'de>,
  err: TlsError,
) -> crate::Result<()>
where
  B: TryExtend<[T; 1]>,
  T: Decode<'de, De>,
{
  chunk::<U24, _>(dw, err, |dw| {
    while !dw.bytes().is_empty() {
      buffer.try_extend([T::decode(dw)?])?;
    }
    Ok(())
  })
}

#[inline]
fn chunk<'de, L, T>(
  dw: &mut DecodeWrapper<'de>,
  err: TlsError,
  cb: impl FnOnce(&mut DecodeWrapper<'de>) -> crate::Result<T>,
) -> crate::Result<T>
where
  L: Decode<'de, De> + Into<usize>,
{
  let len: L = Decode::<'_, De>::decode(dw)?;
  let Some((before, after)) = dw.bytes().split_at_checked(len.into()) else {
    return Err(err.into());
  };
  *dw.bytes_mut() = before;
  let rslt = cb(dw)?;
  *dw.bytes_mut() = after;
  if !before.is_empty() {
    return Err(err.into());
  }
  Ok(rslt)
}
