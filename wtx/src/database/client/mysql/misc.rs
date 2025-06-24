use crate::{
  collection::{ArrayVector, Vector},
  database::client::mysql::{
    DbError, MysqlError,
    mysql_executor::MAX_PAYLOAD,
    mysql_protocol::{
      MysqlProtocol, decode_wrapper_protocol::DecodeWrapperProtocol,
      encode_wrapper_protocol::EncodeWrapperProtocol, packet_req::PacketReq,
    },
  },
  de::{Decode, Encode},
  misc::{
    Usize,
    hints::unlikely_elem,
    net::{PartitionedFilledBuffer, read_header, read_payload},
  },
  stream::Stream,
};
use core::marker::PhantomData;

pub(crate) fn decode<'de, DO, E, T>(bytes: &mut &'de [u8], other: DO) -> Result<T, E>
where
  E: From<crate::Error>,
  T: Decode<'de, MysqlProtocol<DO, E>>,
{
  T::decode(&mut (), &mut DecodeWrapperProtocol { bytes, other })
}

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

pub(crate) async fn fetch_msg<S>(
  capabilities: u64,
  pfb: &mut PartitionedFilledBuffer,
  sequence_id: &mut u8,
  stream: &mut S,
) -> crate::Result<usize>
where
  S: Stream,
{
  let mut total: usize = 0;
  let (payload_len, local_sequence_id) = fetch_one_msg(pfb, stream).await?;
  total = total.wrapping_add(payload_len.wrapping_add(4));
  let first_byte = pfb.current().first().copied();
  if payload_len == Usize::from(MAX_PAYLOAD).into_usize() {
    return Err(crate::Error::from(MysqlError::UnsupportedPayloadLen));
  }
  *sequence_id = local_sequence_id;
  if first_byte == Some(255) {
    return unlikely_elem({
      let db_error: crate::Result<DbError> = decode(&mut pfb.current(), capabilities);
      Err(db_error?.into())
    });
  }
  Ok(total)
}

pub(crate) async fn fetch_protocol<'de, S, T>(
  capabilities: u64,
  pfb: &'de mut PartitionedFilledBuffer,
  sequence_id: &mut u8,
  stream: &mut S,
) -> crate::Result<(T, usize)>
where
  S: Stream,
  T: for<'any> Decode<'de, MysqlProtocol<(), crate::Error>>,
{
  let total = fetch_msg(capabilities, pfb, sequence_id, stream).await?;
  Ok((
    T::decode(&mut (), &mut DecodeWrapperProtocol { bytes: &mut pfb.current(), other: () })?,
    total,
  ))
}

pub(crate) async fn send_packet<E, S, T>(
  (capabilities, sequence_id): (&mut u64, &mut u8),
  encode_buffer: &mut Vector<u8>,
  payload: T,
  stream: &mut S,
) -> Result<(), E>
where
  E: From<crate::Error>,
  S: Stream,
  T: Encode<MysqlProtocol<(), E>>,
{
  *sequence_id = 0;
  write_packet((capabilities, sequence_id), encode_buffer, payload, stream).await
}

// Only used in the connection phase
pub(crate) async fn write_packet<E, S, T>(
  (capabilities, sequence_id): (&mut u64, &mut u8),
  encode_buffer: &mut Vector<u8>,
  payload: T,
  stream: &mut S,
) -> Result<(), E>
where
  E: From<crate::Error>,
  S: Stream,
  T: Encode<MysqlProtocol<(), E>>,
{
  encode_buffer.clear();
  let mut ew = EncodeWrapperProtocol::new(capabilities, encode_buffer);
  PacketReq(payload, PhantomData).encode_and_write(&mut ew, sequence_id, stream).await?;
  Ok(())
}

async fn fetch_one_msg<S>(
  pfb: &mut PartitionedFilledBuffer,
  stream: &mut S,
) -> crate::Result<(usize, u8)>
where
  S: Stream,
{
  pfb.reserve(4)?;
  let mut read = pfb.following_len();
  let buffer = pfb.following_rest_mut();
  let [a0, b0, c0, sequence_id] = read_header::<0, 4, S>(buffer, &mut read, stream).await?;
  let payload_len = Usize::from(u32::from_le_bytes([a0, b0, c0, 0])).into_usize();
  read_payload((4, payload_len), pfb, &mut read, stream).await?;
  Ok((payload_len, sequence_id.wrapping_add(1)))
}
