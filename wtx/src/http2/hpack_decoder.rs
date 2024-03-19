use crate::{
  http::{AbstractHeaders, HeaderName, Method, StatusCode, MAX_HEADER_FIELD_LEN},
  http2::{
    hpack_header::{HpackHeaderBasic, HpackHeaderName},
    huffman_decode,
  },
  misc::{ArrayVector, _unlikely_elem},
};
use alloc::boxed::Box;

type HeaderBuffer = ArrayVector<u8, MAX_HEADER_FIELD_LEN>;

#[derive(Debug)]
pub struct HpackDecoder {
  dyn_headers: AbstractHeaders<HpackHeaderBasic>,
  header_buffers: Box<(HeaderBuffer, HeaderBuffer)>,
  max_dyn_sub_bytes: u16,
  max_dyn_super_bytes: u16,
}

impl HpackDecoder {
  pub(crate) fn with_capacity(bytes: u16, headers: u16, max_bytes: u16) -> Self {
    Self {
      dyn_headers: AbstractHeaders::with_capacity(bytes.into(), headers.into(), max_bytes.into()),
      header_buffers: Box::new((ArrayVector::default(), ArrayVector::default())),
      max_dyn_sub_bytes: max_bytes,
      max_dyn_super_bytes: max_bytes,
    }
  }

  pub(crate) fn _clear(&mut self) {
    let Self { dyn_headers, header_buffers, max_dyn_sub_bytes: _, max_dyn_super_bytes: _ } = self;
    dyn_headers.clear();
    header_buffers.0.clear();
    header_buffers.1.clear();
  }

  pub(crate) fn decode(
    &mut self,
    mut data: &[u8],
    mut cb: impl FnMut((HpackHeaderBasic, &[u8], &[u8])),
  ) -> crate::Result<()> {
    if let [first, ..] = data {
      self.manage_decode(*first, &mut data, &mut cb, || Ok(()))?;
    }
    while let [first, ..] = data {
      self.manage_decode(*first, &mut data, &mut cb, || {
        Err(crate::Error::InvalidDynTableSizeUpdate)
      })?;
    }
    Ok(())
  }

