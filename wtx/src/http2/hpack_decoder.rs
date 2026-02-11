use crate::{
  http::{_HeaderNameBuffer, _HeaderValueBuffer, HeaderName, KnownHeaderName, Method, StatusCode},
  http2::{
    Http2Error, Http2ErrorCode,
    hpack_header::{HpackHeaderBasic, HpackHeaderName},
    hpack_headers::HpackHeaders,
    huffman::huffman_decode,
    misc::protocol_err,
  },
  misc::{Usize, from_utf8_basic},
};
use alloc::boxed::Box;
use core::str;

const DYN_IDX_OFFSET: u32 = 62;

#[derive(Debug)]
pub(crate) struct HpackDecoder {
  dyn_headers: HpackHeaders<HpackHeaderBasic>,
  header_buffers: Box<(_HeaderNameBuffer, _HeaderValueBuffer)>,
  max_bytes: (u32, Option<u32>),
}

impl HpackDecoder {
  pub(crate) fn new() -> Self {
    Self {
      dyn_headers: HpackHeaders::new(0),
      header_buffers: Box::new((_HeaderNameBuffer::new(), _HeaderValueBuffer::new())),
      max_bytes: (0, None),
    }
  }

  pub(crate) fn clear(&mut self) {
    let Self { dyn_headers, header_buffers, max_bytes } = self;
    dyn_headers.clear();
    header_buffers.0.clear();
    header_buffers.1.clear();
    *max_bytes = (0, None);
  }

  #[inline]
  pub(crate) fn decode(
    &mut self,
    mut data: &[u8],
    mut cb: impl FnMut((HpackHeaderBasic, HeaderName<&str>, &str)) -> crate::Result<()>,
  ) -> crate::Result<()> {
    if let Some(elem) = self.max_bytes.1.take() {
      self.dyn_headers.set_max_bytes(*Usize::from(elem), |_| {});
      self.max_bytes.0 = elem;
    }
    let mut did_update = false;
    if let [first, ..] = data {
      self.manage_decode(*first, &mut data, &mut cb, || {
        did_update = true;
        Ok(())
      })?;
    }
    if did_update && let [first, ..] = data {
      self.manage_decode(*first, &mut data, &mut cb, || Ok(()))?;
    }
    while let [first, ..] = data {
      self.manage_decode(*first, &mut data, &mut cb, || {
        Err(crate::Error::Http2ErrorGoAway(
          Http2ErrorCode::CompressionError,
          Http2Error::InvalidDynTableSizeUpdate,
        ))
      })?;
    }
    Ok(())
  }

  pub(crate) fn get(&self, idx: u32) -> crate::Result<(HpackHeaderBasic, HeaderName<&str>, &str)> {
    if let Some(elem) = Self::get_static(idx) {
      Ok(elem)
    } else if let Some(elem) = Self::get_dyn(&self.dyn_headers, idx) {
      Ok(elem)
    } else {
      Err(crate::Error::Http2ErrorGoAway(
        Http2ErrorCode::CompressionError,
        Http2Error::InvalidHpackIdx(idx),
      ))
    }
  }

  pub(crate) fn reserve(&mut self, headers: usize, bytes: usize) -> crate::Result<()> {
    self.dyn_headers.reserve(headers, bytes)
  }

  // It is not possible to lower the initial set value
  pub(crate) fn set_max_bytes(&mut self, max_bytes: u32) {
    self.max_bytes.1 = Some(match self.max_bytes.1 {
      Some(elem) => elem.max(max_bytes),
      None => max_bytes,
    });
  }

  fn decode_integer(data: &mut &[u8], mask: u8) -> crate::Result<(u8, u32)> {
    let mut rslt: (u8, u32) = if let [first, rest @ ..] = data {
      *data = rest;
      let n = *first & mask;
      let rslt = (*first, n.into());
      if n < mask {
        return Ok(rslt);
      }
      rslt
    } else {
      return Err(crate::Error::Http2ErrorGoAway(
        Http2ErrorCode::CompressionError,
        Http2Error::InsufficientHpackBytes,
      ));
    };
    let mut shift: u32 = 0;
    for _ in 0..3 {
      let [first, rest @ ..] = data else {
        return Err(crate::Error::Http2ErrorGoAway(
          Http2ErrorCode::CompressionError,
          Http2Error::InsufficientHpackBytes,
        ));
      };
      *data = rest;
      rslt.1 = rslt.1.wrapping_add(u32::from(first & 0b0111_1111) << shift);
      shift = shift.wrapping_add(7);
      let is_last_byte = first & 0b1000_0000 == 0;
      if is_last_byte {
        return Ok(rslt);
      }
    }

    Err(protocol_err(Http2Error::VeryLargeHeaderInteger))
  }

