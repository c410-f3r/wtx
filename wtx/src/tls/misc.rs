use crate::{
  codec::Decode,
  collections::{ArrayVectorCopy, MaybeUninitSlice, ShortBoxSliceU16, TryExtend, Vector},
  crypto::AEAD_TAG_LEN,
  futures::FnMutFut,
  misc::{TryArithmetic as _, unlikely_elem},
  net::{BufStreamReader, StreamReader, StreamWriter},
  tls::{
    AlertDescription, CHANGE_CIPHER_SPEC, RECORD_HEADER_LEN, SERVER_SIG_CTX, TlsError,
    de::De,
    key_schedule::{KeyScheduleRead, KeyScheduleState, KeyScheduleWrite},
    protocol::{
      alert::Alert,
      extension_ty::ExtensionTy,
      handshake::Handshake,
      handshake_ty::HandshakeTy,
      key_update::{KeyUpdate, KeyUpdateRequest},
      new_session_ticket::NewSessionTicket,
      protocol_version::ProtocolVersion,
      record_content_ty::RecordContentTy,
      u24::U24,
    },
    read_record_info::ReadRecordInfo,
    tls_decode_wrapper::TlsDecodeWrapper,
  },
};
use core::{hint::cold_path, num::NonZeroUsize, ops::Range};

const UNEXPECTED_AFTER_HANDSHAKE_INNER_RECORD: crate::Error = crate::Error::TlsErrorReply(
  TlsError::UnexpectedAfterHandshakeInnerRecord,
  AlertDescription::UnexpectedMessage,
);

pub(crate) fn build_header(ty: RecordContentTy, len: u16) -> [u8; RECORD_HEADER_LEN] {
  let [b0, n1] = len.to_be_bytes();
  [ty.into(), 3, 3, b0, n1]
}

pub(crate) fn decode_extension_ty(
  dw: &mut TlsDecodeWrapper<'_>,
  err: TlsError,
  seen_unknowns: &mut ArrayVectorCopy<u16, 9>,
) -> crate::Result<Option<ExtensionTy>> {
  let tag: u16 = Decode::<'_, De>::decode(dw)?;
  if let Ok(el) = ExtensionTy::try_from(tag) {
    Ok(Some(el))
  } else {
    if seen_unknowns.contains(&tag) {
      return Err(crate::Error::TlsErrorReply(
        TlsError::DuplicatedClientHelloParameters,
        AlertDescription::DecodeError,
      ));
    }
    seen_unknowns.push(tag).map_err(|_err| TlsError::UnknownsOverflow)?;
    u16_chunk(dw, err, |_bytes| Ok(()))?;
    Ok(None)
  }
}

