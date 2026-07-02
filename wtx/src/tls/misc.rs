use core::{hint::cold_path, num::NonZeroUsize};

use crate::{
  codec::Decode,
  collections::{ArrayVectorCopy, MaybeUninitSlice, ShortBoxSliceU16, TryExtend, Vector},
  crypto::AEAD_TAG_LEN,
  misc::{FnMutFut, TryArithmetic as _, unlikely_elem},
  stream::{BufStreamReader, StreamReadItem, StreamReader, StreamWriter},
  tls::{
    SERVER_SIG_CTX, TlsError, TlsMode,
    de::De,
    key_schedule::{KeyScheduleRead, KeyScheduleState, KeyScheduleWrite},
    protocol::{
      alert::Alert,
      handshake::{Handshake, HandshakeType},
      key_update::{KeyUpdate, KeyUpdateRequest},
      new_session_ticket::NewSessionTicket,
      protocol_version::ProtocolVersion,
      record_content_type::RecordContentType,
      u24::U24,
    },
    read_record_info::ReadRecordInfo,
    tls_decode_wrapper::TlsDecodeWrapper,
  },
};

pub(crate) fn build_header(ty: RecordContentType, len: u16) -> [u8; 5] {
  let [b0, n1] = len.to_be_bytes();
  [ty.into(), 3, 3, b0, n1]
}

pub(crate) fn duplicated_error(is_some: bool) -> crate::Result<()> {
  if is_some {
    return Err(TlsError::DuplicatedClientHelloParameters.into());
  }
  Ok(())
}

pub(crate) async fn fetch_rec_from_stream<SR>(
  kss: Option<&mut KeyScheduleState>,
  max_fragment_length: u16,
  reader_buffer: &mut BufStreamReader,
  stream_reader: &mut SR,
) -> crate::Result<StreamReadItem<ReadRecordInfo>>
where
  SR: StreamReader,
{
  let Some(header) = reader_buffer.read_header::<_, 5>(stream_reader).await?.opt() else {
    return Ok(StreamReadItem::empty_cold());
  };
  let [b0, b1, b2, b3, b4] = header;
  let outer_ty = RecordContentType::try_from(b0)?;
  let protocol_version = <u16 as Decode<De>>::decode(&mut TlsDecodeWrapper::from_bytes(&[b1, b2]))?;
  if ProtocolVersion::try_from(protocol_version).ok() != Some(ProtocolVersion::Tls12) {
    return unlikely_elem(Err(TlsError::UnsupportedTlsVersion.into()));
  }
  let len = <u16 as Decode<De>>::decode(&mut TlsDecodeWrapper::from_bytes(&[b3, b4]))?;
  if len > max_fragment_length {
    cold_path();
    return Err(TlsError::ReceivedRecordIsTooLarge.into());
  }
  if reader_buffer.read_payload(len.into(), stream_reader).await?.is_closed() {
    return Ok(StreamReadItem::empty_cold());
  }
  let mut trails: u16 = 0;
  let inner_ty = if let Some(elem) = kss {
    elem.increment_counter();
    let nonce = elem.nonce();
    let secret = elem.cipher_key();
    let record = reader_buffer.current_mut();
    let _ = elem.cipher_suite().aes_decrypt(&header, record, nonce, secret)?;
    let [plaintext @ .., maybe_ty, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _] = record else {
      return Err(crate::crypto::CryptoError::InvalidAesData.into());
    };
    trails = 17;
    if *maybe_ty == 0 {
      let mut inner_ty = 0;
      if let Some(idx) = plaintext.iter().rposition(|el| *el != 0) {
        inner_ty = plaintext.get(idx).copied().unwrap_or_default();
        let local_len = plaintext.len().wrapping_sub(idx);
        trails = trails.try_add(local_len.try_into()?)?;
      }
      RecordContentType::try_from(inner_ty)?
    } else {
      RecordContentType::try_from(*maybe_ty)?
    }
  } else {
    outer_ty
  };
  let plaintext_len = reader_buffer.current().len().wrapping_sub(trails.into());
  Ok(StreamReadItem::from_item(ReadRecordInfo { inner_ty, outer_ty, plaintext_len }))
}

