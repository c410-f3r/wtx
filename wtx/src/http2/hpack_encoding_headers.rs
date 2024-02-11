use crate::{
  http::{self, AbstractHeaders, Method, StatusCode},
  http2::{hpack_header::HpackHeaderBasic, huffman_encode},
  misc::{random_state, FilledBufferWriter, Usize, _unreachable},
  rng::Rng,
};
use ahash::RandomState;
use core::hash::{BuildHasher, Hasher};
use hashbrown::HashMap;

#[derive(Debug)]
pub struct HpackEncodingHeaders {
  dyn_headers: AbstractHeaders<HpackHeaderBasic>,
  indcs: HashMap<u64, usize>,
  max_dyn_sub_bytes: u16,
  max_dyn_super_bytes: u16,
  rs: RandomState,
}

impl HpackEncodingHeaders {
  pub fn new<RNG>(capacity: u16, max_bytes: u16, rng: RNG) -> Self
  where
    RNG: Rng,
  {
    Self {
      dyn_headers: AbstractHeaders::with_capacity(capacity.into()),
      indcs: HashMap::new(),
      max_dyn_sub_bytes: max_bytes,
      max_dyn_super_bytes: max_bytes,
      rs: random_state(rng),
    }
  }

  pub fn encode<'value>(
    &mut self,
    fbw: &mut FilledBufferWriter<'_>,
    pseudo_headers: impl IntoIterator<Item = (HpackHeaderBasic, &'value [u8])>,
    user_headers: impl IntoIterator<Item = (HpackHeaderBasic, &'value [u8], &'value [u8])>,
  ) -> crate::Result<()> {
    for (hhb, value) in pseudo_headers {
      let idx = 'idx: {
        let static_header_idx = Self::pseudo_header_idx((hhb, value));
        if let Some(HeaderIdx { has_known_value: true, idx }) = static_header_idx {
          break 'idx Idx::IndexedNameAndValue(idx);
        }
        self.dyn_idx(hhb, &[], value)
      };
      self.manage_encode(fbw, idx);
    }

