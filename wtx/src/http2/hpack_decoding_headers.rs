use crate::{
  http::{self, AbstractHeaders, Method, StatusCode},
  http2::{
    hpack_header::{HpackHeaderBasic, HpackHeaderName},
    huffman_decode,
  },
  misc::{Usize, _unlikely_dflt, _unlikely_elem, _usize_range_from_u32_range},
};
use core::ops::Range;

#[derive(Debug, Eq, PartialEq)]
pub struct HpackDecodingHeaders {
  dyn_headers: AbstractHeaders<HpackHeaderBasic>,
  max_dyn_sub_bytes: u16,
  max_dyn_super_bytes: u16,
}

impl HpackDecodingHeaders {
  pub fn new(capacity: u16, max_bytes: u16) -> Self {
    Self {
      dyn_headers: AbstractHeaders::with_capacity(capacity.into()),
      max_dyn_sub_bytes: max_bytes,
      max_dyn_super_bytes: max_bytes,
    }
  }

  pub fn decode(
    &mut self,
    mut nb: &[u8],
    mut cb: impl FnMut((HpackHeaderBasic, &[u8], &[u8])),
  ) -> crate::Result<()> {
    if let [first, rest @ ..] = nb {
      nb = rest;
      self.manage_decode(*first, &mut nb, &mut cb, || Ok(()))?;
    }
    while let [first, rest @ ..] = nb {
      nb = rest;
      self
        .manage_decode(*first, &mut nb, &mut cb, || Err(crate::Error::InvalidDynTableSizeUpdate))?;
    }
    Ok(())
  }

  fn decode_integer(bytes: &mut &[u8], pn: PrimitiveN) -> crate::Result<(u8, u16)> {
    let mask =
      if let PrimitiveN::_8 = pn { u8::MAX } else { (1u8 << u8::from(pn)).wrapping_sub(1) };

    let mut rslt: (_, u16) = if let [first, rest @ ..] = bytes {
      *bytes = rest;
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
      let [first, rest @ ..] = bytes else {
        return Err(crate::Error::UnexpectedEOF);
      };
      rslt.1 = rslt.1.wrapping_add(u16::from(first & 0b0111_1111) << shift);
      shift = shift.wrapping_add(7);
      let is_last_byte = first & 0b1000_000 == 0;
      if is_last_byte {
        return Ok(rslt);
      }
      *bytes = rest;
    }

    return Err(crate::Error::VeryLargeHeaderInteger);
  }

  fn decode_literal(&mut self, bytes: &mut &[u8], pn: PrimitiveN) -> crate::Result<()> {
    let dyn_idx = Self::decode_integer(bytes, pn)?.1;
    if dyn_idx == 0 {
      let name = self.decode_string_name(bytes)?;
      let value_range = self.decode_string_value(bytes)?;
      let hhb = HpackHeaderBasic::try_from((
        name.0,
        self
          .dyn_headers
          .buffer()
          .get(_usize_range_from_u32_range(value_range.clone()))
          .unwrap_or_else(_unlikely_dflt),
      ))?;
      self.dyn_headers.push_metadata(hhb, name.1.start, name.1.end, value_range.end);
    } else {
      let ab = self.dyn_headers.get_by_idx(dyn_idx.into()).unwrap();
      let hhb = *ab.misc;
      let value_range = self.decode_string_value(bytes)?;
      let value_bytes = self
        .dyn_headers
        .buffer()
        .get(_usize_range_from_u32_range(value_range.clone()))
        .unwrap_or_else(_unlikely_dflt);
      let new_hhb = match hhb {
        HpackHeaderBasic::Authority => HpackHeaderBasic::Authority,
        HpackHeaderBasic::Field => HpackHeaderBasic::Field,
        HpackHeaderBasic::Method(_) => HpackHeaderBasic::Method(value_bytes.try_into()?),
        HpackHeaderBasic::Path => HpackHeaderBasic::Path,
        HpackHeaderBasic::Protocol => HpackHeaderBasic::Protocol,
        HpackHeaderBasic::Scheme => HpackHeaderBasic::Scheme,
        HpackHeaderBasic::Status(_) => HpackHeaderBasic::Status(value_bytes.try_into()?),
      };
      self.dyn_headers.push_metadata(
        new_hhb,
        value_range.start,
        value_range.start,
        value_range.end,
      );
    }
    Ok(())
  }

