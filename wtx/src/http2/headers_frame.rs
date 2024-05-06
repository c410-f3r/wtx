use crate::{
  http::{HeaderName, Headers},
  http2::{
    misc::trim_frame_pad, uri_buffer::MAX_URI_LEN, FrameHeaderTy, FrameInit, HpackDecoder,
    HpackHeaderBasic, HpackStaticRequestHeaders, HpackStaticResponseHeaders, Http2Params,
    UriBuffer, EOH_MASK, EOS_MASK, U31,
  },
  misc::{from_utf8_basic, ArrayString, Usize},
};

#[derive(Debug)]
pub(crate) struct HeadersFrame<'data> {
  flag: u8,
  hsreqh: HpackStaticRequestHeaders<'data>,
  hsresh: HpackStaticResponseHeaders,
  is_over_size: bool,
  stream_id: U31,
}

impl<'data> HeadersFrame<'data> {
  pub(crate) fn new(
    (hsreqh, hsresh): (HpackStaticRequestHeaders<'data>, HpackStaticResponseHeaders),
    stream_id: U31,
  ) -> Self {
    Self { flag: 0, hsreqh, hsresh, is_over_size: false, stream_id }
  }

  pub(crate) fn bytes(&self) -> [u8; 9] {
    FrameInit::new(0, self.flag, self.stream_id, FrameHeaderTy::Headers).bytes()
  }

  pub(crate) fn hsreqh(&self) -> &HpackStaticRequestHeaders<'data> {
    &self.hsreqh
  }

  pub(crate) fn hsresh(&self) -> HpackStaticResponseHeaders {
    self.hsresh
  }

  pub(crate) fn is_eoh(&self) -> bool {
    self.flag & EOH_MASK == EOH_MASK
  }

  pub(crate) fn is_eos(&self) -> bool {
    self.flag & EOS_MASK == EOS_MASK
  }

  pub(crate) fn is_over_size(&self) -> bool {
    self.is_over_size
  }