  /// The common index is static-unaware so static names are inserted into `header_buffers`.
  /// Otherwise [`DecodeIdx::Indexed`] would return an empty slice.
  fn decode_literal<const STORE: bool>(
    &mut self,
    data: &mut &[u8],
    mask: u8,
    elem_cb: &mut impl FnMut((HpackHeaderBasic, HeaderName<&str>, &str)) -> crate::Result<()>,
  ) -> crate::Result<()> {
    let idx = Self::decode_integer(data, mask)?.1;
    let has_indexed_name = idx != 0;
    let (hhb, name, value) = if has_indexed_name {
      let value = Self::decode_string_value(&mut self.header_buffers.1, data)?;
      if let Some((hhb, static_name, _)) = Self::get_static(idx) {
        let new_hhb = Self::new_hhb(hhb, value)?;
        elem_cb((new_hhb, static_name, value))?;
        (new_hhb, static_name, value)
      } else if let Some((hhb, dyn_name, _)) = Self::get_dyn(&self.dyn_headers, idx) {
        let new_hhb = Self::new_hhb(hhb, value)?;
        elem_cb((new_hhb, dyn_name, value))?;
        self.header_buffers.0.clear();
        self.header_buffers.0.extend_from_copyable_slice(dyn_name.str().as_bytes())?;
        // SAFETY: just a temporary copy of an already existing string
        let string = unsafe { str::from_utf8_unchecked(&self.header_buffers.0) };
        (new_hhb, HeaderName::new(string), value)
      } else {
        return Err(crate::Error::Http2ErrorGoAway(
          Http2ErrorCode::CompressionError,
          Http2Error::InvalidHpackIdx(idx),
        ));
      }
    } else {
      let (hhn, name) = Self::decode_string_name(&mut self.header_buffers.0, data)?;
      let value = Self::decode_string_value(&mut self.header_buffers.1, data)?;
      let hhb = HpackHeaderBasic::try_from((hhn, value))?;
      elem_cb((hhb, name, value))?;
      (hhb, name, value)
    };
    if STORE {
      self.dyn_headers.push_front(false, hhb, name.str(), [value], |_| {})?;
    }
    Ok(())
  }

