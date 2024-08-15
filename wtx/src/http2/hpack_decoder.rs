use crate::{
  http::{
    AbstractHeaders, KnownHeaderName, Method, StatusCode, _HeaderNameBuffer, _HeaderValueBuffer,
  },
  http2::{
    hpack_header::{HpackHeaderBasic, HpackHeaderName},
    huffman_decode,
    misc::protocol_err,
    Http2Error, Http2ErrorCode,
  },
  misc::{ArrayVector, Usize},
};
use alloc::boxed::Box;

const DYN_IDX_OFFSET: usize = 62;

#[derive(Debug)]
pub(crate) struct HpackDecoder {
  dyn_headers: AbstractHeaders<HpackHeaderBasic>,
  header_buffers: Box<(_HeaderNameBuffer, _HeaderValueBuffer)>,
  max_bytes: (u32, Option<u32>),
}

impl HpackDecoder {
  #[inline]
  pub(crate) fn new() -> Self {
    Self {
      dyn_headers: AbstractHeaders::new(0),
      header_buffers: Box::new((ArrayVector::new(), ArrayVector::new())),
      max_bytes: (0, None),
    }
  }

  #[inline]
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
    mut cb: impl FnMut((HpackHeaderBasic, &[u8], &[u8])) -> crate::Result<()>,
  ) -> crate::Result<()> {
    if let Some(elem) = self.max_bytes.1.take() {
      self.dyn_headers.set_max_bytes(*Usize::from(elem), |_, _| {});
      self.max_bytes.0 = elem;
    }
    let mut did_update = false;
    if let [first, ..] = data {
      self.manage_decode(*first, &mut data, &mut cb, || {
        did_update = true;
        Ok(())
      })?;
    }
    if did_update {
      if let [first, ..] = data {
        self.manage_decode(*first, &mut data, &mut cb, || Ok(()))?;
      }
    }
    while let [first, ..] = data {
      self.manage_decode(*first, &mut data, &mut cb, || {
        Err(crate::Error::Http2ErrorGoAway(
          Http2ErrorCode::CompressionError,
          Some(Http2Error::InvalidDynTableSizeUpdate),
        ))
      })?;
    }
    Ok(())
  }

  // It is not possible to lower the initial set value
  #[inline]
  pub(crate) fn set_max_bytes(&mut self, max_bytes: u32) {
    self.max_bytes.1 = Some(match self.max_bytes.1 {
      Some(elem) => elem.max(max_bytes),
      None => max_bytes,
    });
  }

  #[inline]
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
        Some(Http2Error::InsufficientHpackBytes),
      ));
    };
    let mut shift: u32 = 0;
    for _ in 0..3 {
      let [first, rest @ ..] = data else {
        return Err(crate::Error::Http2ErrorGoAway(
          Http2ErrorCode::CompressionError,
          Some(Http2Error::InsufficientHpackBytes),
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
  #[inline]
  fn decode_literal<const STORE: bool>(
    &mut self,
    data: &mut &[u8],
    mask: u8,
    elem_cb: &mut impl FnMut((HpackHeaderBasic, &[u8], &[u8])) -> crate::Result<()>,
  ) -> crate::Result<()> {
    let idx = Self::decode_integer(data, mask)?.1;
    let has_indexed_name = idx != 0;
    let (hhb, name, value) = if has_indexed_name {
      let value = Self::decode_string_value(&mut self.header_buffers.1, data)?;
      let (hhb, (static_name, dyn_name), _) = Self::get(&self.dyn_headers, *Usize::from(idx))?;
      let new_hhb = match hhb {
        HpackHeaderBasic::Authority => HpackHeaderBasic::Authority,
        HpackHeaderBasic::Field => HpackHeaderBasic::Field,
        HpackHeaderBasic::Method(_) => HpackHeaderBasic::Method(value.try_into()?),
        HpackHeaderBasic::Path => HpackHeaderBasic::Path,
        HpackHeaderBasic::Protocol(_) => HpackHeaderBasic::Protocol(value.try_into()?),
        HpackHeaderBasic::Scheme => HpackHeaderBasic::Scheme,
        HpackHeaderBasic::StatusCode(_) => HpackHeaderBasic::StatusCode(value.try_into()?),
      };
      let name = if static_name.is_empty() {
        elem_cb((new_hhb, dyn_name, value))?;
        self.header_buffers.0.clear();
        self.header_buffers.0.extend_from_copyable_slice(dyn_name)?;
        self.header_buffers.0.get_mut(..dyn_name.len()).unwrap_or_default()
      } else {
        elem_cb((new_hhb, static_name, value))?;
        static_name
      };
      (new_hhb, name, value)
    } else {
      let (hhn, name) = Self::decode_string_name(&mut self.header_buffers.0, data)?;
      let value = Self::decode_string_value(&mut self.header_buffers.1, data)?;
      let hhb = HpackHeaderBasic::try_from((hhn, value))?;
      elem_cb((hhb, name, value))?;
      (hhb, name, value)
    };
    if STORE {
      self.dyn_headers.push_front(hhb, name, [value, &[]], false, |_, _| {})?;
    }
    Ok(())
  }

  #[inline]
  fn decode_string_init<'data>(
    data: &mut &'data [u8],
  ) -> crate::Result<(&'data [u8], &'data [u8], bool)> {
    let (first, len) = Self::decode_integer(data, 0b0111_1111)?;
    let Some((bytes_begin, bytes_end)) = data.split_at_checked(*Usize::from(len)) else {
      return Err(crate::Error::Http2ErrorGoAway(
        Http2ErrorCode::CompressionError,
        Some(Http2Error::InsufficientHpackBytes),
      ));
    };
    let is_encoded = first & 0b1000_0000 == 0b1000_0000;
    Ok((bytes_begin, bytes_end, is_encoded))
  }

  #[inline]
  fn decode_string_name<'buffer, 'data, 'rslt>(
    buffer: &'buffer mut _HeaderNameBuffer,
    data: &mut &'data [u8],
  ) -> crate::Result<(HpackHeaderName, &'rslt [u8])>
  where
    'buffer: 'rslt,
    'data: 'rslt,
  {
    let (before, after, is_encoded) = Self::decode_string_init(data)?;
    let rslt = if is_encoded {
      huffman_decode(before, buffer)?;
      (HpackHeaderName::new(buffer)?, &**buffer)
    } else {
      let hhn = HpackHeaderName::new(before)?;
      if hhn.is_field() {
        (hhn, before)
      } else {
        (hhn, &[][..])
      }
    };
    *data = after;
    Ok(rslt)
  }

  #[inline]
  fn decode_string_value<'buffer, 'data, 'rslt>(
    buffer: &'buffer mut _HeaderValueBuffer,
    bytes: &mut &'data [u8],
  ) -> crate::Result<&'rslt [u8]>
  where
    'buffer: 'rslt,
    'data: 'rslt,
  {
    let (before, after, is_encoded) = Self::decode_string_init(bytes)?;
    let rslt = if is_encoded {
      huffman_decode(before, buffer)?;
      buffer
    } else {
      before
    };
    *bytes = after;
    Ok(rslt)
  }

  #[inline]
  fn get(
    dyn_headers: &AbstractHeaders<HpackHeaderBasic>,
    idx: usize,
  ) -> crate::Result<(HpackHeaderBasic, (&'static [u8], &[u8]), (&'static [u8], &[u8]))> {
    Ok(match idx {
      0 => {
        return Err(crate::Error::Http2ErrorGoAway(
          Http2ErrorCode::CompressionError,
          Some(Http2Error::InvalidHpackIdx(Some(0))),
        ))
      }
      1 => (HpackHeaderBasic::Authority, (b":authority", &[]), (&[], &[])),
      2 => (HpackHeaderBasic::Method(Method::Get), (b":method", &[]), (b"GET", &[])),
      3 => (HpackHeaderBasic::Method(Method::Post), (b":method", &[]), (b"POST", &[])),
      4 => (HpackHeaderBasic::Path, (b":path", &[]), (b"/", &[])),
      5 => (HpackHeaderBasic::Path, (b":path", &[]), (b"/index.html", &[])),
      6 => (HpackHeaderBasic::Scheme, (b":scheme", &[]), (b"http", &[])),
      7 => (HpackHeaderBasic::Scheme, (b":scheme", &[]), (b"https", &[])),
      8 => (HpackHeaderBasic::StatusCode(StatusCode::Ok), (b":status", &[]), (b"200", &[])),
      9 => (HpackHeaderBasic::StatusCode(StatusCode::NoContent), (b":status", &[]), (b"204", &[])),
      10 => {
        (HpackHeaderBasic::StatusCode(StatusCode::PartialContent), (b":status", &[]), (b"206", &[]))
      }
      11 => {
        (HpackHeaderBasic::StatusCode(StatusCode::NotModified), (b":status", &[]), (b"304", &[]))
      }
      12 => {
        (HpackHeaderBasic::StatusCode(StatusCode::BadRequest), (b":status", &[]), (b"400", &[]))
      }
      13 => (HpackHeaderBasic::StatusCode(StatusCode::NotFound), (b":status", &[]), (b"404", &[])),
      14 => (
        HpackHeaderBasic::StatusCode(StatusCode::InternalServerError),
        (b":status", &[]),
        (b"500", &[]),
      ),
      15 => (HpackHeaderBasic::Field, (KnownHeaderName::AcceptCharset.into(), &[]), (&[], &[])),
      16 => (
        HpackHeaderBasic::Field,
        (KnownHeaderName::AcceptEncoding.into(), &[]),
        (b"gzip, deflate", &[]),
      ),
      17 => (HpackHeaderBasic::Field, (KnownHeaderName::AcceptLanguage.into(), &[]), (&[], &[])),
      18 => (HpackHeaderBasic::Field, (KnownHeaderName::AcceptRanges.into(), &[]), (&[], &[])),
      19 => (HpackHeaderBasic::Field, (KnownHeaderName::Accept.into(), &[]), (&[], &[])),
      20 => (
        HpackHeaderBasic::Field,
        (KnownHeaderName::AccessControlAllowOrigin.into(), &[]),
        (&[], &[]),
      ),
      21 => (HpackHeaderBasic::Field, (KnownHeaderName::Age.into(), &[]), (&[], &[])),
      22 => (HpackHeaderBasic::Field, (KnownHeaderName::Allow.into(), &[]), (&[], &[])),
      23 => (HpackHeaderBasic::Field, (KnownHeaderName::Authorization.into(), &[]), (&[], &[])),
      24 => (HpackHeaderBasic::Field, (KnownHeaderName::CacheControl.into(), &[]), (&[], &[])),
      25 => {
        (HpackHeaderBasic::Field, (KnownHeaderName::ContentDisposition.into(), &[]), (&[], &[]))
      }
      26 => (HpackHeaderBasic::Field, (KnownHeaderName::ContentEncoding.into(), &[]), (&[], &[])),
      27 => (HpackHeaderBasic::Field, (KnownHeaderName::ContentLanguage.into(), &[]), (&[], &[])),
      28 => (HpackHeaderBasic::Field, (KnownHeaderName::ContentLength.into(), &[]), (&[], &[])),
      29 => (HpackHeaderBasic::Field, (KnownHeaderName::ContentLocation.into(), &[]), (&[], &[])),
      30 => (HpackHeaderBasic::Field, (KnownHeaderName::ContentRange.into(), &[]), (&[], &[])),
      31 => (HpackHeaderBasic::Field, (KnownHeaderName::ContentType.into(), &[]), (&[], &[])),
      32 => (HpackHeaderBasic::Field, (KnownHeaderName::Cookie.into(), &[]), (&[], &[])),
      33 => (HpackHeaderBasic::Field, (KnownHeaderName::Date.into(), &[]), (&[], &[])),
      34 => (HpackHeaderBasic::Field, (KnownHeaderName::Etag.into(), &[]), (&[], &[])),
      35 => (HpackHeaderBasic::Field, (KnownHeaderName::Expect.into(), &[]), (&[], &[])),
      36 => (HpackHeaderBasic::Field, (KnownHeaderName::Expires.into(), &[]), (&[], &[])),
      37 => (HpackHeaderBasic::Field, (KnownHeaderName::From.into(), &[]), (&[], &[])),
      38 => (HpackHeaderBasic::Field, (KnownHeaderName::Host.into(), &[]), (&[], &[])),
      39 => (HpackHeaderBasic::Field, (KnownHeaderName::IfMatch.into(), &[]), (&[], &[])),
      40 => (HpackHeaderBasic::Field, (KnownHeaderName::IfModifiedSince.into(), &[]), (&[], &[])),
      41 => (HpackHeaderBasic::Field, (KnownHeaderName::IfNoneMatch.into(), &[]), (&[], &[])),
      42 => (HpackHeaderBasic::Field, (KnownHeaderName::IfRange.into(), &[]), (&[], &[])),
      43 => (HpackHeaderBasic::Field, (KnownHeaderName::IfUnmodifiedSince.into(), &[]), (&[], &[])),
      44 => (HpackHeaderBasic::Field, (KnownHeaderName::LastModified.into(), &[]), (&[], &[])),
      45 => (HpackHeaderBasic::Field, (KnownHeaderName::Link.into(), &[]), (&[], &[])),
      46 => (HpackHeaderBasic::Field, (KnownHeaderName::Location.into(), &[]), (&[], &[])),
      47 => (HpackHeaderBasic::Field, (KnownHeaderName::MaxForwards.into(), &[]), (&[], &[])),
      48 => (HpackHeaderBasic::Field, (KnownHeaderName::ProxyAuthenticate.into(), &[]), (&[], &[])),
      49 => {
        (HpackHeaderBasic::Field, (KnownHeaderName::ProxyAuthorization.into(), &[]), (&[], &[]))
      }
      50 => (HpackHeaderBasic::Field, (KnownHeaderName::Range.into(), &[]), (&[], &[])),
      51 => (HpackHeaderBasic::Field, (KnownHeaderName::Referer.into(), &[]), (&[], &[])),
      52 => (HpackHeaderBasic::Field, (KnownHeaderName::Refresh.into(), &[]), (&[], &[])),
      53 => (HpackHeaderBasic::Field, (KnownHeaderName::RetryAfter.into(), &[]), (&[], &[])),
      54 => (HpackHeaderBasic::Field, (KnownHeaderName::Server.into(), &[]), (&[], &[])),
      55 => (HpackHeaderBasic::Field, (KnownHeaderName::SetCookie.into(), &[]), (&[], &[])),
      56 => (
        HpackHeaderBasic::Field,
        (KnownHeaderName::StrictTransportSecurity.into(), &[]),
        (&[], &[]),
      ),
      57 => (HpackHeaderBasic::Field, (KnownHeaderName::TransferEncoding.into(), &[]), (&[], &[])),
      58 => (HpackHeaderBasic::Field, (KnownHeaderName::UserAgent.into(), &[]), (&[], &[])),
      59 => (HpackHeaderBasic::Field, (KnownHeaderName::Vary.into(), &[]), (&[], &[])),
      60 => (HpackHeaderBasic::Field, (KnownHeaderName::Via.into(), &[]), (&[], &[])),
      61 => (HpackHeaderBasic::Field, (KnownHeaderName::WwwAuthenticate.into(), &[]), (&[], &[])),
      dyn_idx_with_offset => dyn_headers
        .get_by_idx(dyn_idx_with_offset.wrapping_sub(DYN_IDX_OFFSET))
        .ok_or_else(|| {
          crate::Error::Http2ErrorGoAway(
            Http2ErrorCode::CompressionError,
            Some(Http2Error::InvalidHpackIdx(dyn_idx_with_offset.try_into().ok())),
          )
        })
        .map(|el| (*el.misc, (&[][..], el.name_bytes), (&[][..], el.value_bytes)))?,
    })
  }

  #[inline]
  fn manage_decode(
    &mut self,
    byte: u8,
    data: &mut &[u8],
    elem_cb: &mut impl FnMut((HpackHeaderBasic, &[u8], &[u8])) -> crate::Result<()>,
    mut size_update_cb: impl FnMut() -> crate::Result<()>,
  ) -> crate::Result<()> {
    match DecodeIdx::try_from(byte)? {
      DecodeIdx::Indexed => {
        let idx = Self::decode_integer(data, 0b0111_1111)?.1;
        elem_cb(Self::get(&self.dyn_headers, *Usize::from(idx)).map(|(hhb, name, value)| {
          (
            hhb,
            if name.0.is_empty() { name.1 } else { name.0 },
            if value.0.is_empty() { value.1 } else { value.0 },
          )
        })?)?;
      }
      DecodeIdx::LiteralNeverIndexed | DecodeIdx::LiteralWithoutIndexing => {
        self.decode_literal::<false>(data, 0b0000_1111, elem_cb)?;
      }
      DecodeIdx::LiteralWithIndexing => {
        self.decode_literal::<true>(data, 0b0011_1111, elem_cb)?;
      }
      DecodeIdx::SizeUpdate => {
        size_update_cb()?;
        let local_max_bytes: u32 = Self::decode_integer(data, 0b0001_1111)?.1;
        if local_max_bytes > self.max_bytes.0 {
          return Err(crate::Error::Http2ErrorGoAway(
            Http2ErrorCode::CompressionError,
            Some(Http2Error::OutOfBoundsIndex),
          ));
        }
        self.dyn_headers.set_max_bytes(*Usize::from(local_max_bytes), |_, _| {});
      }
    }
    Ok(())
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

#[cfg(feature = "_bench")]
#[cfg(test)]
mod bench {
  use crate::{
    http2::{HpackDecoder, HpackEncoder},
    misc::{Usize, Vector},
    rng::NoStdRng,
  };

  #[bench]
  fn decode(b: &mut test::Bencher) {
    const N: u32 = 1024 * 1024;
    let data = {
      let mut rslt = crate::bench::_data(*Usize::from(N));
      rslt.iter_mut().filter(|el| **el == b':').for_each(|el| {
        *el = 0;
      });
      rslt
    };
    let mut buffer = Vector::with_capacity(*Usize::from(N)).unwrap();
    let mut he = HpackEncoder::new(NoStdRng::default());
    he.set_max_dyn_super_bytes(N);
    he.encode(&mut buffer, [].into_iter(), {
      data.chunks_exact(128).map(|el| (&el[..64], &el[64..]).into())
    })
    .unwrap();
    let mut hd = HpackDecoder::new();
    hd.set_max_bytes(N);
    b.iter(|| {
      hd.decode(&buffer, |_| Ok(())).unwrap();
      hd.clear();
    });
  }
}