  pub(crate) fn read<const DO_NOT_PUSH_URI: bool>(
    mut data: &[u8],
    fi: FrameInit,
    headers: &mut Headers,
    hp: &Http2Params,
    hpack_dec: &mut HpackDecoder,
    uri: &mut ArrayString<MAX_URI_LEN>,
    uri_buffer: &mut UriBuffer,
  ) -> crate::Result<(Self, usize)> {
    if fi.stream_id.is_zero() {
      return Err(crate::http2::ErrorCode::FrameSizeError.into());
    }

    uri_buffer.clear();

    let _ = trim_frame_pad(&mut data, fi.flags)?;
    let max_expanded_headers_len = *Usize::from(hp.max_expanded_headers_len());
    let mut expanded_headers_len = 0;
    let mut has_fields = false;
    let mut is_malformed = false;
    let mut is_over_size = false;
    let mut method = None;
    let mut protocol = None;
    let mut status = None;

    hpack_dec.decode(data, |(elem, name, value)| {
      match elem {
        HpackHeaderBasic::Authority => {
          push_uri(
            &mut uri_buffer.authority,
            &mut expanded_headers_len,
            &mut has_fields,
            &mut is_malformed,
            &mut is_over_size,
            max_expanded_headers_len,
            name,
            value,
          );
        }
        HpackHeaderBasic::Field => match HeaderName::new(name) {
          HeaderName::CONNECTION
          | HeaderName::KEEP_ALIVE
          | HeaderName::PROXY_CONNECTION
          | HeaderName::TRANSFER_ENCODING
          | HeaderName::UPGRADE => {
            is_malformed = true;
          }
          HeaderName::TE if value != b"trailers" => {
            is_malformed = true;
          }
          _ => {
            has_fields = true;
            let len = decoded_header_size(name.len(), value.len());
            expanded_headers_len = expanded_headers_len.wrapping_add(len);
            is_over_size = expanded_headers_len >= max_expanded_headers_len;
            if !is_over_size {
              headers.reserve(name.len().wrapping_add(value.len()), 1);
              headers.push_front(name, value, false)?;
            }
          }
        },
        HpackHeaderBasic::Method(local_method) => {
          if push_enum(
            &mut expanded_headers_len,
            &mut has_fields,
            &mut is_malformed,
            &mut is_over_size,
            method.is_some(),
            max_expanded_headers_len,
            name,
            value,
          ) {
            method = Some(local_method);
          }
        }
        HpackHeaderBasic::Path => {
          push_uri(
            &mut uri_buffer.path,
            &mut expanded_headers_len,
            &mut has_fields,
            &mut is_malformed,
            &mut is_over_size,
            max_expanded_headers_len,
            name,
            value,
          );
        }
        HpackHeaderBasic::Protocol(local_protocol) => {
          if push_enum(
            &mut expanded_headers_len,
            &mut has_fields,
            &mut is_malformed,
            &mut is_over_size,
            protocol.is_some(),
            max_expanded_headers_len,
            name,
            value,
          ) {
            protocol = Some(local_protocol);
          }
        }
        HpackHeaderBasic::Scheme => {
          push_uri(
            &mut uri_buffer.scheme,
            &mut expanded_headers_len,
            &mut has_fields,
            &mut is_malformed,
            &mut is_over_size,
            max_expanded_headers_len,
            name,
            value,
          );
        }
        HpackHeaderBasic::StatusCode(local_status) => {
          if push_enum(
            &mut expanded_headers_len,
            &mut has_fields,
            &mut is_malformed,
            &mut is_over_size,
            status.is_some(),
            max_expanded_headers_len,
            name,
            value,
          ) {
            status = Some(local_status);
          }
        }
      }
      Ok(())
    })?;

    if DO_NOT_PUSH_URI {
      uri.clear();
      uri.push_str(uri_buffer.scheme.as_str())?;
      uri.push_str(uri_buffer.authority.as_str())?;
      uri.push_str(uri_buffer.path.as_str())?;
    }

    Ok((
      Self {
        flag: fi.flags,
        hsreqh: HpackStaticRequestHeaders {
          authority: &[],
          method,
          path: &[],
          protocol,
          scheme: &[],
        },
        hsresh: HpackStaticResponseHeaders { status_code: status },
        is_over_size,
        stream_id: fi.stream_id,
      },
      expanded_headers_len,
    ))
  }

  pub(crate) fn set_eoh(&mut self) {
    self.flag |= EOH_MASK;
  }

  pub(crate) fn set_eos(&mut self) {
    self.flag |= EOS_MASK;
  }
}

#[inline]
fn decoded_header_size(name: usize, value: usize) -> usize {
  name.wrapping_add(value).wrapping_add(32)
}

#[inline]
fn push_enum(
  expanded_headers_len: &mut usize,
  has_fields: &mut bool,
  is_malformed: &mut bool,
  is_over_size: &mut bool,
  is_some: bool,
  max_expanded_headers_len: usize,
  name: &[u8],
  value: &[u8],
) -> bool {
  if *has_fields || is_some {
    *is_malformed = true;
    false
  } else {
    let len = decoded_header_size(name.len().wrapping_add(1), value.len());
    *expanded_headers_len = expanded_headers_len.wrapping_add(len);
    *is_over_size = *expanded_headers_len >= max_expanded_headers_len;
    !*is_over_size
  }
}

#[inline]
fn push_uri<const N: usize>(
  buffer: &mut ArrayString<N>,
  expanded_headers_len: &mut usize,
  has_fields: &mut bool,
  is_malformed: &mut bool,
  is_over_size: &mut bool,
  max_expanded_headers_len: usize,
  name: &[u8],
  value: &[u8],
) {
  if *has_fields || !buffer.is_empty() {
    *is_malformed = true;
  } else {
    let len = decoded_header_size(name.len().wrapping_add(1), value.len());
    *expanded_headers_len = expanded_headers_len.wrapping_add(len);
    *is_over_size = *expanded_headers_len >= max_expanded_headers_len;
    if !*is_over_size {
      let _ = from_utf8_basic(value).ok().and_then(|el| buffer.push_str(el).ok());
    }
  }
}