#[inline]
pub(crate) async fn fetch_rec_from_stream<SR, const CHECK_CCS: bool, const IS_CH: bool>(
  kss: Option<&mut KeyScheduleState>,
  max_fragment_length: u16,
  reader_buffer: &mut BufStreamReader,
  stream_reader: &mut SR,
) -> crate::Result<Option<ReadRecordInfo>>
where
  SR: StreamReader,
{
  let Some(header) = reader_buffer.read_header::<_, 5>(stream_reader).await? else {
    return Ok(None);
  };
  if CHECK_CCS && header == [RecordContentTy::ChangeCipherSpec.into(), 3, 3, 0, 1] {
    reader_buffer.read_payload(1, stream_reader).await?;
    return Ok(Some(ReadRecordInfo {
      inner_ty: RecordContentTy::ChangeCipherSpec,
      outer_ty: RecordContentTy::ChangeCipherSpec,
      plaintext_len: 1,
    }));
  }
  let [b0, b1, b2, b3, b4] = header;
  let outer_ty = RecordContentTy::try_from(b0)?;
  let prot_version_num = <u16 as Decode<De>>::decode(&mut TlsDecodeWrapper::from_bytes(&[b1, b2]))?;
  if IS_CH {
    if b1 != 3 {
      return Err(crate::Error::TlsError(TlsError::UnknownProtocolVersion));
    }
  } else {
    let protocol_version = ProtocolVersion::try_from(prot_version_num)?;
    if protocol_version != ProtocolVersion::Tls12 {
      return unlikely_elem(Err(TlsError::UnsupportedRecTlsVersion(protocol_version).into()));
    }
  }
  let len = <u16 as Decode<De>>::decode(&mut TlsDecodeWrapper::from_bytes(&[b3, b4]))?;
  let mut max_allowed_len = max_fragment_length;
  if kss.is_some() {
    max_allowed_len = max_allowed_len.saturating_add(256);
  }
  if len > max_allowed_len {
    cold_path();
    return Err(crate::Error::TlsErrorReply(
      TlsError::ReceivedRecordIsTooLarge,
      AlertDescription::RecordOverflow,
    ));
  }
  reader_buffer.read_payload(len.into(), stream_reader).await?;
  let mut trails: u16 = 0;
  let inner_ty = if let Some(elem) = kss {
    let nonce = elem.nonce();
    let secret = elem.cipher_key();
    let record = reader_buffer.current_mut();
    if elem.cipher_suite().aes_decrypt(&header, record, nonce, secret).is_err() {
      return Err(crate::Error::TlsErrorReply(
        TlsError::UnencryptedRecord,
        AlertDescription::BadRecordMac,
      ));
    }
    elem.increment_counter();
    let Some((plaintext, [maybe_ty, ..])) = record.split_last_chunk_mut::<17>() else {
      return Err(TlsError::InvalidAesData.into());
    };
    if plaintext.len() > max_fragment_length.into() {
      return Err(crate::Error::TlsErrorReply(
        TlsError::ReceivedRecordIsTooLarge,
        AlertDescription::RecordOverflow,
      ));
    }
    trails = 17;
    if *maybe_ty == 0 {
      let mut inner_ty = 0;
      if let Some(idx) = plaintext.iter().rposition(|el| *el != 0) {
        inner_ty = plaintext.get(idx).copied().unwrap_or_default();
        let local_len = plaintext.len().wrapping_sub(idx);
        trails = trails.try_add(local_len.try_into()?)?;
      }
      RecordContentTy::try_from(inner_ty)?
    } else {
      RecordContentTy::try_from(*maybe_ty)?
    }
  } else {
    outer_ty
  };
  let plaintext_len = reader_buffer.current().len().wrapping_sub(trails.into());
  let rri = ReadRecordInfo { inner_ty, outer_ty, plaintext_len };
  _trace!(target: crate::tls::_TARGET, "Read Record: {:?}", &rri);
  Ok(Some(rri))
}

#[inline]
pub(crate) fn handshake_bytes_adjust(
  reader_buffer: &mut BufStreamReader,
  rri: &ReadRecordInfo,
  (split_begin, split_len): (&mut usize, &mut usize),
) {
  let ant_end_idx = reader_buffer.antecedent_end_idx();
  if *split_len == 0 {
    *split_begin = ant_end_idx;
    *split_len = rri.plaintext_len;
  } else {
    cold_path();
    let dest = split_begin.wrapping_add(*split_len);
    let src_begin = ant_end_idx;
    let src_end = ant_end_idx.wrapping_add(rri.plaintext_len);
    reader_buffer.buffer_mut().copy_within(src_begin..src_end, dest);
    *split_len = split_len.wrapping_add(rri.plaintext_len);
  }
}