  fn decode_init<'data>(data: &mut &'data [u8]) -> crate::Result<(&'data [u8], &'data [u8], bool)> {
    let (first, len) = Self::decode_integer(data, PrimitiveN::_7)?;
    let (bytes_begin, bytes_end) = if data.len() >= len.into() {
      data.split_at(len.into())
    } else {
      return Err(crate::Error::UnexpectedEOF);
    };
    let is_encoded = first & 0b1000_0000 == 0b1000_0000;
    Ok((bytes_begin, bytes_end, is_encoded))
  }

  fn decode_integer(data: &mut &[u8], pn: PrimitiveN) -> crate::Result<(u8, u16)> {
    let mask =
      if let PrimitiveN::_8 = pn { u8::MAX } else { (1u8 << u8::from(pn)).wrapping_sub(1) };

    let mut rslt: (u8, u16) = if let [first, rest @ ..] = data {
      *data = rest;
      let n = *first & mask;
      let rslt = (*first, n.into());
      if n < mask {
        return Ok(rslt);
      }
      rslt
    } else {
      return Err(crate::Error::UnexpectedEOF);
    };

    let mut shift: u16 = 0;
    for _ in 0..2 {
      let [first, rest @ ..] = data else {
        return Err(crate::Error::UnexpectedEOF);
      };
      rslt.1 = rslt.1.wrapping_add(u16::from(first & 0b0111_1111) << shift);
      shift = shift.wrapping_add(7);
      let is_last_byte = first & 0b1000_000 == 0;
      if is_last_byte {
        return Ok(rslt);
      }
      *data = rest;
    }

    return Err(crate::Error::VeryLargeHeaderInteger);
  }

  fn decode_literal(&mut self, data: &mut &[u8], pn: PrimitiveN) -> crate::Result<()> {
    let idx = Self::decode_integer(data, pn)?.1;
    let has_indexed_name = idx != 0;
    if has_indexed_name {
      let hhb = self.get(idx.into()).unwrap().0;
      let value = Self::decode_string_value(&mut self.header_buffers.1, data)?;
      let new_hhb = match hhb {
        HpackHeaderBasic::Authority => HpackHeaderBasic::Authority,
        HpackHeaderBasic::Field => HpackHeaderBasic::Field,
        HpackHeaderBasic::Method(_) => HpackHeaderBasic::Method(value.try_into()?),
        HpackHeaderBasic::Path => HpackHeaderBasic::Path,
        HpackHeaderBasic::Protocol(_) => HpackHeaderBasic::Protocol(value.try_into()?),
        HpackHeaderBasic::Scheme => HpackHeaderBasic::Scheme,
        HpackHeaderBasic::Status(_) => HpackHeaderBasic::Status(value.try_into()?),
      };
      self.dyn_headers.reserve(value.len(), 1);
      self.dyn_headers.push_front(new_hhb, &[], value, false);
    } else {
      let (hhn, name) = Self::decode_string_name(&mut self.header_buffers.0, data)?;
      let value = Self::decode_string_value(&mut self.header_buffers.1, data)?;
      let hhb = HpackHeaderBasic::try_from((hhn, value))?;
      self.dyn_headers.reserve(name.len().wrapping_add(value.len()), 1);
      self.dyn_headers.push_front(hhb, name, value, false);
    }
    Ok(())
  }

  fn decode_string_name<'buffer, 'data, 'rslt>(
    buffer: &'buffer mut HeaderBuffer,
    data: &mut &'data [u8],
  ) -> crate::Result<(HpackHeaderName, &'rslt [u8])>
  where
    'buffer: 'rslt,
    'data: 'rslt,
  {
    let (before, after, is_encoded) = Self::decode_init(data)?;
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

  fn decode_string_value<'buffer, 'data, 'rslt>(
    buffer: &'buffer mut HeaderBuffer,
    bytes: &mut &'data [u8],
  ) -> crate::Result<&'rslt [u8]>
  where
    'buffer: 'rslt,
    'data: 'rslt,
  {
    let (before, after, is_encoded) = Self::decode_init(bytes)?;
    let rslt = if is_encoded {
      huffman_decode(before, buffer)?;
      buffer
    } else {
      before
    };
    *bytes = after;
    Ok(rslt)
  }

  fn get(&self, idx: usize) -> crate::Result<(HpackHeaderBasic, &[u8], &[u8])> {
    Ok(match idx {
      0 => return _unlikely_elem(Err(crate::Error::InvalidHpackIdx(0))),
      1 => (HpackHeaderBasic::Authority, &[], &[]),
      2 => (HpackHeaderBasic::Method(Method::Get), &[], &[]),
      3 => (HpackHeaderBasic::Method(Method::Post), &[], &[]),
      4 => (HpackHeaderBasic::Path, &[], b"/"),
      5 => (HpackHeaderBasic::Path, &[], b"/index.html"),
      6 => (HpackHeaderBasic::Scheme, &[], b"http"),
      7 => (HpackHeaderBasic::Scheme, &[], b"https"),
      8 => (HpackHeaderBasic::Status(StatusCode::Ok), &[], &[]),
      9 => (HpackHeaderBasic::Status(StatusCode::NoContent), &[], &[]),
      10 => (HpackHeaderBasic::Status(StatusCode::PartialContent), &[], &[]),
      11 => (HpackHeaderBasic::Status(StatusCode::NotModified), &[], &[]),
      12 => (HpackHeaderBasic::Status(StatusCode::BadRequest), &[], &[]),
      13 => (HpackHeaderBasic::Status(StatusCode::NotFound), &[], &[]),
      14 => (HpackHeaderBasic::Status(StatusCode::InternalServerError), &[], &[]),
      15 => (HpackHeaderBasic::Field, HeaderName::ACCEPT_CHARSET.bytes(), &[]),
      16 => (HpackHeaderBasic::Field, HeaderName::ACCEPT_ENCODING.bytes(), b"gzip, deflate"),
      17 => (HpackHeaderBasic::Field, HeaderName::ACCEPT_LANGUAGE.bytes(), &[]),
      18 => (HpackHeaderBasic::Field, HeaderName::ACCEPT_RANGES.bytes(), &[]),
      19 => (HpackHeaderBasic::Field, HeaderName::ACCEPT.bytes(), &[]),
      20 => (HpackHeaderBasic::Field, HeaderName::ACCESS_CONTROL_ALLOW_ORIGIN.bytes(), &[]),
      21 => (HpackHeaderBasic::Field, HeaderName::AGE.bytes(), &[]),
      22 => (HpackHeaderBasic::Field, HeaderName::ALLOW.bytes(), &[]),
      23 => (HpackHeaderBasic::Field, HeaderName::AUTHORIZATION.bytes(), &[]),
      24 => (HpackHeaderBasic::Field, HeaderName::CACHE_CONTROL.bytes(), &[]),
      25 => (HpackHeaderBasic::Field, HeaderName::CONTENT_DISPOSITION.bytes(), &[]),
      26 => (HpackHeaderBasic::Field, HeaderName::CONTENT_ENCODING.bytes(), &[]),
      27 => (HpackHeaderBasic::Field, HeaderName::CONTENT_LANGUAGE.bytes(), &[]),
      28 => (HpackHeaderBasic::Field, HeaderName::CONTENT_LENGTH.bytes(), &[]),
      29 => (HpackHeaderBasic::Field, HeaderName::CONTENT_LOCATION.bytes(), &[]),
      30 => (HpackHeaderBasic::Field, HeaderName::CONTENT_RANGE.bytes(), &[]),
      31 => (HpackHeaderBasic::Field, HeaderName::CONTENT_TYPE.bytes(), &[]),
      32 => (HpackHeaderBasic::Field, HeaderName::COOKIE.bytes(), &[]),
      33 => (HpackHeaderBasic::Field, HeaderName::DATE.bytes(), &[]),
      34 => (HpackHeaderBasic::Field, HeaderName::ETAG.bytes(), &[]),
      35 => (HpackHeaderBasic::Field, HeaderName::EXPECT.bytes(), &[]),
      36 => (HpackHeaderBasic::Field, HeaderName::EXPIRES.bytes(), &[]),
      37 => (HpackHeaderBasic::Field, HeaderName::FROM.bytes(), &[]),
      38 => (HpackHeaderBasic::Field, HeaderName::HOST.bytes(), &[]),
      39 => (HpackHeaderBasic::Field, HeaderName::IF_MATCH.bytes(), &[]),
      40 => (HpackHeaderBasic::Field, HeaderName::IF_MODIFIED_SINCE.bytes(), &[]),
      41 => (HpackHeaderBasic::Field, HeaderName::IF_NONE_MATCH.bytes(), &[]),
      42 => (HpackHeaderBasic::Field, HeaderName::IF_RANGE.bytes(), &[]),
      43 => (HpackHeaderBasic::Field, HeaderName::IF_UNMODIFIED_SINCE.bytes(), &[]),
      44 => (HpackHeaderBasic::Field, HeaderName::LAST_MODIFIED.bytes(), &[]),
      45 => (HpackHeaderBasic::Field, HeaderName::LINK.bytes(), &[]),
      46 => (HpackHeaderBasic::Field, HeaderName::LOCATION.bytes(), &[]),
      47 => (HpackHeaderBasic::Field, HeaderName::MAX_FORWARDS.bytes(), &[]),
      48 => (HpackHeaderBasic::Field, HeaderName::PROXY_AUTHENTICATE.bytes(), &[]),
      49 => (HpackHeaderBasic::Field, HeaderName::PROXY_AUTHORIZATION.bytes(), &[]),
      50 => (HpackHeaderBasic::Field, HeaderName::RANGE.bytes(), &[]),
      51 => (HpackHeaderBasic::Field, HeaderName::REFERER.bytes(), &[]),
      52 => (HpackHeaderBasic::Field, HeaderName::REFRESH.bytes(), &[]),
      53 => (HpackHeaderBasic::Field, HeaderName::RETRY_AFTER.bytes(), &[]),
      54 => (HpackHeaderBasic::Field, HeaderName::SERVER.bytes(), &[]),
      55 => (HpackHeaderBasic::Field, HeaderName::SET_COOKIE.bytes(), &[]),
      56 => (HpackHeaderBasic::Field, HeaderName::STRICT_TRANSPORT_SECURITY.bytes(), &[]),
      57 => (HpackHeaderBasic::Field, HeaderName::TRANSFER_ENCODING.bytes(), &[]),
      58 => (HpackHeaderBasic::Field, HeaderName::USER_AGENT.bytes(), &[]),
      59 => (HpackHeaderBasic::Field, HeaderName::VARY.bytes(), &[]),
      60 => (HpackHeaderBasic::Field, HeaderName::VIA.bytes(), &[]),
      61 => (HpackHeaderBasic::Field, HeaderName::WWW_AUTHENTICATE.bytes(), &[]),
      dyn_idx => self
        .dyn_headers
        .get_by_idx(dyn_idx)
        .ok_or(crate::Error::InvalidHpackIdx(dyn_idx))
        .map(|el| (*el.misc, el.name_bytes, el.value_bytes))?,
    })
  }

  fn manage_decode(
    &mut self,
    byte: u8,
    data: &mut &[u8],
    elem_cb: &mut impl FnMut((HpackHeaderBasic, &[u8], &[u8])),
    size_update_cb: impl Fn() -> crate::Result<()>,
  ) -> crate::Result<()> {
    match IndexTy::try_from(byte)? {
      IndexTy::Indexed => {
        let idx = Self::decode_integer(data, PrimitiveN::_7)?.1;
        elem_cb(self.get(idx.into()).map_err(_unlikely_elem)?)
      }
      IndexTy::LiteralNeverIndexed | IndexTy::LiteralWithoutIndexing => {
        self.decode_literal(data, PrimitiveN::_4)?;
        elem_cb(
          self.dyn_headers.first().map(|el| (*el.misc, el.name_bytes, el.value_bytes)).unwrap(),
        );
        self.dyn_headers.pop_front();
      }
      IndexTy::LiteralWithIndexing => {
        self.decode_literal(data, PrimitiveN::_6)?;
        elem_cb(
          self.dyn_headers.first().map(|el| (*el.misc, el.name_bytes, el.value_bytes)).unwrap(),
        );
      }
      IndexTy::SizeUpdate => {
        size_update_cb()?;
        let max_dyn_sub_bytes = Self::decode_integer(data, PrimitiveN::_5)?.1;
        self.set_max_dyn_sub_bytes(max_dyn_sub_bytes.into())?;
      }
    }
    Ok(())
  }

  pub(crate) fn set_max_dyn_sub_bytes(&mut self, max_dyn_sub_bytes: u16) -> crate::Result<()> {
    if max_dyn_sub_bytes > self.max_dyn_super_bytes {
      return Err(crate::Error::UnboundedNumber {
        expected: 0..=self.max_dyn_super_bytes.into(),
        received: max_dyn_sub_bytes.into(),
      });
    }
    self.max_dyn_sub_bytes = max_dyn_sub_bytes;
    self.dyn_headers.set_max_bytes(max_dyn_sub_bytes.into());
    Ok(())
  }

  pub(crate) fn set_max_dyn_super_bytes(&mut self, max_dyn_super_bytes: u16) {
    self.max_dyn_super_bytes = max_dyn_super_bytes;
    self.dyn_headers.set_max_bytes(max_dyn_super_bytes.into());
  }
}

