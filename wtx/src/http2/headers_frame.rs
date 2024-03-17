use crate::{
  http::{HeaderName, Headers},
  http2::{
    misc::trim_frame_pad, uri_buffer::MAX_URI_LEN, FrameHeaderTy, FrameInit, HpackDecoder,
    HpackEncoder, HpackHeaderBasic, HpackStaticRequestHeaders, HpackStaticResponseHeaders,
    StreamId, UriBuffer,
  },
  misc::{from_utf8_basic, ArrayString, ByteVector},
};

const ALL: u8 = END_STREAM | END_HEADERS | PADDED | PRIORITY;
const END_HEADERS: u8 = 0b0000_0100;
const END_STREAM: u8 = 0b0000_0001;
const PADDED: u8 = 0b0000_1000;
const PRIORITY: u8 = 0b0010_0000;

#[derive(Debug)]
pub struct HeadersFrame<'data, 'headers> {
  flag: u8,
  headers: &'headers Headers,
  hsreqh: HpackStaticRequestHeaders<'data>,
  hsresh: HpackStaticResponseHeaders,
  is_over_size: bool,
  stream_id: StreamId,
}

impl<'data, 'headers> HeadersFrame<'data, 'headers> {
  pub(crate) fn new(
    headers: &'headers Headers,
    hsreqh: HpackStaticRequestHeaders<'data>,
    hsresh: HpackStaticResponseHeaders,
    stream_id: StreamId,
  ) -> Self {
    Self { flag: END_HEADERS, headers, hsreqh, hsresh, is_over_size: false, stream_id }
  }