#[inline]
pub(crate) async fn read_after_handshake_data<A, SR, TM, const IS_CLIENT: bool>(
  mut aux: A,
  mut bytes: MaybeUninitSlice<'_, u8>,
  ksr: &mut KeyScheduleRead,
  max_fragment_length: u16,
  new_session_ticket: &mut Option<NewSessionTicket<ShortBoxSliceU16<u8>>>,
  plaintext_consumed: &mut usize,
  plaintext_len: &mut usize,
  reader_buffer: &mut BufStreamReader,
  stream_reader: &mut SR,
  mut alert_cb: impl for<'any> FnMutFut<(&'any mut A, Alert, &'any mut SR), Result = crate::Result<()>>,
  mut key_update_cb: impl for<'any> FnMutFut<
    (&'any mut A, KeyUpdate, &'any mut SR),
    Result = crate::Result<()>,
  >,
) -> crate::Result<StreamReadItem<NonZeroUsize>>
where
  SR: StreamReader,
  TM: TlsMode,
{
  if TM::TY.is_plain_text() {
    return stream_reader.read(bytes).await;
  }
  if let Some(1..=usize::MAX) = plaintext_len.checked_sub(*plaintext_consumed) {
    return Ok(transfer_after_handshake_data(
      &mut bytes,
      reader_buffer.current().get(*plaintext_consumed..*plaintext_len).unwrap_or_default(),
      |len| *plaintext_consumed = plaintext_consumed.wrapping_add(len.get()),
    ));
  }
  loop {
    let Some(rri) = fetch_rec_from_stream(
      Some(ksr.state_mut()),
      max_fragment_length,
      reader_buffer,
      stream_reader,
    )
    .await?
    .opt() else {
      return Ok(StreamReadItem::empty_cold());
    };
    let RecordContentType::ApplicationData = rri.outer_ty else {
      cold_path();
      return Err(TlsError::UnexpectedAfterHandshakeOuterRecord.into());
    };
    let plaintext = reader_buffer.current().get(..*plaintext_len).unwrap_or_default();
    match rri.inner_ty {
      RecordContentType::Alert => {
        cold_path();
        let alert = Alert::decode(&mut TlsDecodeWrapper::from_bytes(plaintext))?;
        alert_cb.call((&mut aux, alert, stream_reader)).await?;
        return Ok(StreamReadItem::empty_cold());
      }
      RecordContentType::ApplicationData => {
        *plaintext_len = rri.plaintext_len;
        return Ok(transfer_after_handshake_data(&mut bytes, plaintext, |len| {
          *plaintext_consumed = len.get();
        }));
      }
      RecordContentType::ChangeCipherSpec => {
        cold_path();
        return Err(TlsError::UnexpectedAfterHandshakeInnerRecord.into());
      }
      RecordContentType::Handshake => {
        cold_path();
        let hs = Handshake::<&[u8]>::decode(&mut TlsDecodeWrapper::from_bytes(plaintext))?;
        match hs.msg_type {
          HandshakeType::KeyUpdate => {
            let remote_key_update = KeyUpdate::decode(&mut TlsDecodeWrapper::from_bytes(hs.data))?;
            ksr.state_mut().rotate()?;
            if matches!(remote_key_update.request_update, KeyUpdateRequest::UpdateRequested) {
              let local_key_update =
                KeyUpdate { request_update: KeyUpdateRequest::UpdateNotRequested };
              key_update_cb.call((&mut aux, local_key_update, stream_reader)).await?;
            }
          }
          HandshakeType::NewSessionTicket => {
            if !IS_CLIENT {
              return Err(TlsError::UnexpectedAfterHandshakeInnerRecord.into());
            }
            let dw = &mut TlsDecodeWrapper::from_bytes(plaintext);
            *new_session_ticket = Some(NewSessionTicket::decode(dw)?);
          }
          HandshakeType::Certificate
          | HandshakeType::CertificateRequest
          | HandshakeType::CertificateVerify
          | HandshakeType::ClientHello
          | HandshakeType::EncryptedExtensions
          | HandshakeType::EndOfEarlyData
          | HandshakeType::Finished
          | HandshakeType::MessageHash
          | HandshakeType::ServerHello => {
            return Err(TlsError::UnexpectedAfterHandshakeInnerRecord.into());
          }
        }
      }
    }
  }
}

pub(crate) fn server_sig_msg(transcript: &[u8]) -> crate::Result<ArrayVectorCopy<u8, 146>> {
  let mut msg = ArrayVectorCopy::<u8, 146>::from_array([b' '; 64]);
  let _ = msg.extend_from_copyable_slices([SERVER_SIG_CTX.as_bytes(), transcript])?;
  Ok(msg)
}

#[inline(always)]
fn transfer_after_handshake_data(
  bytes: &mut MaybeUninitSlice<'_, u8>,
  plaintext: &[u8],
  non_empty_cb: impl FnOnce(NonZeroUsize),
) -> StreamReadItem<NonZeroUsize> {
  let all_mut = bytes.all_mut();
  let all_mut_len = all_mut.len();
  let plaintext_len = plaintext.len();
  if let Some(all_mut_partial) = all_mut.get_mut(..plaintext_len) {
    let _ = all_mut_partial.write_copy_of_slice(plaintext);
    // SAFETY: `plaintext` is always is a non-empty slice
    let len = unsafe { NonZeroUsize::new_unchecked(plaintext_len) };
    non_empty_cb(len);
    return StreamReadItem::from_item(len);
  }
  if let Some(plaintext_partial @ [_not_empty, ..]) = plaintext.get(..all_mut_len) {
    let _ = all_mut.write_copy_of_slice(plaintext_partial);
    // SAFETY: The above check just confirmed that plaintext_len is greater than zero
    let len = unsafe { NonZeroUsize::new_unchecked(plaintext_len) };
    non_empty_cb(len);
    return StreamReadItem::from_item(len);
  }
  StreamReadItem::empty_cold()
}