#[derive(Debug)]
enum IndexTy {
  /// Header is already stored
  Indexed,
  /// Header must not be stored
  LiteralNeverIndexed,
  /// New header that has to stored
  LiteralWithIndexing,
  /// Header must not be stored
  LiteralWithoutIndexing,
  /// Table resizing
  SizeUpdate,
}

impl TryFrom<u8> for IndexTy {
  type Error = crate::Error;

  #[inline]
  fn try_from(from: u8) -> Result<Self, Self::Error> {
    Ok(match from >> 4 {
      0b0000_0000 => Self::LiteralWithoutIndexing,
      0b0000_0001 => Self::LiteralNeverIndexed,
      0b0000_0010 => Self::SizeUpdate,
      0b0000_0100 => Self::LiteralWithIndexing,
      0b0000_1000 => Self::Indexed,
      _ => panic!(),
    })
  }
}

#[derive(Debug)]
enum PrimitiveN {
  _1,
  _2,
  _3,
  _4,
  _5,
  _6,
  _7,
  _8,
}

impl From<PrimitiveN> for u8 {
  #[inline]
  fn from(from: PrimitiveN) -> Self {
    match from {
      PrimitiveN::_1 => 1,
      PrimitiveN::_2 => 2,
      PrimitiveN::_3 => 3,
      PrimitiveN::_4 => 4,
      PrimitiveN::_5 => 5,
      PrimitiveN::_6 => 6,
      PrimitiveN::_7 => 7,
      PrimitiveN::_8 => 8,
    }
  }
}

#[cfg(feature = "_bench")]
#[cfg(test)]
mod bench {
  use crate::http2::HpackDecoder;

  #[bench]
  fn decode(b: &mut test::Bencher) {
    let len: u16 = 64 << 14;
    let data = crate::bench::_data(len.into());
    let mut hh = HpackDecoder::with_capacity(len, len, len);
    b.iter(|| {
      hh.decode(&data, |_| {}).unwrap();
    });
  }
}