  fn decode_string<'any, T>(
    &mut self,
    bytes: &mut &'any [u8],
    cb: impl FnOnce(&mut Self, u32, bool, &[u8]) -> crate::Result<T>,
  ) -> crate::Result<(T, Range<u32>)> {
    let (first, len) = Self::decode_integer(bytes, PrimitiveN::_7)?;
    let (bytes_begin, bytes_end) = if bytes.len() >= len.into() {
      bytes.split_at(len.into())
    } else {
      return Err(crate::Error::UnexpectedEOF);
    };
    let decoded_begin = self.dyn_headers.bytes_len();
    let is_encoded = first & 0b1000_0000 == 0b1000_0000;
    let rslt = cb(self, decoded_begin, is_encoded, bytes_begin)?;
    let decoded_end = self.dyn_headers.bytes_len();
    *bytes = bytes_end;
    Ok((rslt, decoded_begin..decoded_end))
  }

  fn decode_string_name<'any>(
    &mut self,
    bytes: &mut &'any [u8],
  ) -> crate::Result<(HpackHeaderName, Range<u32>)> {
    self.decode_string(bytes, |this, idx, is_encoded, local_bytes| {
      if is_encoded {
        huffman_decode(local_bytes, this.dyn_headers.buffer_mut())?;
        HpackHeaderName::new(
          this.dyn_headers.buffer().get(*Usize::from(idx)..).unwrap_or(_unlikely_dflt()),
        )
      } else {
        let hht = HpackHeaderName::new(local_bytes)?;
        if hht.is_field() {
          this.dyn_headers.buffer_mut().extend(local_bytes);
        }
        Ok(hht)
      }
    })
  }

  fn decode_string_value<'any>(&mut self, bytes: &mut &'any [u8]) -> crate::Result<Range<u32>> {
    self
      .decode_string(bytes, |this, _, is_encoded, local_bytes| {
        if is_encoded {
          huffman_decode(local_bytes, this.dyn_headers.buffer_mut())?;
        } else {
          this.dyn_headers.buffer_mut().extend(local_bytes);
        }
        crate::Result::Ok(())
      })
      .map(|(_, el)| el)
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
      15 => (HpackHeaderBasic::Field, http::ACCEPT_CHARSET.bytes(), &[]),
      16 => (HpackHeaderBasic::Field, http::ACCEPT_ENCODING.bytes(), b"gzip, deflate"),
      17 => (HpackHeaderBasic::Field, http::ACCEPT_LANGUAGE.bytes(), &[]),
      18 => (HpackHeaderBasic::Field, http::ACCEPT_RANGES.bytes(), &[]),
      19 => (HpackHeaderBasic::Field, http::ACCEPT.bytes(), &[]),
      20 => (HpackHeaderBasic::Field, http::ACCESS_CONTROL_ALLOW_ORIGIN.bytes(), &[]),
      21 => (HpackHeaderBasic::Field, http::AGE.bytes(), &[]),
      22 => (HpackHeaderBasic::Field, http::ALLOW.bytes(), &[]),
      23 => (HpackHeaderBasic::Field, http::AUTHORIZATION.bytes(), &[]),
      24 => (HpackHeaderBasic::Field, http::CACHE_CONTROL.bytes(), &[]),
      25 => (HpackHeaderBasic::Field, http::CONTENT_DISPOSITION.bytes(), &[]),
      26 => (HpackHeaderBasic::Field, http::CONTENT_ENCODING.bytes(), &[]),
      27 => (HpackHeaderBasic::Field, http::CONTENT_LANGUAGE.bytes(), &[]),
      28 => (HpackHeaderBasic::Field, http::CONTENT_LENGTH.bytes(), &[]),
      29 => (HpackHeaderBasic::Field, http::CONTENT_LOCATION.bytes(), &[]),
      30 => (HpackHeaderBasic::Field, http::CONTENT_RANGE.bytes(), &[]),
      31 => (HpackHeaderBasic::Field, http::CONTENT_TYPE.bytes(), &[]),
      32 => (HpackHeaderBasic::Field, http::COOKIE.bytes(), &[]),
      33 => (HpackHeaderBasic::Field, http::DATE.bytes(), &[]),
      34 => (HpackHeaderBasic::Field, http::ETAG.bytes(), &[]),
      35 => (HpackHeaderBasic::Field, http::EXPECT.bytes(), &[]),
      36 => (HpackHeaderBasic::Field, http::EXPIRES.bytes(), &[]),
      37 => (HpackHeaderBasic::Field, http::FROM.bytes(), &[]),
      38 => (HpackHeaderBasic::Field, http::HOST.bytes(), &[]),
      39 => (HpackHeaderBasic::Field, http::IF_MATCH.bytes(), &[]),
      40 => (HpackHeaderBasic::Field, http::IF_MODIFIED_SINCE.bytes(), &[]),
      41 => (HpackHeaderBasic::Field, http::IF_NONE_MATCH.bytes(), &[]),
      42 => (HpackHeaderBasic::Field, http::IF_RANGE.bytes(), &[]),
      43 => (HpackHeaderBasic::Field, http::IF_UNMODIFIED_SINCE.bytes(), &[]),
      44 => (HpackHeaderBasic::Field, http::LAST_MODIFIED.bytes(), &[]),
      45 => (HpackHeaderBasic::Field, http::LINK.bytes(), &[]),
      46 => (HpackHeaderBasic::Field, http::LOCATION.bytes(), &[]),
      47 => (HpackHeaderBasic::Field, http::MAX_FORWARDS.bytes(), &[]),
      48 => (HpackHeaderBasic::Field, http::PROXY_AUTHENTICATE.bytes(), &[]),
      49 => (HpackHeaderBasic::Field, http::PROXY_AUTHORIZATION.bytes(), &[]),
      50 => (HpackHeaderBasic::Field, http::RANGE.bytes(), &[]),
      51 => (HpackHeaderBasic::Field, http::REFERER.bytes(), &[]),
      52 => (HpackHeaderBasic::Field, http::REFRESH.bytes(), &[]),
      53 => (HpackHeaderBasic::Field, http::RETRY_AFTER.bytes(), &[]),
      54 => (HpackHeaderBasic::Field, http::SERVER.bytes(), &[]),
      55 => (HpackHeaderBasic::Field, http::SET_COOKIE.bytes(), &[]),
      56 => (HpackHeaderBasic::Field, http::STRICT_TRANSPORT_SECURITY.bytes(), &[]),
      57 => (HpackHeaderBasic::Field, http::TRANSFER_ENCODING.bytes(), &[]),
      58 => (HpackHeaderBasic::Field, http::USER_AGENT.bytes(), &[]),
      59 => (HpackHeaderBasic::Field, http::VARY.bytes(), &[]),
      60 => (HpackHeaderBasic::Field, http::VIA.bytes(), &[]),
      61 => (HpackHeaderBasic::Field, http::WWW_AUTHENTICATE.bytes(), &[]),
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
    bytes: &mut &[u8],
    elem_cb: &mut impl FnMut((HpackHeaderBasic, &[u8], &[u8])),
    size_update_cb: impl Fn() -> crate::Result<()>,
  ) -> crate::Result<()> {
    match IndexTy::try_from(byte)? {
      IndexTy::Indexed => {
        let idx = Self::decode_integer(bytes, PrimitiveN::_7)?.1;
        elem_cb(self.get(idx.into()).map_err(_unlikely_elem)?)
      }
      IndexTy::LiteralNeverIndexed | IndexTy::LiteralWithoutIndexing => {
        self.decode_literal(bytes, PrimitiveN::_4)?;
        elem_cb(
          self.dyn_headers.last().map(|el| (*el.misc, el.name_bytes, el.value_bytes)).unwrap(),
        );
        self.dyn_headers.pop_back();
      }
      IndexTy::LiteralWithIndexing => {
        self.decode_literal(bytes, PrimitiveN::_6)?;
        elem_cb(
          self.dyn_headers.last().map(|el| (*el.misc, el.name_bytes, el.value_bytes)).unwrap(),
        );
      }
      IndexTy::SizeUpdate => {
        size_update_cb()?;
        let max_dyn_sub_bytes = Self::decode_integer(bytes, PrimitiveN::_5)?.1;
        self.set_max_dyn_sub_bytes(max_dyn_sub_bytes.into())?;
      }
    }
    Ok(())
  }

  fn set_max_dyn_sub_bytes(&mut self, max_dyn_sub_bytes: u16) -> crate::Result<()> {
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

  pub fn set_max_dyn_super_bytes(&mut self, max_dyn_super_bytes: u16) {
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
  use crate::http2::HpackDecodingHeaders;

  #[bench]
  fn decode(b: &mut test::Bencher) {
    let len: u16 = 64 << 14;
    let data = crate::bench::_data(len.into());
    let mut hh = HpackDecodingHeaders::new(len, len);
    b.iter(|| {
      hh.decode(&data, |_| {}).unwrap();
    });
  }
}