  pub(crate) fn hsreqh(&self) -> &HpackStaticRequestHeaders<'data> {
    &self.hsreqh
  }

  pub(crate) fn hsresh(&self) -> HpackStaticResponseHeaders {
    self.hsresh
  }

  pub(crate) fn is_end_headers(flag: u8) -> bool {
    flag & END_HEADERS == END_HEADERS
  }

  pub(crate) fn is_end_stream(flag: u8) -> bool {
    flag & END_STREAM == END_STREAM
  }

  pub(crate) fn is_over_size(&self) -> bool {
    self.is_over_size
  }

  pub(crate) fn is_padded(flag: u8) -> bool {
    flag & PADDED == PADDED
  }

  pub(crate) fn read<const IS_CLIENT: bool, const IS_REQUEST: bool>(
    mut data: &[u8],
    fi: FrameInit,
    headers: &'headers mut Headers,
    hpack_dec: &mut HpackDecoder,
    max_header_list_size: usize,
    uri: &mut ArrayString<MAX_URI_LEN>,
    uri_buffer: &mut UriBuffer,
  ) -> crate::Result<Self> {
    if fi.stream_id.is_zero() {
      return Err(crate::http2::ErrorCode::FrameSizeError.into());
    }

    uri_buffer.clear();

    let _ = trim_frame_pad(&mut data, fi.flag)?;
    let mut has_fields = false;
    let mut headers_size = 0;
    let mut is_malformed = false;
    let mut is_over_size = false;
    let mut method = None;
    let mut protocol = None;
    let mut status = None;

    hpack_dec.decode(data, |(elem, name, value)| match elem {
      HpackHeaderBasic::Authority => {
        push_uri(
          &mut uri_buffer.authority,
          &mut has_fields,
          &mut headers_size,
          &mut is_malformed,
          &mut is_over_size,
          max_header_list_size,
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
          headers_size = headers_size.wrapping_add(len);
          is_over_size = headers_size >= max_header_list_size;
          if !is_over_size {
            headers.push_front(name, value, false);
          }
        }
      },
      HpackHeaderBasic::Method(local_method) => {
        if push_enum(
          &mut has_fields,
          &mut headers_size,
          &mut is_malformed,
          &mut is_over_size,
          method.is_some(),
          max_header_list_size,
          name,
          value,
        ) {
          method = Some(local_method);
        }
      }
      HpackHeaderBasic::Path => {
        push_uri(
          &mut uri_buffer.path,
          &mut has_fields,
          &mut headers_size,
          &mut is_malformed,
          &mut is_over_size,
          max_header_list_size,
          name,
          value,
        );
      }
      HpackHeaderBasic::Protocol(local_protocol) => {
        if push_enum(
          &mut has_fields,
          &mut headers_size,
          &mut is_malformed,
          &mut is_over_size,
          protocol.is_some(),
          max_header_list_size,
          name,
          value,
        ) {
          protocol = Some(local_protocol);
        }
      }
      HpackHeaderBasic::Scheme => {
        push_uri(
          &mut uri_buffer.scheme,
          &mut has_fields,
          &mut headers_size,
          &mut is_malformed,
          &mut is_over_size,
          max_header_list_size,
          name,
          value,
        );
      }
      HpackHeaderBasic::Status(local_status) => {
        if push_enum(
          &mut has_fields,
          &mut headers_size,
          &mut is_malformed,
          &mut is_over_size,
          status.is_some(),
          max_header_list_size,
          name,
          value,
        ) {
          status = Some(local_status);
        }
      }
    })?;

    if !IS_REQUEST {
      uri.clear();
      let _ = uri.try_push_str(uri_buffer.scheme.as_str());
      let _ = uri.try_push_str(uri_buffer.authority.as_str());
      let _ = uri.try_push_str(uri_buffer.path.as_str());
    }

    Ok(Self {
      flag: fi.flag,
      headers,
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
    })
  }

  pub(crate) fn write<const IS_CLIENT: bool>(
    &self,
    hpack_enc: &mut HpackEncoder,
    wb: &mut ByteVector,
  ) -> crate::Result<()> {
    wb.extend_from_slice(
      FrameInit::new(self.flag.into(), 0, self.stream_id, FrameHeaderTy::Headers)
        .bytes()
        .as_slice(),
    );
    if IS_CLIENT {
      hpack_enc.encode(
        wb,
        self.hsreqh.iter(),
        self.headers.iter().map(|(name, value, is_sensitive)| (name, value, is_sensitive)),
      )?;
    } else {
      hpack_enc.encode(
        wb,
        self.hsresh.iter(),
        self.headers.iter().map(|(name, value, is_sensitive)| (name, value, is_sensitive)),
      )?;
    }
    Ok(())
  }
}

#[inline]
fn decoded_header_size(name: usize, value: usize) -> usize {
  name.wrapping_add(value).wrapping_add(32)
}

#[inline]
fn push_enum(
  has_fields: &mut bool,
  headers_size: &mut usize,
  is_malformed: &mut bool,
  is_over_size: &mut bool,
  is_some: bool,
  max_header_list_size: usize,
  name: &[u8],
  value: &[u8],
) -> bool {
  if *has_fields || is_some {
    *is_malformed = true;
    false
  } else {
    let len = decoded_header_size(name.len().wrapping_add(1), value.len());
    *headers_size = headers_size.wrapping_add(len);
    *is_over_size = *headers_size >= max_header_list_size;
    !*is_over_size
  }
}

#[inline]
fn push_uri<const N: usize>(
  buffer: &mut ArrayString<N>,
  has_fields: &mut bool,
  headers_size: &mut usize,
  is_malformed: &mut bool,
  is_over_size: &mut bool,
  max_header_list_size: usize,
  name: &[u8],
  value: &[u8],
) {
  if *has_fields || !buffer.is_empty() {
    *is_malformed = true;
  } else {
    let len = decoded_header_size(name.len().wrapping_add(1), value.len());
    *headers_size = headers_size.wrapping_add(len);
    *is_over_size = *headers_size >= max_header_list_size;
    if !*is_over_size {
      let _ = from_utf8_basic(value).ok().and_then(|el| buffer.try_push_str(el).ok());
    }
  }
}
