use crate::{
  http::{AbstractHeaders, HeaderName, Method, StatusCode},
  http2::{hpack_header::HpackHeaderBasic, huffman_encode},
  misc::{ByteVector, Usize, _random_state, _unreachable},
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

  pub(crate) fn clear(&mut self) {
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
      let header_idx_opt = Self::pseudo_header_idx((hhb, value));
      match self.manage_should_not_index(header_idx_opt, hhb, false, &[], value) {
        Err(idx) => {
          self.manage_encode((&[], value), idx, wb)?;
        }
        Ok(should_not_index) => {
          let idx = self.dyn_idx((&[], value, false), header_idx_opt, hhb, should_not_index);
          self.manage_encode((&[], value), idx, wb)?;
        }
      }
    }

    for (name, value, is_sensitive) in user_headers {
      let header_idx_opt = Self::dyn_header_idx((name, value));
      let hhb = HpackHeaderBasic::Field;
      match self.manage_should_not_index(header_idx_opt, hhb, is_sensitive, name, value) {
        Err(idx) => {
          self.manage_encode((name, value), idx, wb)?;
        }
        Ok(should_not_index) => {
          let idx = self.dyn_idx((name, value, is_sensitive), header_idx_opt, hhb, should_not_index);
          self.manage_encode((name, value), idx, wb)?;
        }
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
    header: (&[u8], &[u8], bool),
    header_idx_opt: Option<HeaderIdx>,
    hhb: HpackHeaderBasic,
    should_not_index: bool,
  ) -> Idx {
    let (name, value, is_sensitive) = header;

    let mut name_hasher = self.rs.build_hasher();
    name_hasher.write(name);

    let mut pair_hasher = name_hasher.clone();
    pair_hasher.write(value);
    let pair_hash = pair_hasher.finish();

    match (should_not_index, self.indcs.get(&pair_hash).copied()) {
      (false, None) => {}
      (false, Some(idx)) => return Idx::SavedIndexNameSavedIndexValue(idx),
      (true, _) => return Idx::UnsavedLiteralNameUnsavedLiteralValue,
    }

    let name_hash = name_hasher.finish();

    match (should_not_index, self.indcs.get(&name_hash).copied()) {
      (false, None) => {}
      (false, Some(idx)) => {
        self.dyn_headers.push_front(hhb, &[], value, is_sensitive);
        return Idx::SavedIndexNameSavedLiteralValue(idx);
      }
      (true, None) => return Idx::UnsavedLiteralNameUnsavedLiteralValue,
      (true, Some(idx)) => return Idx::SavedIndexNameUnsavedLiteralValue(idx),
    }

    let idx = self.dyn_headers.headers_len();
    self.dyn_headers.reserve(name.len().wrapping_add(value.len()), 1);
    self.dyn_headers.push_front(hhb, name, value, is_sensitive);
    self.indcs.insert(name_hash, 0);
    self.indcs.insert(pair_hash, 0);
    if let Some(header_idx) = header_idx_opt {
      Idx::SavedIndexNameSavedLiteralValue(header_idx.idx)
    } else {
      Idx::SavedIndexNameSavedIndexValue(0)
    }
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

  // Regardless of the "sensitive" flag set by users, these headers may carry sensitive content
  // that shouldn't be indexed.
  fn header_is_naturally_sensitive(hhb: HpackHeaderBasic, name: &[u8]) -> bool {
    match hhb {
      HpackHeaderBasic::Field => matches!(
        HeaderName::new(name),
        HeaderName::AGE
          | HeaderName::AUTHORIZATION
          | HeaderName::CONTENT_LENGTH
          | HeaderName::ETAG
          | HeaderName::IF_MODIFIED_SINCE
          | HeaderName::IF_NONE_MATCH
          | HeaderName::LOCATION
          | HeaderName::COOKIE
          | HeaderName::SET_COOKIE
      ),
      HpackHeaderBasic::Path => true,
      _ => false,
    }
  }

  // Very large headers are not good candidates for indexing.
  fn header_is_very_large(&self, hhb: HpackHeaderBasic, name: &[u8], value: &[u8]) -> bool {
    hhb
      .len(name, value)
      .checked_mul(4)
      .and_then(|lhs| Some(lhs > usize::from(self.max_dyn_sub_bytes).checked_mul(3)?))
      .unwrap_or_default()
  }

  fn manage_encode(
    &mut self,
    header: (&[u8], &[u8]),
    idx: Idx,
    wb: &mut ByteVector,
  ) -> crate::Result<()> {
    let (name, value) = header;
    match idx {
      Idx::SavedIndexNameSavedIndexValue(idx) => {
        Self::encode_int(0b1000_0000, 0b0111_1111, idx, wb);
      }
      Idx::SavedIndexNameSavedLiteralValue(name_idx) => {
        Self::encode_int(0b0100_0000, 0b0011_1111, name_idx, wb);
        Self::encode_str(value, wb)?;
      }
      Idx::SavedIndexNameUnsavedLiteralValue(name_idx) => {
        Self::encode_int(0b0001_0000, 0b0000_1111, name_idx, wb);
        Self::encode_str(value, wb)?;
      }
      Idx::UnsavedLiteralNameSavedLiteralValue => {
        wb.push(0b0100_0000);
        Self::encode_str(name, wb)?;
        Self::encode_str(value, wb)?;
      }
      Idx::UnsavedLiteralNameUnsavedLiteralValue => {
        wb.push(0b0001_0000);
        Self::encode_str(name, wb)?;
        Self::encode_str(value, wb)?;
      }
    }
    Ok(())
  }

  /// If an index is found, returns Err. Otherwise, returns if the header should be indexed.
  ///
  /// `Result` is used as a convenient wrapper.
  fn manage_should_not_index(
    &self,
    header_idx: Option<HeaderIdx>,
    hhb: HpackHeaderBasic,
    is_sensitive: bool,
    name: &[u8],
    value: &[u8],
  ) -> Result<bool, Idx> {
    match header_idx {
      None => Ok(self.should_not_index(hhb, is_sensitive, name, value)),
      Some(HeaderIdx { has_known_value: true, idx }) => {
        Err(Idx::SavedIndexNameSavedIndexValue(idx))
      }
      Some(HeaderIdx { has_known_value: false, idx }) => {
        if self.should_not_index(hhb, is_sensitive, name, value) {
          Err(Idx::SavedIndexNameUnsavedLiteralValue(idx))
        } else {
          Ok(false)
        }
      }
    }
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

  fn should_not_index(
    &self,
    hhb: HpackHeaderBasic,
    is_sensitive: bool,
    name: &[u8],
    value: &[u8],
  ) -> bool {
    is_sensitive
      || Self::header_is_naturally_sensitive(hhb, name)
      || self.header_is_very_large(hhb, name, value)
  }
}

/// https://datatracker.ietf.org/doc/html/rfc7541#section-6.2
#[derive(Debug)]
pub(crate) enum Idx {
  /// Both elements have been saved and the common referenced index is used for encoding.
  SavedIndexNameSavedIndexValue(u16), // Indexed - Ok
  /// The name is saved and the referenced index is used for encoding. The value is saved and the
  /// literal contents are used for encoding.
  SavedIndexNameSavedLiteralValue(u16), // InsertedValue
  /// Both "Never Indexed" and "Without Indexing" variants.
  ///
  /// The name is saved and the referenced index is used for encoding. The value is not saved
  /// and the literal contents are used for encoding.
  SavedIndexNameUnsavedLiteralValue(u16), // Name or "Err" if previous header is NOT "NotIndexed" - Ok
  /// The name is not saved and the literal contents are used for encoding. The value is saved and
  /// the literal contents are used for encoding.
  UnsavedLiteralNameSavedLiteralValue, // Inserted
  /// Both "Never Indexed" and "Without Indexing" variants.
  ///
  /// The name is not saved and the literal contents are used for encoding. The value is not saved
  /// and the literal contents are used for encoding.
  UnsavedLiteralNameUnsavedLiteralValue, // NotIndexed or "Err" if previous header is "NotIndexed" - Ok
}

#[derive(Clone, Copy, Debug)]
struct HeaderIdx {
  has_known_value: bool,
  idx: u16,
}