#[inline]
pub(crate) fn u8_chunk<'de, T>(
  dw: &mut TlsDecodeWrapper<'de>,
  err: TlsError,
  cb: impl FnOnce(&mut TlsDecodeWrapper<'de>) -> crate::Result<T>,
) -> crate::Result<T> {
  chunk::<u8, T>(dw, err, cb)
}

#[inline]
pub(crate) fn u8_list<'de, B, T>(
  buffer: &mut B,
  dw: &mut TlsDecodeWrapper<'de>,
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
  dw: &mut TlsDecodeWrapper<'de>,
  err: TlsError,
  cb: impl FnOnce(&mut TlsDecodeWrapper<'de>) -> crate::Result<T>,
) -> crate::Result<T> {
  chunk::<u16, T>(dw, err, cb)
}

#[inline]
pub(crate) fn u16_list<'de, B, T>(
  buffer: &mut B,
  dw: &mut TlsDecodeWrapper<'de>,
  err: TlsError,
) -> crate::Result<()>
where
  B: TryExtend<[T; 1]>,
  T: Decode<'de, De>,
{
  chunk::<u16, _>(dw, err, |local_dw| {
    while !local_dw.bytes().is_empty() {
      buffer.try_extend([T::decode(local_dw)?])?;
    }
    Ok(())
  })
}

#[inline]
pub(crate) fn u24_chunk<'de, T>(
  dw: &mut TlsDecodeWrapper<'de>,
  err: TlsError,
  cb: impl FnOnce(&mut TlsDecodeWrapper<'de>) -> crate::Result<T>,
) -> crate::Result<T>
where
  T: Decode<'de, De>,
{
  chunk::<U24, T>(dw, err, cb)
}

#[inline]
pub(crate) fn u24_list<'de, B, T>(
  buffer: &mut B,
  dw: &mut TlsDecodeWrapper<'de>,
  err: TlsError,
) -> crate::Result<()>
where
  B: TryExtend<[T; 1]>,
  T: Decode<'de, De>,
{
  chunk::<U24, _>(dw, err, |local_dw| {
    while !local_dw.bytes().is_empty() {
      buffer.try_extend([T::decode(local_dw)?])?;
    }
    Ok(())
  })
}

#[inline]
pub(crate) async fn write_data<SW>(
  bytes: &[&[u8]],
  ksw: &mut KeyScheduleWrite,
  max_fragment_length: u16,
  stream_writer: &mut SW,
  writer_buffer: &mut Vector<u8>,
) -> crate::Result<()>
where
  SW: StreamWriter,
{
  let idx = writer_buffer.len();
  for data in bytes {
    for chunk in data.chunks(max_fragment_length.into()) {
      let len = chunk.len().try_into().unwrap_or_default();
      let header = build_header(RecordContentType::ApplicationData, len);
      let plaintext_begin_idx = writer_buffer.len().wrapping_add(header.len());
      let _ = writer_buffer.extend_from_copyable_slices([
        header.as_slice(),
        chunk,
        &[RecordContentType::ApplicationData.into()],
        &[0; AEAD_TAG_LEN],
      ])?;
      let plaintext_len = chunk.len().wrapping_add(1);
      let plaintext = writer_buffer
        .get_mut(plaintext_begin_idx..plaintext_begin_idx.wrapping_add(plaintext_len))
        .unwrap_or_default();
      let ksw_state = ksw.state_mut();
      let nonce = ksw_state.nonce();
      let secret = ksw_state.cipher_key();
      let tag = ksw_state.cipher_suite().aes_encrypt(&header, plaintext, nonce, secret)?;
      if let Some(buffer_tag) = writer_buffer.last_chunk_mut::<AEAD_TAG_LEN>() {
        buffer_tag.copy_from_slice(&tag);
      }
      ksw_state.increment_counter();
    }
  }
  stream_writer.write_all(writer_buffer.get(idx..).unwrap_or_default()).await?;
  writer_buffer.clear();
  Ok(())
}

#[inline]
fn chunk<'de, L, T>(
  dw: &mut TlsDecodeWrapper<'de>,
  err: TlsError,
  cb: impl FnOnce(&mut TlsDecodeWrapper<'de>) -> crate::Result<T>,
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
