use crate::{
  database::client::mysql::mysql_protocol::{
    MysqlProtocol, decode_wrapper_protocol::DecodeWrapperProtocol,
    encode_wrapper_protocol::EncodeWrapperProtocol, packet_req::PacketReq,
  },
  misc::{
    _read_header, _read_payload, ArrayVector, Decode, Encode, Stream, Usize, Vector,
    partitioned_filled_buffer::PartitionedFilledBuffer,
  },
};
use core::marker::PhantomData;

#[inline]
pub(crate) fn decode<'de, DO, E, T>(bytes: &mut &'de [u8], other: DO) -> Result<T, E>
where
  E: From<crate::Error>,
  T: Decode<'de, MysqlProtocol<DO, E>>,
{
  T::decode(&mut (), &mut DecodeWrapperProtocol { bytes, other })
}

#[inline]
pub(crate) fn encoded_len(len: usize) -> crate::Result<ArrayVector<u8, 9>> {
  let [a, b, c, d, e, f, g, h] = len.to_le_bytes();
  let mut rslt = ArrayVector::new();
  match len {
    0..=250 => rslt.push(a)?,
    251..=65535 => {
      rslt.push(252)?;
      rslt.extend_from_copyable_slice(&[a, b])?;
    }
    65536..=16777215 => {
      rslt.push(253)?;
      rslt.extend_from_copyable_slice(&[a, b, c])?;
    }
    _ => {
      rslt.push(254)?;
      rslt.extend_from_copyable_slice(&[a, b, c, d, e, f, g, h])?;
    }
  }
  Ok(rslt)
}

#[inline]
pub(crate) async fn fetch_msg<S>(
  pfb: &mut PartitionedFilledBuffer,
  sequence_id: &mut u8,
  stream: &mut S,
) -> crate::Result<()>
where
  S: Stream,
{
  let (mut len, mut local_sequence_id) = fetch_one_msg(pfb, stream).await?;
  while len == 0xFF_FF_FF {
    let (local_len, local_local_sequence_id) = fetch_one_msg(pfb, stream).await?;
    len = local_len;
    local_sequence_id = local_local_sequence_id;
  }
  *sequence_id = local_sequence_id;
  Ok(())
}

#[inline]
pub(crate) async fn fetch_protocol<'de, S, T>(
  pfb: &'de mut PartitionedFilledBuffer,
  sequence_id: &mut u8,
  stream: &mut S,
) -> crate::Result<T>
where
  S: Stream,
  T: for<'any> Decode<'de, MysqlProtocol<(), crate::Error>>,
{
  fetch_msg(pfb, sequence_id, stream).await?;
  T::decode(&mut (), &mut DecodeWrapperProtocol { bytes: &mut pfb._current(), other: () })
}

#[inline]
pub(crate) async fn send_packet<E, S, T>(
  (capabilities, sequence_id): (&mut u64, &mut u8),
  enc_buffer: &mut Vector<u8>,
  payload: T,
  stream: &mut S,
) -> Result<(), E>
where
  E: From<crate::Error>,
  S: Stream,
  T: Encode<MysqlProtocol<(), E>>,
{
  *sequence_id = 0;
  write_packet((capabilities, sequence_id), enc_buffer, payload, stream).await
}

// Only used in the connection phase
#[inline]
pub(crate) async fn write_packet<E, S, T>(
  (capabilities, sequence_id): (&mut u64, &mut u8),
  enc_buffer: &mut Vector<u8>,
  payload: T,
  stream: &mut S,
) -> Result<(), E>
where
  E: From<crate::Error>,
  S: Stream,
  T: Encode<MysqlProtocol<(), E>>,
{
  let mut ew = EncodeWrapperProtocol::new(capabilities, enc_buffer);
  PacketReq(payload, PhantomData).encode_and_write(&mut ew, sequence_id, stream).await?;
  Ok(())
}

#[inline]
async fn fetch_one_msg<S>(
  pfb: &mut PartitionedFilledBuffer,
  stream: &mut S,
) -> crate::Result<(usize, u8)>
where
  S: Stream,
{
  pfb._reserve(4)?;
  let mut read = pfb._following_len();
  let buffer = pfb._following_rest_mut();
  let [a0, b0, c0, sequence_id] = _read_header::<0, 4, S>(buffer, &mut read, stream).await?;
  let len = Usize::from(u32::from_le_bytes([a0, b0, c0, 0])).into_usize();
  _read_payload((4, len), pfb, &mut read, stream).await?;
  Ok((len, sequence_id.wrapping_add(1)))
}
