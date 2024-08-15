use crate::{
  http::{Header, HeaderName, KnownHeaderName, Method, ReqResBuffer},
  http2::{
    misc::{protocol_err, trim_frame_pad},
    CommonFlags, FrameInit, FrameInitTy, HpackDecoder, HpackHeaderBasic, HpackStaticRequestHeaders,
    HpackStaticResponseHeaders, Http2Error, Http2Params, UriBuffer, U31,
  },
  misc::{atoi, from_utf8_basic, ArrayString, Usize},
};

// Some fields of `hsreqh` are only meant to be used locally for writing purposes.
#[derive(Debug)]
pub(crate) struct HeadersFrame<'uri> {
  cf: CommonFlags,
  hsreqh: HpackStaticRequestHeaders<'uri>,
  hsresh: HpackStaticResponseHeaders,
  is_over_size: bool,
  stream_id: U31,
}

impl<'uri> HeadersFrame<'uri> {
  #[inline]
  pub(crate) const fn new(
    (hsreqh, hsresh): (HpackStaticRequestHeaders<'uri>, HpackStaticResponseHeaders),
    stream_id: U31,
  ) -> Self {
    Self { cf: CommonFlags::empty(), hsreqh, hsresh, is_over_size: false, stream_id }
  }

  #[inline]
  pub(crate) const fn bytes(&self) -> [u8; 9] {
    FrameInit::new(self.cf, 0, self.stream_id, FrameInitTy::Headers).bytes()
  }

  #[inline]
  pub(crate) const fn has_eos(&self) -> bool {
    self.cf.has_eos()
  }

  #[inline]
  pub(crate) const fn hsreqh(&self) -> &HpackStaticRequestHeaders<'uri> {
    &self.hsreqh
  }

  #[inline]
  pub(crate) const fn hsresh(&self) -> HpackStaticResponseHeaders {
    self.hsresh
  }

  #[inline]
  pub(crate) const fn is_over_size(&self) -> bool {
    self.is_over_size
  }

  #[inline]
  pub(crate) fn read<const IS_CLIENT: bool, const IS_TRAILER: bool>(
    data: Option<&[u8]>,
    mut fi: FrameInit,
    hp: &Http2Params,
    hpack_dec: &mut HpackDecoder,
    rrb: &mut ReqResBuffer,
    uri_buffer: &mut UriBuffer,
  ) -> crate::Result<(Option<usize>, Self)> {
    if fi.stream_id.is_zero() {
      return Err(protocol_err(Http2Error::InvalidHeadersFrameZeroId));
    }

    fi.cf.only_eoh_eos_pad();
    uri_buffer.clear();

    let (rrb_body, rrb_headers) = rrb.parts_mut();
    let mut data_bytes = data.unwrap_or(rrb_body);
    let _ = trim_frame_pad(fi.cf, &mut data_bytes)?;
    let max_headers_len = *Usize::from(hp.max_headers_len());
    let mut content_length = None;
    let mut expanded_headers_len = 0;
    let mut has_fields = false;
    let mut is_malformed = false;
    let mut is_over_size = false;
    let mut method = None;
    let mut protocol = None;
    let mut status = None;

    hpack_dec.decode(data_bytes, |(elem, name, value)| {
      match elem {
        HpackHeaderBasic::Authority => {
          push_uri(
            &mut uri_buffer.authority,
            &mut expanded_headers_len,
            &mut has_fields,
            &mut is_malformed,
            &mut is_over_size,
            max_headers_len,
            name,
            value,
          );
        }
        HpackHeaderBasic::Field => match KnownHeaderName::try_from(name) {
          Ok(
            KnownHeaderName::Connection
            | KnownHeaderName::KeepAlive
            | KnownHeaderName::ProxyConnection
            | KnownHeaderName::TransferEncoding
            | KnownHeaderName::Upgrade,
          ) => {
            is_malformed = true;
          }
          Ok(KnownHeaderName::Te) if value != b"trailers" => {
            is_malformed = true;
          }
          _ => {
            let header_name = match HeaderName::http2p(name) {
              Ok(elem) => elem,
              Err(_err) => {
                return Err(protocol_err(Http2Error::InvalidHeaderData));
              }
            };
            has_fields = true;
            let len = decoded_header_size(name.len(), value.len());
            expanded_headers_len = expanded_headers_len.wrapping_add(len);
            is_over_size = expanded_headers_len >= max_headers_len;
            if !is_over_size {
              if let Ok(KnownHeaderName::ContentLength) = KnownHeaderName::try_from(header_name) {
                content_length = Some(atoi(value)?);
              }
              rrb_headers.push_front(
                Header { is_sensitive: false, is_trailer: IS_TRAILER, name, value },
                &[],
              )?;
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
            max_headers_len,
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
            max_headers_len,
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
            max_headers_len,
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
            max_headers_len,
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
            max_headers_len,
            name,
            value,
          ) {
            status = Some(local_status);
          }
        }
      }
      Ok(())
    })?;

    if is_malformed {
      return Err(protocol_err(Http2Error::InvalidHeaderData));
    }
    if !IS_TRAILER {
      if IS_CLIENT {
        if method.is_some() || protocol.is_some() {
          return Err(protocol_err(Http2Error::InvalidHeaderData));
        }
      } else {
        if status.is_some() {
          return Err(protocol_err(Http2Error::InvalidHeaderData));
        }
        if let Some(Method::Connect) = method {
          if uri_buffer.authority.is_empty() {
            return Err(protocol_err(Http2Error::InvalidHeaderData));
          }
        } else {
          if uri_buffer.path.is_empty() || uri_buffer.scheme.is_empty() {
            return Err(protocol_err(Http2Error::InvalidHeaderData));
          }
        }
        rrb.set_uri_from_parts(
          uri_buffer.scheme.as_str(),
          uri_buffer.authority.as_str(),
          uri_buffer.path.as_str(),
        )?;
      }
    }

    Ok((
      content_length,
      Self {
        cf: fi.cf,
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
    ))
  }

  #[inline]
  pub(crate) fn set_eoh(&mut self) {
    self.cf.set_eoh();
  }

  #[inline]
  pub(crate) fn set_eos(&mut self) {
    self.cf.set_eos();
  }
}

#[inline]
const fn decoded_header_size(name: usize, value: usize) -> usize {
  name.wrapping_add(value).wrapping_add(32)
}

#[inline]
fn push_enum(
  expanded_headers_len: &mut usize,
  has_fields: &mut bool,
  is_malformed: &mut bool,
  is_over_size: &mut bool,
  is_some: bool,
  max_headers_len: usize,
  name: &[u8],
  value: &[u8],
) -> bool {
  if *has_fields || is_some {
    *is_malformed = true;
    false
  } else {
    let len = decoded_header_size(name.len().wrapping_add(1), value.len());
    *expanded_headers_len = expanded_headers_len.wrapping_add(len);
    *is_over_size = *expanded_headers_len >= max_headers_len;
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
  max_headers_len: usize,
  name: &[u8],
  value: &[u8],
) {
  if *has_fields || !buffer.is_empty() {
    *is_malformed = true;
  } else {
    let len = decoded_header_size(name.len().wrapping_add(1), value.len());
    *expanded_headers_len = expanded_headers_len.wrapping_add(len);
    *is_over_size = *expanded_headers_len >= max_headers_len;
    if !*is_over_size {
      let _ = from_utf8_basic(value).ok().and_then(|el| buffer.push_str(el).ok());
    }
  }
}