#[inline]
pub(crate) fn handshake_bytes_decode<'rb>(
  reader_buffer: &'rb BufStreamReader,
  (split_begin, split_len): (&mut usize, &mut usize),
) -> crate::Result<Option<(HandshakeTy, Range<usize>, TlsDecodeWrapper<'rb>)>> {
  let end_idx = split_begin.wrapping_add(*split_len);
  let plaintext = reader_buffer.filled().get(*split_begin..end_idx).unwrap_or_default();
  if plaintext.len() < Handshake::<()>::HEADER_LEN {
    cold_path();
    return Ok(None);
  }
  let mut dw = TlsDecodeWrapper::from_bytes(plaintext);
  let msg_type = HandshakeTy::try_from(<u8 as Decode<De>>::decode(&mut dw)?)?;
  let payload_len: usize = <U24 as Decode<'_, De>>::decode(&mut dw)?.into();
  let rec_len = Handshake::<()>::HEADER_LEN.wrapping_add(payload_len);
  if *split_len < rec_len {
    cold_path();
    return Ok(None);
  }
  *dw.bytes_mut() = dw.bytes().get(..payload_len).unwrap_or_default();
  let rec_range = *split_begin..split_begin.wrapping_add(rec_len);
  *split_begin = split_begin.wrapping_add(rec_len);
  *split_len = split_len.wrapping_sub(rec_len);
  Ok(Some((msg_type, rec_range, dw)))
}

pub(crate) async fn manage_err<SW, T>(
  has_sent_ccs: bool,
  kss: &mut KeyScheduleState,
  rslt: crate::Result<T>,
  stream_writer: &mut SW,
) -> crate::Result<T>
where
  SW: StreamWriter,
{
  match rslt {
    Err(err @ crate::Error::TlsErrorReply(_, description)) => {
      cold_path();
      if kss.cipher_key().is_empty() {
        let alert = Alert::fatal(description).record_bytes_unencrypted();
        stream_writer.write_all(&alert[..]).await?;
      } else {
        let alert = Alert::fatal(description).record_bytes(kss)?;
        if has_sent_ccs {
          stream_writer.write_all(&alert[..]).await?;
        } else {
          stream_writer.write_all_vectored(&[&CHANGE_CIPHER_SPEC[..], &alert[..]]).await?;
        }
      }
      Err(err)
    }
    Ok(elem) => Ok(elem),
    Err(err) => Err(err),
  }
}

pub(crate) fn post_handshake_dec_error(
  after_bytes: &[u8],
  handshake_ty: HandshakeTy,
) -> crate::Result<()> {
  if !after_bytes.is_empty() {
    return Err(crate::Error::TlsErrorReply(
      TlsError::PostHandshakeDecError(handshake_ty),
      if handshake_ty.is_finished() {
        AlertDescription::DecryptError
      } else {
        AlertDescription::DecodeError
      },
    ));
  }
  Ok(())
}

pub(crate) fn pre_handshake_dec_error(condition: bool) -> crate::Result<()> {
  if condition {
    return Err(crate::Error::TlsErrorReply(
      TlsError::PreHandshakeDecError,
      AlertDescription::UnexpectedMessage,
    ));
  }
  Ok(())
}

