use crate::{
  collection::ArrayStringU8,
  http::{Header, KnownHeaderName, Method, ReqResBuffer},
  http2::{
    Http2Error, Http2Params,
    common_flags::CommonFlags,
    frame_init::{FrameInit, FrameInitTy},
    hpack_decoder::HpackDecoder,
    hpack_header::HpackHeaderBasic,
    hpack_static_headers::{HpackStaticRequestHeaders, HpackStaticResponseHeaders},
    misc::{protocol_err, trim_frame_pad},
    u31::U31,
  },
  misc::{LeaseMut, Usize},
};
use alloc::string::String;

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
  pub(crate) const fn new(
    (hsreqh, hsresh): (HpackStaticRequestHeaders<'uri>, HpackStaticResponseHeaders),
    stream_id: U31,
  ) -> Self {
    Self { cf: CommonFlags::empty(), hsreqh, hsresh, is_over_size: false, stream_id }
  }

  pub(crate) const fn bytes(&self) -> [u8; 9] {
    FrameInit::new(self.cf, 0, self.stream_id, FrameInitTy::Headers).bytes()
  }

  pub(crate) const fn has_eos(&self) -> bool {
    self.cf.has_eos()
  }

  pub(crate) const fn hsreqh(&self) -> &HpackStaticRequestHeaders<'uri> {
    &self.hsreqh
  }

  pub(crate) const fn hsresh(&self) -> HpackStaticResponseHeaders {
    self.hsresh
  }

  pub(crate) const fn is_over_size(&self) -> bool {
    self.is_over_size
  }

  // Authority -> Path -> Scheme: Create path buffer¹ to push everything at the end.
  // Authority -> Scheme -> Path: Push everything at path level.
  // Path -> Authority -> Scheme: Create path buffer¹ to push everything at the end.
  // Path -> Scheme -> Authority: Create path buffer¹ to push everything at the end.
  // Scheme -> Authority -> Path: Push everything at path level.
  // Scheme -> Path -> Authority: Create path buffer¹ to push everything at the end.
  //
  // ¹If path is static, then the spacing buffer isn't necessary.
  #[expect(clippy::too_many_lines, reason = "variables are highly coupled")]
  pub(crate) fn read<const IS_CLIENT: bool, const IS_TRAILER: bool>(
    data: Option<&[u8]>,
    mut fi: FrameInit,
    hp: &Http2Params,
    hpack_dec: &mut HpackDecoder,
    (rrb, rrb_body_start): (&mut ReqResBuffer, usize),
  ) -> crate::Result<(Option<usize>, Self)> {
    if fi.stream_id.is_zero() {
      return Err(protocol_err(Http2Error::InvalidHeadersFrameZeroId));
    }

    fi.cf.only_eoh_eos_pad_pri();

    let lease = rrb.lease_mut();
    let (rrb_body, rrb_headers, rrb_uri) = (&lease.body, &mut lease.headers, &mut lease.uri);
    let mut data_bytes = data.unwrap_or_else(|| rrb_body.get(rrb_body_start..).unwrap_or_default());
    let _ = trim_frame_pad(fi.cf, &mut data_bytes)?;
    trim_priority(fi.cf, &mut data_bytes);
    let max_headers_len = *Usize::from(hp.max_headers_len());
    let mut content_length = None;
    let mut expanded_headers_len = 0;
    let mut has_fields = false;
    let mut is_malformed = false;
    let mut is_over_size = false;
    let mut method = None;
    let mut protocol = None;
    let mut status = None;

    let mut already_created_path = false;
    let mut authority = ArrayStringU8::<60>::new();
    let mut path_len = 0;
    let mut scheme = ArrayStringU8::<12>::new();
    let mut static_path = None;
    let mut uri_buffer = rrb_uri.reset();

    hpack_dec.decode(data_bytes, |(hhb, name, value)| {
      match hhb {
        HpackHeaderBasic::Authority => {
          push_uri(
            !authority.is_empty(),
            &mut expanded_headers_len,
            &mut has_fields,
            &mut is_malformed,
            &mut is_over_size,
            max_headers_len,
            name.str(),
            value,
            |local_value| {
              let _ = authority.push_str(local_value).ok();
            },
          );
        }
        HpackHeaderBasic::Field => match KnownHeaderName::try_from(name.str().as_bytes()) {
          Ok(
            KnownHeaderName::Connection
            | KnownHeaderName::KeepAlive
            | KnownHeaderName::ProxyConnection
            | KnownHeaderName::TransferEncoding
            | KnownHeaderName::Upgrade,
          ) => {
            is_malformed = true;
          }
          Ok(KnownHeaderName::Te) if value != "trailers" => {
            is_malformed = true;
          }
          _ => {
            has_fields = true;
            let len = decoded_header_size(name.str().len(), value.len());
            expanded_headers_len = expanded_headers_len.wrapping_add(len);
            is_over_size = expanded_headers_len >= max_headers_len;
            if !is_over_size {
              if let Ok(KnownHeaderName::ContentLength) =
                KnownHeaderName::try_from(name.str().as_bytes())
              {
                content_length = Some(value.parse()?);
              }
              rrb_headers.push_from_iter(Header::new(false, IS_TRAILER, name.str(), [value]))?;
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
            name.str(),
            value,
          ) {
            method = Some(local_method);
          }
        }
        HpackHeaderBasic::Path => {
          push_uri(
            path_len > 0,
            &mut expanded_headers_len,
            &mut has_fields,
            &mut is_malformed,
            &mut is_over_size,
            max_headers_len,
            name.str(),
            value,
            |local_value| {
              path_len = local_value.len();
              if !scheme.is_empty() && !authority.is_empty() {
                push_uri_buffer(&scheme, &authority, local_value, &mut uri_buffer);
                already_created_path = true;
              } else {
                match local_value {
                  "/" => static_path = Some("/"),
                  "/index.html" => static_path = Some("/index.html"),
                  _ => create_path_buffer(&mut uri_buffer, local_value),
                }
              }
            },
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
            name.str(),
            value,
          ) {
            protocol = Some(local_protocol);
          }
        }
        HpackHeaderBasic::Scheme => {
          push_uri(
            !scheme.is_empty(),
            &mut expanded_headers_len,
            &mut has_fields,
            &mut is_malformed,
            &mut is_over_size,
            max_headers_len,
            name.str(),
            value,
            |local_value| {
              let _ = scheme.push_str(local_value).ok();
            },
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
            name.str(),
            value,
          ) {
            status = Some(local_status);
          }
        }
      }
      Ok(())
    })?;

    if is_malformed {
      return Err(protocol_err(Http2Error::InvalidHeaderFrame));
    }
    if !IS_TRAILER {
      if IS_CLIENT {
        if method.is_some() || protocol.is_some() {
          return Err(protocol_err(Http2Error::InvalidClientHeader));
        }
      } else {
        if status.is_some() {
          return Err(protocol_err(Http2Error::InvalidServerHeader));
        }
        if let Some(Method::Connect) = method {
          if authority.is_empty() {
            return Err(protocol_err(Http2Error::InvalidServerHeader));
          }
        } else if scheme.is_empty() || path_len == 0 {
          return Err(protocol_err(Http2Error::InvalidServerHeader));
        } else {
        }
        if !already_created_path {
          if let Some(path) = static_path {
            push_uri_buffer(&scheme, &authority, path, &mut uri_buffer);
          } else {
            push_uri_in_path_buffer(&scheme, &authority, &mut uri_buffer)?;
          }
        }
      }
    }

    Ok((
      content_length,
      Self {
        cf: fi.cf,
        hsreqh: HpackStaticRequestHeaders { authority: "", method, path: "", protocol, scheme: "" },
        hsresh: HpackStaticResponseHeaders { status_code: status },
        is_over_size,
        stream_id: fi.stream_id,
      },
    ))
  }

  pub(crate) const fn set_eoh(&mut self) {
    self.cf.set_eoh();
  }

  pub(crate) const fn set_eos(&mut self) {
    self.cf.set_eos();
  }
}