  fn decode_string_init<'data>(
    data: &mut &'data [u8],
  ) -> crate::Result<(&'data [u8], &'data [u8], bool)> {
    let (first, len) = Self::decode_integer(data, 0b0111_1111)?;
    let Some((bytes_begin, bytes_end)) = data.split_at_checked(*Usize::from(len)) else {
      return Err(crate::Error::Http2ErrorGoAway(
        Http2ErrorCode::CompressionError,
        Http2Error::InsufficientHpackBytes,
      ));
    };
    let is_encoded = first & 0b1000_0000 == 0b1000_0000;
    Ok((bytes_begin, bytes_end, is_encoded))
  }

  fn decode_string_name<'buffer, 'data, 'rslt>(
    buffer: &'buffer mut _HeaderNameBuffer,
    data: &mut &'data [u8],
  ) -> crate::Result<(HpackHeaderName, HeaderName<&'rslt str>)>
  where
    'buffer: 'rslt,
    'data: 'rslt,
  {
    let (before, after, is_encoded) = Self::decode_string_init(data)?;
    let (hhn, bytes) = if is_encoded {
      huffman_decode(before, buffer)?;
      (HpackHeaderName::new(buffer)?, &**buffer)
    } else {
      let hhn = HpackHeaderName::new(before)?;
      if hhn.is_field() { (hhn, before) } else { (hhn, &[][..]) }
    };
    *data = after;
    Ok((hhn, HeaderName::from_checked_bytes(bytes)?))
  }

  fn decode_string_value<'buffer, 'data, 'rslt>(
    buffer: &'buffer mut _HeaderValueBuffer,
    bytes: &mut &'data [u8],
  ) -> crate::Result<&'rslt str>
  where
    'buffer: 'rslt,
    'data: 'rslt,
  {
    let (before, after, is_encoded) = Self::decode_string_init(bytes)?;
    let rslt = from_utf8_basic(if is_encoded {
      huffman_decode(before, buffer)?;
      buffer
    } else {
      before
    })?;
    *bytes = after;
    Ok(rslt)
  }

  fn get_dyn(
    dyn_headers: &HpackHeaders<HpackHeaderBasic>,
    idx: u32,
  ) -> Option<(HpackHeaderBasic, HeaderName<&str>, &str)> {
    let dyn_idx = *Usize::from(idx.wrapping_sub(DYN_IDX_OFFSET));
    let header = dyn_headers.get_by_idx(dyn_idx)?;
    Some((*header.misc, HeaderName::new(header.name), header.value))
  }

  fn get_static(idx: u32) -> Option<(HpackHeaderBasic, HeaderName<&'static str>, &'static str)> {
    let (hhb, name, value) = match idx {
      1 => (HpackHeaderBasic::Authority, ":authority", ""),
      2 => (HpackHeaderBasic::Method(Method::Get), ":method", "GET"),
      3 => (HpackHeaderBasic::Method(Method::Post), ":method", "POST"),
      4 => (HpackHeaderBasic::Path, ":path", "/"),
      5 => (HpackHeaderBasic::Path, ":path", "/index.html"),
      6 => (HpackHeaderBasic::Scheme, ":scheme", "http"),
      7 => (HpackHeaderBasic::Scheme, ":scheme", "https"),
      8 => (HpackHeaderBasic::StatusCode(StatusCode::Ok), ":status", "200"),
      9 => (HpackHeaderBasic::StatusCode(StatusCode::NoContent), ":status", "204"),
      10 => (HpackHeaderBasic::StatusCode(StatusCode::PartialContent), ":status", "206"),
      11 => (HpackHeaderBasic::StatusCode(StatusCode::NotModified), ":status", "304"),
      12 => (HpackHeaderBasic::StatusCode(StatusCode::BadRequest), ":status", "400"),
      13 => (HpackHeaderBasic::StatusCode(StatusCode::NotFound), ":status", "404"),
      14 => (HpackHeaderBasic::StatusCode(StatusCode::InternalServerError), ":status", "500"),
      15 => (HpackHeaderBasic::Field, KnownHeaderName::AcceptCharset.into(), ""),
      16 => (HpackHeaderBasic::Field, KnownHeaderName::AcceptEncoding.into(), "gzip, deflate"),
      17 => (HpackHeaderBasic::Field, KnownHeaderName::AcceptLanguage.into(), ""),
      18 => (HpackHeaderBasic::Field, KnownHeaderName::AcceptRanges.into(), ""),
      19 => (HpackHeaderBasic::Field, KnownHeaderName::Accept.into(), ""),
      20 => (HpackHeaderBasic::Field, KnownHeaderName::AccessControlAllowOrigin.into(), ""),
      21 => (HpackHeaderBasic::Field, KnownHeaderName::Age.into(), ""),
      22 => (HpackHeaderBasic::Field, KnownHeaderName::Allow.into(), ""),
      23 => (HpackHeaderBasic::Field, KnownHeaderName::Authorization.into(), ""),
      24 => (HpackHeaderBasic::Field, KnownHeaderName::CacheControl.into(), ""),
      25 => (HpackHeaderBasic::Field, KnownHeaderName::ContentDisposition.into(), ""),
      26 => (HpackHeaderBasic::Field, KnownHeaderName::ContentEncoding.into(), ""),
      27 => (HpackHeaderBasic::Field, KnownHeaderName::ContentLanguage.into(), ""),
      28 => (HpackHeaderBasic::Field, KnownHeaderName::ContentLength.into(), ""),
      29 => (HpackHeaderBasic::Field, KnownHeaderName::ContentLocation.into(), ""),
      30 => (HpackHeaderBasic::Field, KnownHeaderName::ContentRange.into(), ""),
      31 => (HpackHeaderBasic::Field, KnownHeaderName::ContentType.into(), ""),
      32 => (HpackHeaderBasic::Field, KnownHeaderName::Cookie.into(), ""),
      33 => (HpackHeaderBasic::Field, KnownHeaderName::Date.into(), ""),
      34 => (HpackHeaderBasic::Field, KnownHeaderName::Etag.into(), ""),
      35 => (HpackHeaderBasic::Field, KnownHeaderName::Expect.into(), ""),
      36 => (HpackHeaderBasic::Field, KnownHeaderName::Expires.into(), ""),
      37 => (HpackHeaderBasic::Field, KnownHeaderName::From.into(), ""),
      38 => (HpackHeaderBasic::Field, KnownHeaderName::Host.into(), ""),
      39 => (HpackHeaderBasic::Field, KnownHeaderName::IfMatch.into(), ""),
      40 => (HpackHeaderBasic::Field, KnownHeaderName::IfModifiedSince.into(), ""),
      41 => (HpackHeaderBasic::Field, KnownHeaderName::IfNoneMatch.into(), ""),
      42 => (HpackHeaderBasic::Field, KnownHeaderName::IfRange.into(), ""),
      43 => (HpackHeaderBasic::Field, KnownHeaderName::IfUnmodifiedSince.into(), ""),
      44 => (HpackHeaderBasic::Field, KnownHeaderName::LastModified.into(), ""),
      45 => (HpackHeaderBasic::Field, KnownHeaderName::Link.into(), ""),
      46 => (HpackHeaderBasic::Field, KnownHeaderName::Location.into(), ""),
      47 => (HpackHeaderBasic::Field, KnownHeaderName::MaxForwards.into(), ""),
      48 => (HpackHeaderBasic::Field, KnownHeaderName::ProxyAuthenticate.into(), ""),
      49 => (HpackHeaderBasic::Field, KnownHeaderName::ProxyAuthorization.into(), ""),
      50 => (HpackHeaderBasic::Field, KnownHeaderName::Range.into(), ""),
      51 => (HpackHeaderBasic::Field, KnownHeaderName::Referer.into(), ""),
      52 => (HpackHeaderBasic::Field, KnownHeaderName::Refresh.into(), ""),
      53 => (HpackHeaderBasic::Field, KnownHeaderName::RetryAfter.into(), ""),
      54 => (HpackHeaderBasic::Field, KnownHeaderName::Server.into(), ""),
      55 => (HpackHeaderBasic::Field, KnownHeaderName::SetCookie.into(), ""),
      56 => (HpackHeaderBasic::Field, KnownHeaderName::StrictTransportSecurity.into(), ""),
      57 => (HpackHeaderBasic::Field, KnownHeaderName::TransferEncoding.into(), ""),
      58 => (HpackHeaderBasic::Field, KnownHeaderName::UserAgent.into(), ""),
      59 => (HpackHeaderBasic::Field, KnownHeaderName::Vary.into(), ""),
      60 => (HpackHeaderBasic::Field, KnownHeaderName::Via.into(), ""),
      61 => (HpackHeaderBasic::Field, KnownHeaderName::WwwAuthenticate.into(), ""),
      _ => return None,
    };
    Some((hhb, HeaderName::new(name), value))
  }

  fn manage_decode(
    &mut self,
    byte: u8,
    data: &mut &[u8],
    elem_cb: &mut impl FnMut((HpackHeaderBasic, HeaderName<&str>, &str)) -> crate::Result<()>,
    mut size_update_cb: impl FnMut() -> crate::Result<()>,
  ) -> crate::Result<()> {
    match DecodeIdx::try_from(byte)? {
      DecodeIdx::Indexed => {
        let idx = Self::decode_integer(data, 0b0111_1111)?.1;
        let (hhb, name, value) = self.get(idx)?;
        elem_cb((hhb, name, value))?;
      }
      DecodeIdx::LiteralNeverIndexed | DecodeIdx::LiteralWithoutIndexing => {
        self.decode_literal::<false>(data, 0b0000_1111, elem_cb)?;
      }
      DecodeIdx::LiteralWithIndexing => {
        self.decode_literal::<true>(data, 0b0011_1111, elem_cb)?;
      }
      DecodeIdx::SizeUpdate => {
        size_update_cb()?;
        let local_max_bytes = Self::decode_integer(data, 0b0001_1111)?.1;
        if local_max_bytes > self.max_bytes.0 {
          return Err(crate::Error::Http2ErrorGoAway(
            Http2ErrorCode::CompressionError,
            Http2Error::OutOfBoundsIndex,
          ));
        }
        self.dyn_headers.set_max_bytes(*Usize::from(local_max_bytes), |_| {});
      }
    }
    Ok(())
  }

  fn new_hhb(hhb: HpackHeaderBasic, value: &str) -> crate::Result<HpackHeaderBasic> {
    Ok(match hhb {
      HpackHeaderBasic::Authority => HpackHeaderBasic::Authority,
      HpackHeaderBasic::Field => HpackHeaderBasic::Field,
      HpackHeaderBasic::Method(_) => HpackHeaderBasic::Method(value.try_into()?),
      HpackHeaderBasic::Path => HpackHeaderBasic::Path,
      HpackHeaderBasic::Protocol(_) => HpackHeaderBasic::Protocol(value.try_into()?),
      HpackHeaderBasic::Scheme => HpackHeaderBasic::Scheme,
      HpackHeaderBasic::StatusCode(_) => HpackHeaderBasic::StatusCode(value.try_into()?),
    })
  }
}

#[derive(Debug)]
enum DecodeIdx {
  Indexed,
  LiteralNeverIndexed,
  LiteralWithIndexing,
  LiteralWithoutIndexing,
  SizeUpdate,
}

impl TryFrom<u8> for DecodeIdx {
  type Error = crate::Error;

  #[inline]
  fn try_from(from: u8) -> Result<Self, Self::Error> {
    Ok(match from & 0b1111_0000 {
      0b0000_0000 => Self::LiteralWithoutIndexing,
      0b0001_0000 => Self::LiteralNeverIndexed,
      n => {
        if n & 0b1000_0000 == 0b1000_0000 {
          Self::Indexed
        } else if n & 0b1100_0000 == 0b0100_0000 {
          Self::LiteralWithIndexing
        } else if n & 0b1110_0000 == 0b0010_0000 {
          Self::SizeUpdate
        } else {
          return Err(protocol_err(Http2Error::UnexpectedHpackIdx));
        }
      }
    })
  }
}