#[inline]
pub(crate) async fn read_after_handshake_data<A, SR, const IS_CLIENT: bool>(
  mut aux: A,
  mut bytes: MaybeUninitSlice<'_, u8>,
  ksr: &mut KeyScheduleRead,
  max_fragment_length: u16,
  new_session_ticket: &mut Option<NewSessionTicket<ShortBoxSliceU16<u8>>>,
  plaintext_consumed: &mut usize,
  plaintext_len: &mut usize,
  reader_buffer: &mut BufStreamReader,
  split_begin: &mut usize,
  split_len: &mut usize,
  stream_reader: &mut SR,
  mut alert_cb: impl for<'any> FnMutFut<
    (&'any mut A, Alert, &'any mut SR),
    Result = crate::Result<bool>,
  >,
  closed_conn_cb: impl FnOnce(&mut A),
  mut key_update_cb: impl for<'any> FnMutFut<
    (&'any mut A, Option<KeyUpdate>, &'any mut SR),
    Result = crate::Result<()>,
  >,
) -> crate::Result<Option<NonZeroUsize>>
where
  SR: StreamReader,
{
  if let Some(1..=usize::MAX) = plaintext_len.checked_sub(*plaintext_consumed) {
    return Ok(transfer_after_handshake_data(
      &mut bytes,
      reader_buffer.current().get(*plaintext_consumed..*plaintext_len).unwrap_or_default(),
      |len| *plaintext_consumed = plaintext_consumed.wrapping_add(len.get()),
    ));
  }
  loop {
    let Some(rri) = fetch_rec_from_stream::<_, false, false>(
      Some(ksr.state_mut()),
      max_fragment_length,
      reader_buffer,
      stream_reader,
    )
    .await?
    else {
      cold_path();
      closed_conn_cb(&mut aux);
      return Ok(None);
    };
    let RecordContentTy::ApplicationData = rri.outer_ty else {
      cold_path();
      return Err(TlsError::UnexpectedAfterHandshakeOuterRecord.into());
    };
    let plaintext = reader_buffer.current().get(..rri.plaintext_len).unwrap_or_default();
    match rri.inner_ty {
      RecordContentTy::Alert => {
        cold_path();
        let alert = Alert::decode(&mut TlsDecodeWrapper::from_bytes(plaintext))?;
        if alert_cb.call((&mut aux, alert, stream_reader)).await? {
          return Ok(None);
        }
      }
      RecordContentTy::ApplicationData => {
        *plaintext_len = rri.plaintext_len;
        let written = transfer_after_handshake_data(&mut bytes, plaintext, |len| {
          *plaintext_consumed = len.get();
        });
        return Ok(written);
      }
      RecordContentTy::ChangeCipherSpec => {
        cold_path();
        return Err(UNEXPECTED_AFTER_HANDSHAKE_INNER_RECORD);
      }
      RecordContentTy::Handshake => {
        cold_path();
        *reader_buffer.forbid_clear_mut() = true;
        handshake_bytes_adjust(reader_buffer, &rri, (split_begin, split_len));
        while let Some(tuple) = handshake_bytes_decode(reader_buffer, (split_begin, split_len))? {
          let (msg_type, _range, mut dw) = tuple;
          match msg_type {
            HandshakeTy::KeyUpdate => {
              let remote_ku = KeyUpdate::decode(&mut dw)?;
              ksr.state_mut().rotate()?;
              let resend = matches!(remote_ku.request_update, KeyUpdateRequest::UpdateRequested);
              key_update_cb
                .call((
                  &mut aux,
                  resend.then_some(KeyUpdate::new(KeyUpdateRequest::UpdateNotRequested)),
                  stream_reader,
                ))
                .await?;
            }
            HandshakeTy::NewSessionTicket => {
              manage_nst::<IS_CLIENT>(dw.bytes(), new_session_ticket)?;
            }
            HandshakeTy::Certificate
            | HandshakeTy::CertificateRequest
            | HandshakeTy::CertificateVerify
            | HandshakeTy::ClientHello
            | HandshakeTy::EncryptedExtensions
            | HandshakeTy::EndOfEarlyData
            | HandshakeTy::Finished
            | HandshakeTy::MessageHash
            | HandshakeTy::ServerHello => {
              return Err(UNEXPECTED_AFTER_HANDSHAKE_INNER_RECORD);
            }
          }
          *reader_buffer.forbid_clear_mut() = false;
          *split_begin = 0;
          *split_len = 0;
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

#[inline]
pub(crate) fn u8_chunk<'de, T>(
  dw: &mut TlsDecodeWrapper<'de>,
  err: TlsError,
  cb: impl FnOnce(&mut TlsDecodeWrapper<'de>) -> crate::Result<T>,
) -> crate::Result<T> {
  chunk::<u8, T>(dw, err, cb)
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
pub(crate) async fn write_payloads<SW>(
  inner_ty: RecordContentTy,
  ksw: &mut KeyScheduleWrite,
  max_fragment_length_send: u16,
  payloads: &[&[u8]],
  stream_writer: &mut SW,
  writer_buffer: &mut Vector<u8>,
) -> crate::Result<()>
where
  SW: StreamWriter,
{
  let total_len: usize = payloads.iter().map(|slice| slice.len()).sum();
  let mut total_unwritten = total_len;
  writer_buffer.reserve(total_len)?;
  let mut payloads_iter = payloads.iter().copied();
  let mut current_slice = payloads_iter.next().unwrap_or_default();
  while total_unwritten > 0 {
    let record_data_len = total_unwritten.min(max_fragment_length_send.into());
    total_unwritten = total_unwritten.wrapping_sub(record_data_len);
    let len_usize = record_data_len.wrapping_add(1).wrapping_add(AEAD_TAG_LEN);
    let len = len_usize.try_into().unwrap_or_default();
    let header = build_header(RecordContentTy::ApplicationData, len);
    let plaintext_begin_idx = writer_buffer.len().wrapping_add(header.len());
    writer_buffer.extend_from_copyable_slice(header.as_slice())?;
    let mut needed = record_data_len;
    while needed > 0 {
      if current_slice.is_empty() {
        current_slice = payloads_iter.next().unwrap_or_default();
      }
      let take = needed.min(current_slice.len());
      let Some((data, rest)) = current_slice.split_at_checked(take) else {
        break;
      };
      writer_buffer.extend_from_copyable_slice(data)?;
      current_slice = rest;
      needed = needed.wrapping_sub(take);
    }
    let array = [&[inner_ty.into()][..], &[0; AEAD_TAG_LEN]];
    let _ = writer_buffer.extend_from_copyable_slices(array)?;
    let plaintext_len = record_data_len.wrapping_add(1);
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
  stream_writer.write_all(writer_buffer).await?;
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
  Ok(rslt)
}

#[inline]
fn manage_nst<const IS_CLIENT: bool>(
  hs_data: &[u8],
  new_session_ticket: &mut Option<NewSessionTicket<crate::collections::ShortBoxSlice<u16, u8>>>,
) -> Result<(), crate::Error> {
  if !IS_CLIENT {
    return Err(UNEXPECTED_AFTER_HANDSHAKE_INNER_RECORD);
  }
  let dw = &mut TlsDecodeWrapper::from_bytes(hs_data);
  let nst = NewSessionTicket::<ShortBoxSliceU16<_>>::decode(dw)?;
  if nst.opaque().is_empty() {
    return Err(crate::Error::TlsErrorReply(
      TlsError::EmptyNewSessionTicket,
      AlertDescription::DecodeError,
    ));
  }
  *new_session_ticket = Some(nst);
  Ok(())
}

#[inline(always)]
fn transfer_after_handshake_data(
  bytes: &mut MaybeUninitSlice<'_, u8>,
  plaintext: &[u8],
  non_empty_cb: impl FnOnce(NonZeroUsize),
) -> Option<NonZeroUsize> {
  // SAFETY: No data is uninitialized, quite the opposite.
  let all_mut = unsafe { bytes.all_mut() };
  let all_mut_len = all_mut.len();
  let plaintext_len = plaintext.len();
  if let Some(all_mut_partial) = all_mut.get_mut(..plaintext_len) {
    let _ = all_mut_partial.write_copy_of_slice(plaintext);
    // SAFETY: `plaintext` is always is a non-empty slice
    let len = unsafe { NonZeroUsize::new_unchecked(plaintext_len) };
    non_empty_cb(len);
    return Some(len);
  }
  if let Some(plaintext_partial @ [_not_empty, ..]) = plaintext.get(..all_mut_len) {
    let _ = all_mut.write_copy_of_slice(plaintext_partial);
    // SAFETY: The above check just confirmed that all_mut_len is greater than zero
    let len = unsafe { NonZeroUsize::new_unchecked(all_mut_len) };
    non_empty_cb(len);
    return Some(len);
  }
  None
}