fn create_path_buffer(uri_buffer: &mut String, path: &str) {
  uri_buffer.reserve(64usize.wrapping_add(path.len()));
  // SAFETY: zero is ASCII
  uri_buffer.push_str(unsafe { str::from_utf8_unchecked(&[0; 64]) });
  uri_buffer.push_str(path);
}

const fn decoded_header_size(name: usize, value: usize) -> usize {
  name.wrapping_add(value).wrapping_add(32)
}

const fn push_enum(
  expanded_headers_len: &mut usize,
  has_fields: &mut bool,
  is_malformed: &mut bool,
  is_over_size: &mut bool,
  is_some: bool,
  max_headers_len: usize,
  name: &str,
  value: &str,
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

fn push_uri_buffer(scheme: &str, authority: &str, path: &str, uri_buffer: &mut String) {
  uri_buffer.reserve(
    Usize::from(scheme.len())
      .wrapping_add(3)
      .wrapping_add(*Usize::from(authority.len()))
      .wrapping_add(*Usize::from(path.len())),
  );
  uri_buffer.push_str(scheme);
  uri_buffer.push_str("://");
  uri_buffer.push_str(authority);
  uri_buffer.push_str(path);
}

fn push_uri(
  already_has_part: bool,
  expanded_headers_len: &mut usize,
  has_fields: &mut bool,
  is_malformed: &mut bool,
  is_over_size: &mut bool,
  max_headers_len: usize,
  name: &str,
  value: &str,
  cb: impl FnOnce(&str),
) {
  if *has_fields || already_has_part {
    *is_malformed = true;
  } else {
    let len = decoded_header_size(name.len().wrapping_add(1), value.len());
    *expanded_headers_len = expanded_headers_len.wrapping_add(len);
    *is_over_size = *expanded_headers_len >= max_headers_len;
    if !*is_over_size {
      cb(value);
    }
  }
}

fn push_uri_in_path_buffer(
  scheme: &str,
  authority: &str,
  uri_buffer: &mut str,
) -> crate::Result<()> {
  let sum = scheme.len().wrapping_add(3).wrapping_add(authority.len());
  if sum > 64 {
    return Err(protocol_err(Http2Error::InvalidServerHeaderUriOverflow));
  }
  // SAFETY: `scheme` and `authority` are UTF-8
  let bytes = unsafe { uri_buffer.as_bytes_mut() };
  if let Some((lhs, _)) = bytes.split_at_mut_checked(64) {
    let mut start = *Usize::from(sum);
    {
      let from = lhs.len().wrapping_sub(start);
      let to = from.wrapping_add(*Usize::from(scheme.len()));
      if let Some(elem) = lhs.get_mut(from..to) {
        elem.copy_from_slice(scheme.as_bytes());
      }
      start = to;
    }
    {
      let from = start;
      let to = start.wrapping_add(3);
      if let Some([a, b, c]) = lhs.get_mut(from..to) {
        *a = b':';
        *b = b'/';
        *c = b'/';
      }
      start = to;
    }
    {
      if let Some(elem) = lhs.get_mut(start..) {
        elem.copy_from_slice(authority.as_bytes());
      }
    }
  }
  Ok(())
}

const fn trim_priority(cf: CommonFlags, data: &mut &[u8]) {
  if cf.has_pri() {
    let [_, _, _, _, _, rest @ ..] = data else {
      return;
    };
    *data = rest;
  }
}

#[cfg(test)]
mod tests {
  use crate::{
    http2::headers_frame::{create_path_buffer, push_uri_in_path_buffer},
    misc::UriString,
  };
  use alloc::string::String;

  #[test]
  fn uri_is_correctly_created() {
    let mut uri = UriString::new(String::new());
    {
      let mut uri_buffer = uri.reset();
      create_path_buffer(&mut uri_buffer, "/world");
      push_uri_in_path_buffer("http", "hello.com", &mut uri_buffer).unwrap();
    }
    assert_eq!(uri.as_str(), "http://hello.com/world");
  }
}