    for (hhb, name, value) in user_headers {
      let idx = 'idx: {
        let dyn_header_idx = Self::dyn_header_idx((hhb, name, value));
        if let Some(HeaderIdx { has_known_value: true, idx }) = dyn_header_idx {
          break 'idx Idx::IndexedNameAndValue(idx);
        }
        self.dyn_idx(hhb, name, value)
      };
      self.manage_encode(fbw, idx);
    }

    Ok(())
  }

  fn dyn_header_idx((hhb, name, value): (HpackHeaderBasic, &[u8], &[u8])) -> Option<HeaderIdx> {
    let HpackHeaderBasic::Field = hhb else {
      return None;
    };
    let (idx, has_known_value) = match name {
      elem if elem == http::ACCEPT_CHARSET.bytes() => (15, false),
      elem if elem == http::ACCEPT_ENCODING.bytes() => {
        if value == b"gzip, deflate" {
          (16, true)
        } else {
          (16, false)
        }
      }
      elem if elem == http::ACCEPT_LANGUAGE.bytes() => (17, false),
      elem if elem == http::ACCEPT_RANGES.bytes() => (18, false),
      elem if elem == http::ACCEPT.bytes() => (19, false),
      elem if elem == http::ACCESS_CONTROL_ALLOW_ORIGIN.bytes() => (20, false),
      elem if elem == http::AGE.bytes() => (21, false),
      elem if elem == http::ALLOW.bytes() => (22, false),
      elem if elem == http::AUTHORIZATION.bytes() => (23, false),
      elem if elem == http::CACHE_CONTROL.bytes() => (24, false),
      elem if elem == http::CONTENT_DISPOSITION.bytes() => (25, false),
      elem if elem == http::CONTENT_ENCODING.bytes() => (26, false),
      elem if elem == http::CONTENT_LANGUAGE.bytes() => (27, false),
      elem if elem == http::CONTENT_LENGTH.bytes() => (28, false),
      elem if elem == http::CONTENT_LOCATION.bytes() => (29, false),
      elem if elem == http::CONTENT_RANGE.bytes() => (30, false),
      elem if elem == http::CONTENT_TYPE.bytes() => (31, false),
      elem if elem == http::COOKIE.bytes() => (32, false),
      elem if elem == http::DATE.bytes() => (33, false),
      elem if elem == http::ETAG.bytes() => (34, false),
      elem if elem == http::EXPECT.bytes() => (35, false),
      elem if elem == http::EXPIRES.bytes() => (36, false),
      elem if elem == http::FROM.bytes() => (37, false),
      elem if elem == http::HOST.bytes() => (38, false),
      elem if elem == http::IF_MATCH.bytes() => (39, false),
      elem if elem == http::IF_MODIFIED_SINCE.bytes() => (40, false),
      elem if elem == http::IF_NONE_MATCH.bytes() => (41, false),
      elem if elem == http::IF_RANGE.bytes() => (42, false),
      elem if elem == http::IF_UNMODIFIED_SINCE.bytes() => (43, false),
      elem if elem == http::LAST_MODIFIED.bytes() => (44, false),
      elem if elem == http::LINK.bytes() => (45, false),
      elem if elem == http::LOCATION.bytes() => (46, false),
      elem if elem == http::MAX_FORWARDS.bytes() => (47, false),
      elem if elem == http::PROXY_AUTHENTICATE.bytes() => (48, false),
      elem if elem == http::PROXY_AUTHORIZATION.bytes() => (49, false),
      elem if elem == http::RANGE.bytes() => (50, false),
      elem if elem == http::REFERER.bytes() => (51, false),
      elem if elem == http::REFRESH.bytes() => (52, false),
      elem if elem == http::RETRY_AFTER.bytes() => (53, false),
      elem if elem == http::SERVER.bytes() => (54, false),
      elem if elem == http::SET_COOKIE.bytes() => (55, false),
      elem if elem == http::STRICT_TRANSPORT_SECURITY.bytes() => (56, false),
      elem if elem == http::TRANSFER_ENCODING.bytes() => (57, false),
      elem if elem == http::USER_AGENT.bytes() => (58, false),
      elem if elem == http::VARY.bytes() => (59, false),
      elem if elem == http::VIA.bytes() => (60, false),
      elem if elem == http::WWW_AUTHENTICATE.bytes() => (61, false),
      _ => return None,
    };
    Some(HeaderIdx { has_known_value, idx })
  }

  fn dyn_idx(&mut self, hhb: HpackHeaderBasic, name: &[u8], value: &[u8]) -> Idx {
    let pair_hash = {
      let mut hasher = self.rs.build_hasher();
      hasher.write(name);
      hasher.write(value);
      hasher.finish()
    };
    if let Some(idx) = self.indcs.get(&pair_hash).and_then(|&idx| idx.try_into().ok()) {
      return Idx::IndexedNameAndValue(idx);
    }

    let name_hash = self.rs.hash_one(hhb.name(name));
    if let Some(name_idx) = self.indcs.get(&name_hash).copied() {
      let value_idx = self.dyn_headers.elements_len();
      self.dyn_headers.push(hhb, &[], value);
      return Idx::IndexedNameLiteralValue(name_idx, *Usize::from(value_idx));
    }

    let idx = self.dyn_headers.elements_len();
    self.dyn_headers.push(hhb, name, value);
    Idx::LiteralNameAndValue(*Usize::from(idx))
  }

  fn encode_int(fbw: &mut FilledBufferWriter<'_>, first_byte: u8, mask: u8, mut n: u16) {
    if n < mask.into() {
      fbw._extend_from_byte(first_byte | n as u8);
      return;
    }

    n = n.wrapping_sub(mask.into());
    fbw._extend_from_byte(first_byte | mask);

    for _ in 0..2 {
      if n < 128 {
        break;
      }
      fbw._extend_from_byte(0b1000_0000 | n as u8);
      n >>= 7;
    }

    fbw._extend_from_byte(n as u8);
  }

  fn encode_str(bytes: &[u8], fbw: &mut FilledBufferWriter<'_>) {
    if bytes.is_empty() {
      fbw._extend_from_byte(0);
      return;
    }

    let idx = fbw._len();
    fbw._extend_from_byte(0);
    huffman_encode(bytes, fbw);

    let huff_len = fbw._len().wrapping_sub(idx.wrapping_add(1));

    if huff_len < 0b0111_1111 {
      let Some(byte) = fbw._curr_bytes().get_mut(idx) else {
        _unreachable();
      };
      *byte = 0b1000_0000 | huff_len as u8;
    } else {
      // Write the head to a placeholder
      const PLACEHOLDER_LEN: usize = 8;
      let mut buf = [0u8; PLACEHOLDER_LEN];

      let head_len = {
        let mut head_dst = &mut buf[..];
        Self::encode_int(fbw, 0b1000_0000, 0b0111_1111, huff_len);
        PLACEHOLDER_LEN - head_dst.remaining_mut()
      };

      // This is just done to reserve space in the destination
      fbw._extend_from_slice(&buf[1..head_len]);

      // Shift the header forward
      for i in 0..huff_len {
        let src_i = idx + 1 + (huff_len - (i + 1));
        let dst_i = idx + head_len + (huff_len - (i + 1));
        dst[dst_i] = dst[src_i];
      }

      // Copy in the head
      for i in 0..head_len {
        dst[idx + i] = buf[i];
      }
    }
  }

  fn manage_encode(&mut self, fbw: &mut FilledBufferWriter<'_>, idx: Idx) {
    match idx {
      Idx::IndexedNameAndValue(idx) => {
        Self::encode_int(fbw, 0b1000_0000, 0b0111_1111, idx);
      }
      Idx::IndexedNameLiteralValue(_, _) => todo!(),
      Idx::LiteralNameAndValue(_) => todo!(),
      Idx::InsertedValue(_, _) => todo!(),
      Idx::NotInserted(_) => todo!(),
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
      HpackHeaderBasic::Protocol => return None,
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
  IndexedNameLiteralValue(usize, usize), // Name
  /// Both elements are encoded as literals
  LiteralNameAndValue(usize), // Inserted
  /// The value has been inserted into the buffer
  InsertedValue(usize, usize), // InsertedValue
  /// Not stored
  NotInserted(HpackHeaderBasic), // NotIndexed
}

struct HeaderIdx {
  has_known_value: bool,
  idx: u16,
}
