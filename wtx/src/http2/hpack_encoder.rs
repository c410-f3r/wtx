use crate::{
  http::{AbstractHeaders, HeaderName, Method, StatusCode},
  http2::{hpack_header::HpackHeaderBasic, huffman_encode},
  misc::{ByteVector, _random_state, _unreachable},
  rng::Rng,
};
use ahash::RandomState;
use core::hash::{BuildHasher, Hasher};
use hashbrown::HashMap;

#[derive(Debug)]
pub struct HpackEncoder {
  dyn_headers: AbstractHeaders<HpackHeaderBasic>,
  indcs: HashMap<u64, u16>,
  max_dyn_sub_bytes: u16,
  max_dyn_super_bytes: u16,
  rs: RandomState,
}

impl HpackEncoder {
  pub(crate) fn with_capacity<RNG>(bytes: usize, headers: usize, max_bytes: u16, rng: RNG) -> Self
  where
    RNG: Rng,
  {
    Self {
      dyn_headers: AbstractHeaders::with_capacity(bytes, headers, max_bytes.into()),
      indcs: HashMap::with_capacity(headers),
      max_dyn_sub_bytes: max_bytes,
      max_dyn_super_bytes: max_bytes,
      rs: _random_state(rng),
    }
  }

  pub(crate) fn _clear(&mut self) {
    let Self { dyn_headers, indcs, max_dyn_sub_bytes: _, max_dyn_super_bytes: _, rs: _ } = self;
    dyn_headers.clear();
    indcs.clear();
  }

  pub(crate) fn encode<'value>(
    &mut self,
    wb: &mut ByteVector,
    pseudo_headers: impl IntoIterator<Item = (HpackHeaderBasic, &'value [u8])>,
    user_headers: impl IntoIterator<Item = (&'value [u8], &'value [u8], bool)>,
  ) -> crate::Result<()> {
    for (hhb, value) in pseudo_headers {
      let idx = 'idx: {
        let static_header_idx = Self::pseudo_header_idx((hhb, value));
        if let Some(HeaderIdx { has_known_value: true, idx }) = static_header_idx {
          break 'idx Idx::IndexedNameAndValue(idx);
        }
        self.dyn_idx(hhb, &[], value, false)
      };
      self.manage_encode(idx, false, wb)?;
    }

