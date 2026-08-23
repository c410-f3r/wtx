use crate::{
  collections::ArrayStringU8,
  http::{Header, HttpRecvParams, KnownHeaderName, Method, MsgBufferString, U31},
  http2::{
    Http2Error,
    common_flags::CommonFlags,
    frame_init::{FrameInit, FrameInitTy},
    hpack_decoder::HpackDecoder,
    hpack_header::HpackHeaderBasic,
    hpack_static_headers::{HpackStaticRequestHeaders, HpackStaticResponseHeaders},
    misc::{protocol_err, trim_frame_pad},
  },
  misc::{LeaseMut as _, Usize},
};

// Some fields of `hsreqh` are only meant to be used locally for writing purposes.
#[derive(Debug)]
pub(crate) struct HeadersFrame<'uri> {
  cf: CommonFlags,
  hsreqh: HpackStaticRequestHeaders<'uri>,
  hsresph: HpackStaticResponseHeaders,
  is_over_size: bool,
  stream_id: U31,
}

impl<'uri> HeadersFrame<'uri> {
  pub(crate) const fn new(
    (hsreqh, hsresph): (HpackStaticRequestHeaders<'uri>, HpackStaticResponseHeaders),
    stream_id: U31,
  ) -> Self {
    Self { cf: CommonFlags::empty(), hsreqh, hsresph, is_over_size: false, stream_id }
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
    self.hsresph
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
    hp: &HttpRecvParams,
    hpack_dec: &mut HpackDecoder,
    (msg_buffer, rrb_body_start): (&mut MsgBufferString, usize),
  ) -> crate::Result<(Option<usize>, Self)> {
    if fi.stream_id.is_zero() {
      return Err(protocol_err(Http2Error::InvalidHeadersFrameZeroId));
    }

    fi.cf.only_eoh_eos_pad_pri();

    let lease = msg_buffer.lease_mut();
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

    let mut authority = ArrayStringU8::<64>::new();
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
              match local_value {
                "/" => static_path = Some("/"),
                "/index.html" => static_path = Some("/index.html"),
                _ => uri_buffer.push_str(local_value),
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
        }
        let mut prefix = ArrayStringU8::<79>::new();
        let _rslt0 = prefix.push_str(&scheme);
        let _rslt1 = prefix.push_str("://");
        let _rslt2 = prefix.push_str(&authority);
        if let Some(path) = static_path {
          uri_buffer.push_str(&prefix);
          uri_buffer.push_str(path);
        } else {
          uri_buffer.insert_str(0, prefix.as_str());
        }
      }
    }

    Ok((
      content_length,
      Self {
        cf: fi.cf,
        hsreqh: HpackStaticRequestHeaders { authority: "", method, path: "", protocol, scheme: "" },
        hsresph: HpackStaticResponseHeaders { status_code: status },
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

const fn trim_priority(cf: CommonFlags, data: &mut &[u8]) {
  if cf.has_pri() {
    let [_, _, _, _, _, rest @ ..] = data else {
      return;
    };
    *data = rest;
  }
}