    for (name, value, is_sensitive) in user_headers {
      let idx = 'idx: {
        let dyn_header_idx = Self::dyn_header_idx((name, value));
        if let Some(HeaderIdx { has_known_value: true, idx }) = dyn_header_idx {
          break 'idx Idx::IndexedNameAndValue(idx);
        }
        self.dyn_idx(HpackHeaderBasic::Field, name, value, is_sensitive)
      };
      self.manage_encode(idx, is_sensitive, wb)?;
    }

    Ok(())
  }

  fn dyn_header_idx((name, value): (&[u8], &[u8])) -> Option<HeaderIdx> {
    let (idx, has_known_value) = match HeaderName::new(name) {
      HeaderName::ACCEPT_CHARSET => (15, false),
      HeaderName::ACCEPT_ENCODING => {
        if value == b"gzip, deflate" {
          (16, true)
        } else {
          (16, false)
        }
      }
      HeaderName::ACCEPT_LANGUAGE => (17, false),
      HeaderName::ACCEPT_RANGES => (18, false),
      HeaderName::ACCEPT => (19, false),
      HeaderName::ACCESS_CONTROL_ALLOW_ORIGIN => (20, false),
      HeaderName::AGE => (21, false),
      HeaderName::ALLOW => (22, false),
      HeaderName::AUTHORIZATION => (23, false),
      HeaderName::CACHE_CONTROL => (24, false),
      HeaderName::CONTENT_DISPOSITION => (25, false),
      HeaderName::CONTENT_ENCODING => (26, false),
      HeaderName::CONTENT_LANGUAGE => (27, false),
      HeaderName::CONTENT_LENGTH => (28, false),
      HeaderName::CONTENT_LOCATION => (29, false),
      HeaderName::CONTENT_RANGE => (30, false),
      HeaderName::CONTENT_TYPE => (31, false),
      HeaderName::COOKIE => (32, false),
      HeaderName::DATE => (33, false),
      HeaderName::ETAG => (34, false),
      HeaderName::EXPECT => (35, false),
      HeaderName::EXPIRES => (36, false),
      HeaderName::FROM => (37, false),
      HeaderName::HOST => (38, false),
      HeaderName::IF_MATCH => (39, false),
      HeaderName::IF_MODIFIED_SINCE => (40, false),
      HeaderName::IF_NONE_MATCH => (41, false),
      HeaderName::IF_RANGE => (42, false),
      HeaderName::IF_UNMODIFIED_SINCE => (43, false),
      HeaderName::LAST_MODIFIED => (44, false),
      HeaderName::LINK => (45, false),
      HeaderName::LOCATION => (46, false),
      HeaderName::MAX_FORWARDS => (47, false),
      HeaderName::PROXY_AUTHENTICATE => (48, false),
      HeaderName::PROXY_AUTHORIZATION => (49, false),
      HeaderName::RANGE => (50, false),
      HeaderName::REFERER => (51, false),
      HeaderName::REFRESH => (52, false),
      HeaderName::RETRY_AFTER => (53, false),
      HeaderName::SERVER => (54, false),
      HeaderName::SET_COOKIE => (55, false),
      HeaderName::STRICT_TRANSPORT_SECURITY => (56, false),
      HeaderName::TRANSFER_ENCODING => (57, false),
      HeaderName::USER_AGENT => (58, false),
      HeaderName::VARY => (59, false),
      HeaderName::VIA => (60, false),
      HeaderName::WWW_AUTHENTICATE => (61, false),
      _ => return None,
    };
    Some(HeaderIdx { has_known_value, idx })
  }

  fn dyn_idx(
    &mut self,
    hhb: HpackHeaderBasic,
    name: &[u8],
    value: &[u8],
    is_sensitive: bool,
  ) -> Idx {
    let mut hasher = self.rs.build_hasher();
    hasher.write(name);

    let name_hash = hasher.finish();
    if let Some(name_idx) = self.indcs.get(&name_hash).copied() {
      let value_idx = self.dyn_headers.elements_len();
      self.dyn_headers.push_front(hhb, &[], value, is_sensitive);
      return Idx::IndexedNameLiteralValue(name_idx, value_idx);
    }

    hasher.write(value);
    let pair_hash = hasher.finish();
    if let Some(idx) = self.indcs.get(&pair_hash).and_then(|&idx| idx.try_into().ok()) {
      return Idx::IndexedNameAndValue(idx);
    }

    let idx = self.dyn_headers.elements_len();
    self.dyn_headers.push_front(hhb, name, value, is_sensitive);
    Idx::LiteralNameAndValue(idx)
  }

  fn encode_int(first_byte: u8, mask: u8, mut n: u16, wb: &mut ByteVector) {
    wb.reserve(4);

    if n < mask.into() {
      wb.push_within_cap(first_byte | n as u8);
      return;
    }

    n = n.wrapping_sub(mask.into());
    wb.push_within_cap(first_byte | mask);

    for _ in 0..2 {
      if n < 128 {
        break;
      }
      wb.push_within_cap(0b1000_0000 | n as u8);
      n >>= 7;
    }

    wb.push_within_cap(n as u8);
  }

  fn encode_str(bytes: &[u8], wb: &mut ByteVector) -> crate::Result<()> {
    let idx = wb.len();
    wb.push(0);
    if bytes.is_empty() {
      return Ok(());
    }
    huffman_encode(bytes, wb);
    let len = wb.len().wrapping_sub(idx.wrapping_add(1));
    let (true, Ok(len)) = (len < 0b0111_1111, u8::try_from(len)) else {
      return Err(crate::Error::UnsupportedHeaderNameOrValueLen);
    };
    let Some(byte) = wb.get_mut(idx) else {
      _unreachable();
    };
    *byte = 0b1000_0000 | len;
    Ok(())
  }

  fn manage_encode(
    &mut self,
    idx: Idx,
    is_sensitive: bool,
    wb: &mut ByteVector,
  ) -> crate::Result<()> {
    match idx {
      Idx::IndexedNameAndValue(idx) => {
        Self::encode_int(0b1000_0000, 0b0111_1111, idx, wb);
      }
      Idx::IndexedNameLiteralValue(name_idx, value_idx) => {
        let (first_byte, mask) =
          if is_sensitive { (0b0001_0000, 0b0000_1111) } else { (0, 0b0000_1111) };
        Self::encode_int(first_byte, mask, name_idx, wb);
        let Some(ab) = self.dyn_headers.get_by_idx(value_idx) else {
          unreachable!();
        };
        Self::encode_str(ab.value_bytes, wb)?;
      }
      Idx::InsertedValue(name_idx, value_idx) => {
        let Some(ab) = self.dyn_headers.get_by_idx(value_idx) else {
          unreachable!();
        };
        Self::encode_int(0b0100_0000, 0b0011_1111, name_idx, wb);
        Self::encode_str(ab.value_bytes, wb)?;
      }
      Idx::LiteralNameAndValue(idx) => {
        wb.push(0b0100_0000);
        let Some(ab) = self.dyn_headers.get_by_idx(idx) else {
          unreachable!();
        };
        Self::encode_str(ab.name_bytes, wb)?;
        Self::encode_str(ab.value_bytes, wb)?;
      }
      Idx::NotInserted(idx) => {
        wb.push(if is_sensitive { 0b10000 } else { 0 });
        let Some(ab) = self.dyn_headers.get_by_idx(idx) else {
          unreachable!();
        };
        Self::encode_str(ab.name_bytes, wb)?;
        Self::encode_str(ab.value_bytes, wb)?;
      }
    }
    Ok(())
  }

  fn pseudo_header_idx((hhb, value): (HpackHeaderBasic, &[u8])) -> Option<HeaderIdx> {
    let (idx, has_known_value) = match hhb {
      HpackHeaderBasic::Authority => (1, false),
      HpackHeaderBasic::Field => return None,
      HpackHeaderBasic::Method(method) => match method {
        Method::Get => (2, true),
        Method::Post => (3, true),
        _ => (2, false),
      },
      HpackHeaderBasic::Path => match value {
        b"/" => (4, true),
        b"/index.html" => (5, true),
        _ => (4, false),
      },
      HpackHeaderBasic::Protocol(_) => return None,
      HpackHeaderBasic::Scheme => match value {
        b"http" => (6, true),
        b"https" => (7, true),
        _ => (6, false),
      },
      HpackHeaderBasic::Status(status) => match status {
        StatusCode::Ok => (8, true),
        StatusCode::NoContent => (9, true),
        StatusCode::PartialContent => (10, true),
        StatusCode::NotModified => (11, true),
        StatusCode::BadRequest => (12, true),
        StatusCode::NotFound => (13, true),
        StatusCode::InternalServerError => (14, true),
        _ => (8, false),
      },
    };
    Some(HeaderIdx { has_known_value, idx })
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

  fn set_max_dyn_super_bytes(&mut self, max_dyn_super_bytes: u16) {
    self.max_dyn_super_bytes = max_dyn_super_bytes;
    self.dyn_headers.set_max_bytes(max_dyn_super_bytes.into());
  }
}

#[derive(Debug)]
pub(crate) enum Idx {
  /// Both elements are encoded using the referenced index.
  IndexedNameAndValue(u16), // Indexed
  /// The name is encoded using the referenced index. The value is encoded verbatim.
  IndexedNameLiteralValue(u16, usize), // Name
  /// Both elements are encoded as literals
  LiteralNameAndValue(usize), // Inserted
  /// The value has been inserted into the buffer
  InsertedValue(u16, usize), // InsertedValue
  /// Not stored
  NotInserted(usize), // NotIndexed
}

struct HeaderIdx {
  has_known_value: bool,
  idx: u16,
}
