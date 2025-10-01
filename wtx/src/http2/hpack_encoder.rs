// In `EncodeIdx::RefNameSavedValue` the name is locally indexed along side the value to allow
// future usages of `RefNameRefValue` potentially reducing sent bytes. It is possible to only
// store non-static names but that would de-synchronize the decoder.
//
// On the other hand, `EncodeIdx::RefNameUnsavedValue` does not index anything although the name
// contents could be used to create a more "recent" entry.
//
// It is unknown if the above descriptions are optimal in regards to local stored size, total sent
// bytes or runtime performance.

use crate::{
  collection::Vector,
  http::{Header, KnownHeaderName, Method, StatusCode},
  http2::{
    Http2Error, hpack_header::HpackHeaderBasic, hpack_headers::HpackHeaders,
    huffman::huffman_encode, misc::protocol_err,
  },
  misc::{Usize, bytes_transfer::shift_copyable_chunks, hints::_unreachable, random_state},
  rng::Rng,
};
use core::{
  hash::{BuildHasher, Hasher},
  iter,
};
use foldhash::fast::FixedState;
use hashbrown::HashMap;

const DYN_IDX_OFFSET: u32 = 61;

#[derive(Debug)]
pub(crate) struct HpackEncoder {
  dyn_headers: HpackHeaders<Metadata>,
  idx: u32,
  indcs: HashMap<u64, u32>,
  // Defined by external actors.
  max_dyn_sub_bytes: Option<(u32, Option<u32>)>,
  // Defined by the system.
  max_dyn_super_bytes: u32,
  rs: FixedState,
}

impl HpackEncoder {
  pub(crate) fn new<RNG>(rng: &mut RNG) -> Self
  where
    RNG: Rng,
  {
    Self {
      dyn_headers: HpackHeaders::new(0),
      idx: 0,
      indcs: HashMap::new(),
      max_dyn_sub_bytes: None,
      max_dyn_super_bytes: 0,
      rs: random_state(rng),
    }
  }

  pub(crate) fn clear(&mut self) {
    let Self { dyn_headers, idx, indcs, max_dyn_sub_bytes, max_dyn_super_bytes: _, rs: _ } = self;
    dyn_headers.clear();
    *idx = 0;
    indcs.clear();
    *max_dyn_sub_bytes = None;
  }

  pub(crate) fn encode<'pseudo, 'user>(
    &mut self,
    buffer: &mut Vector<u8>,
    pseudo_headers: impl IntoIterator<Item = (HpackHeaderBasic, &'pseudo str)>,
    user_headers: impl IntoIterator<Item = Header<'user, &'user str>>,
  ) -> crate::Result<()> {
    let pseudo_headers_iter = pseudo_headers.into_iter();
    let user_headers_iter = user_headers.into_iter();
    self.adjust_indices(
      pseudo_headers_iter
        .size_hint()
        .1
        .and_then(|el| el.checked_add(user_headers_iter.size_hint().1?))
        .unwrap_or(usize::MAX),
    );
    let reserve = pseudo_headers_iter.size_hint().0.wrapping_add(user_headers_iter.size_hint().0);
    buffer.reserve(reserve)?;
    self.manage_size_update(buffer)?;
    for (hhb, value) in pseudo_headers_iter {
      let idx = self.encode_idx(("", value, false), hhb, Self::shi_pseudo((hhb, value)))?;
      Self::manage_encode(buffer, ("", value), idx)?;
    }
    for Header { is_sensitive, name, value, .. } in user_headers_iter {
      let idx = self.encode_idx(
        (name, value, is_sensitive),
        HpackHeaderBasic::Field,
        Self::shi_user((name, value)),
      )?;
      Self::manage_encode(buffer, (name, value), idx)?;
    }
    Ok(())
  }

  pub(crate) fn reserve(&mut self, headers: usize, bytes: usize) -> crate::Result<()> {
    self.dyn_headers.reserve(headers, bytes)
  }

  // It is not possible to lower the initial set value
  pub(crate) fn set_max_dyn_sub_bytes(&mut self, max_dyn_sub_bytes: u32) -> crate::Result<()> {
    if max_dyn_sub_bytes > self.max_dyn_super_bytes {
      return Err(crate::Error::UnboundedNumber {
        expected: 0..=self.max_dyn_super_bytes.try_into().unwrap_or(i32::MAX),
        received: max_dyn_sub_bytes.try_into().unwrap_or(i32::MAX),
      });
    }
    match self.max_dyn_sub_bytes {
      Some((lower, None | Some(_))) => {
        if max_dyn_sub_bytes > lower {
          self.max_dyn_sub_bytes = Some((lower, Some(max_dyn_sub_bytes)));
        } else {
          self.max_dyn_sub_bytes = Some((max_dyn_sub_bytes, None));
        }
      }
      None => {
        if max_dyn_sub_bytes != self.max_dyn_super_bytes {
          self.max_dyn_sub_bytes = Some((max_dyn_sub_bytes, None));
        }
      }
    }
    Ok(())
  }

  pub(crate) fn set_max_dyn_super_bytes(&mut self, max_dyn_super_bytes: u32) {
    self.max_dyn_sub_bytes = None;
    self.max_dyn_super_bytes = max_dyn_super_bytes;
  }

  fn adjust_indices(&mut self, len: usize) {
    let new_idx = u64::from(self.idx).checked_add(Usize::from(len).into());
    if new_idx < Some(u64::from(u32::MAX)) {
      return;
    }
    for (_, idx) in &mut self.indcs {
      *idx = idx.wrapping_sub(self.idx);
    }
    self.idx = 0;
  }

  fn dyn_idx(
    &mut self,
    header: (&str, &str, bool),
    should_not_index: bool,
  ) -> crate::Result<EncodeIdx> {
    let (name, value, is_sensitive) = header;

    let mut name_hasher = self.rs.build_hasher();
    name_hasher.write(name.as_bytes());

    let mut pair_hasher = name_hasher.clone();
    pair_hasher.write(value.as_bytes());
    let pair_hash = pair_hasher.finish();

    if let (false, Some(pair_idx)) = (should_not_index, self.indcs.get(&pair_hash).copied()) {
      return Ok(EncodeIdx::RefNameRefValue(self.idx_to_encode_idx(pair_idx)));
    }

    let name_hash = name_hasher.finish();

    match (should_not_index, self.indcs.get(&name_hash).copied()) {
      (false, None) => {}
      (false, Some(name_idx)) => {
        return self.store_header_with_ref_name::<false>(
          (name, value, is_sensitive),
          self.idx_to_encode_idx(name_idx),
          pair_hash,
        );
      }
      (true, None) => return Ok(EncodeIdx::UnsavedNameUnsavedValue),
      (true, Some(name_idx)) => {
        return Ok(EncodeIdx::RefNameUnsavedValue(self.idx_to_encode_idx(name_idx)));
      }
    }

    self.push_dyn_headers((name, value, is_sensitive), (Some(name_hash), pair_hash))?;
    let next_dyn_idx = self.next_dyn_idx();
    self.indcs.reserve(2);
    let _ = self.indcs.insert(name_hash, next_dyn_idx);
    let _ = self.indcs.insert(pair_hash, next_dyn_idx);
    Ok(EncodeIdx::SavedNameSavedValue)
  }

  fn dyn_idx_with_static_name(
    &mut self,
    header: (&str, &str, bool),
    name_idx: u32,
  ) -> crate::Result<EncodeIdx> {
    let (name, value, is_sensitive) = header;
    let pair_hash = self.rs.hash_one((name, value));
    if let Some(common_idx) = self.indcs.get(&pair_hash).copied() {
      return Ok(EncodeIdx::RefNameRefValue(self.idx_to_encode_idx(common_idx)));
    }
    self.store_header_with_ref_name::<true>((name, value, is_sensitive), name_idx, pair_hash)
  }

  fn encode_idx(
    &mut self,
    header: (&str, &str, bool),
    hhb: HpackHeaderBasic,
    static_header: Option<StaticHeader>,
  ) -> crate::Result<EncodeIdx> {
    match static_header {
      None => {
        let (name, value, is_sensitive) = header;
        let should_not_index = self.should_not_index((name, value, is_sensitive), hhb);
        self.dyn_idx((name, value, is_sensitive), should_not_index)
      }
      Some(StaticHeader { has_value: true, idx, name: _ }) => Ok(EncodeIdx::RefNameRefValue(idx)),
      Some(StaticHeader { has_value: false, idx, name }) => {
        let (_, value, is_sensitive) = header;
        if self.should_not_index((name, value, is_sensitive), hhb) {
          Ok(EncodeIdx::RefNameUnsavedValue(idx))
        } else {
          self.dyn_idx_with_static_name((name, value, is_sensitive), idx)
        }
      }
    }
  }

  fn encode_int(buffer: &mut Vector<u8>, first_byte: u8, mut n: u32) -> crate::Result<u8> {
    fn last_byte(n: u32) -> u8 {
      n.to_be_bytes()[3]
    }

    let mask = first_byte.wrapping_sub(1);
    buffer.reserve(4)?;

    if n < u32::from(mask) {
      buffer.push(first_byte | last_byte(n))?;
      return Ok(1);
    }

    n = n.wrapping_sub(mask.into());
    buffer.push(first_byte | mask)?;

    for len in 2..5 {
      if n <= 127 {
        buffer.push(last_byte(n))?;
        return Ok(len);
      }
      buffer.push(0b1000_0000 | last_byte(n))?;
      n >>= 7;
    }

    Err(protocol_err(Http2Error::VeryLargeHeaderInteger))
  }

  // 1. 0 -> 0xxxx -> 4xxxx
  // 2,3,4. 0 -> 0xxxxxxxxxx -> 0xxxxxxxxxx10 -> 10xxxxxxxxxx
  fn encode_str(buffer: &mut Vector<u8>, bytes: &str) -> crate::Result<()> {
    let before_byte = buffer.len();
    buffer.push(0)?;
    if bytes.is_empty() {
      return Ok(());
    }
    let after_byte = buffer.len();
    huffman_encode(bytes.as_bytes(), buffer)?;
    let after_huffman = buffer.len();
    let len_usize = after_huffman.wrapping_sub(after_byte);
    let fits_in_1_byte = len_usize < 0b0111_1111;
    if let (true, Ok(len)) = (fits_in_1_byte, u8::try_from(len_usize)) {
      let Some(byte) = buffer.get_mut(before_byte) else {
        _unreachable();
      };
      *byte = 0b1000_0000 | len;
    } else if let Ok(len) = u32::try_from(len_usize) {
      let octets = Self::encode_int(buffer, 0b1000_0000, len)?;
      let mut array = [0; 4];
      match (octets, buffer.as_slice()) {
        (2, [.., a, b]) => {
          array[0] = *a;
          array[1] = *b;
        }
        (3, [.., a, b, c]) => {
          array[0] = *a;
          array[1] = *b;
          array[2] = *c;
        }
        (4, [.., a, b, c, d]) => {
          array[0] = *a;
          array[1] = *b;
          array[2] = *c;
          array[3] = *d;
        }
        _ => return Ok(()),
      }
      let _ = shift_copyable_chunks(
        before_byte.wrapping_add(octets.into()),
        buffer,
        iter::once(after_byte..after_huffman),
      );
      buffer.truncate(buffer.len().wrapping_sub(1));
      match (octets, buffer.get_mut(before_byte..)) {
        (2, Some([a, b, ..])) => {
          *a = array[0];
          *b = array[1];
        }
        (3, Some([a, b, c, ..])) => {
          *a = array[0];
          *b = array[1];
          *c = array[2];
        }
        (4, Some([a, b, c, d, ..])) => {
          *a = array[0];
          *b = array[1];
          *c = array[2];
          *d = array[3];
        }
        _ => {}
      }
    } else {
      return Err(protocol_err(Http2Error::UnsupportedHeaderNameOrValueLen));
    }
    Ok(())
  }

  // Regardless of the "sensitive" flag set by users, these headers may carry sensitive content
  // that shouldn't be indexed.
  fn header_is_naturally_sensitive(hhb: HpackHeaderBasic, name: &str) -> bool {
    match hhb {
      HpackHeaderBasic::Field => matches!(
        KnownHeaderName::try_from(name.as_bytes()),
        Ok(
          KnownHeaderName::Age
            | KnownHeaderName::Authorization
            | KnownHeaderName::ContentLength
            | KnownHeaderName::Cookie
            | KnownHeaderName::Etag
            | KnownHeaderName::IfModifiedSince
            | KnownHeaderName::IfNoneMatch
            | KnownHeaderName::Location
            | KnownHeaderName::SetCookie
        )
      ),
      HpackHeaderBasic::Path => true,
      _ => false,
    }
  }

  // Very large headers are not good candidates for indexing.
  fn header_is_very_large(&self, hhb: HpackHeaderBasic, name: &str, value: &str) -> bool {
    let lhs = hhb.len(name, value);
    let rhs = (self.dyn_headers.max_bytes() / 4).wrapping_mul(3);
    lhs >= rhs
  }

  fn idx_to_encode_idx(&self, idx: u32) -> u32 {
    self.idx.wrapping_sub(idx).wrapping_add(DYN_IDX_OFFSET)
  }

  fn manage_encode(
    buffer: &mut Vector<u8>,
    (name, value): (&str, &str),
    idx: EncodeIdx,
  ) -> crate::Result<()> {
    match idx {
      EncodeIdx::RefNameRefValue(local_idx) => {
        let _ = Self::encode_int(buffer, 0b1000_0000, local_idx)?;
      }
      EncodeIdx::RefNameSavedValue(name_idx) => {
        let _ = Self::encode_int(buffer, 0b0100_0000, name_idx)?;
        Self::encode_str(buffer, value)?;
      }
      EncodeIdx::RefNameUnsavedValue(name_idx) => {
        let _ = Self::encode_int(buffer, 0b0001_0000, name_idx)?;
        Self::encode_str(buffer, value)?;
      }
      EncodeIdx::SavedNameSavedValue => {
        buffer.push(0b0100_0000)?;
        Self::encode_str(buffer, name)?;
        Self::encode_str(buffer, value)?;
      }
      EncodeIdx::UnsavedNameUnsavedValue => {
        buffer.push(0b0001_0000)?;
        Self::encode_str(buffer, name)?;
        Self::encode_str(buffer, value)?;
      }
    }
    Ok(())
  }

  fn manage_size_update(&mut self, buffer: &mut Vector<u8>) -> crate::Result<()> {
    match self.max_dyn_sub_bytes.take() {
      Some((lower, None)) => {
        self.dyn_headers.set_max_bytes(*Usize::from(lower), |metadata| {
          Self::remove_outdated_indices(&mut self.indcs, metadata);
        });
        let _ = Self::encode_int(buffer, 0b0010_0000, lower)?;
      }
      Some((lower, Some(upper))) => {
        self.dyn_headers.set_max_bytes(*Usize::from(lower), |metadata| {
          Self::remove_outdated_indices(&mut self.indcs, metadata);
        });
        self.dyn_headers.set_max_bytes(*Usize::from(upper), |metadata| {
          Self::remove_outdated_indices(&mut self.indcs, metadata);
        });
        let _ = Self::encode_int(buffer, 0b0010_0000, lower)?;
        let _ = Self::encode_int(buffer, 0b0010_0000, upper)?;
      }
      None => {}
    }
    Ok(())
  }

  /// Must be called after insertion
  fn next_dyn_idx(&self) -> u32 {
    self.idx.wrapping_sub(1)
  }

  fn push_dyn_headers(
    &mut self,
    (name, value, is_sensitive): (&str, &str, bool),
    (name_hash, pair_hash): (Option<u64>, u64),
  ) -> crate::Result<()> {
    self.idx = self.idx.wrapping_add(1);
    self.dyn_headers.push_front(
      Metadata { name_hash, pair_hash },
      name,
      [value],
      is_sensitive,
      |metadata| {
        Self::remove_outdated_indices(&mut self.indcs, metadata);
      },
    )?;
    Ok(())
  }

  /// Static header index of pseudo header
  fn shi_pseudo((hhb, value): (HpackHeaderBasic, &str)) -> Option<StaticHeader> {
    let (has_value, idx, name): (_, _, &str) = match hhb {
      HpackHeaderBasic::Authority => (false, 1, ":authority"),
      HpackHeaderBasic::Method(method) => {
        let name = ":method";
        let (has_value, idx) = match method {
          Method::Get => (true, 2),
          Method::Post => (true, 3),
          _ => (false, 2),
        };
        (has_value, idx, name)
      }
      HpackHeaderBasic::Path => {
        let name = ":path";
        let (has_value, idx) = match value {
          "/" => (true, 4),
          "/index.html" => (true, 5),
          _ => (false, 4),
        };
        (has_value, idx, name)
      }
      HpackHeaderBasic::Scheme => {
        let name = ":path";
        let (has_value, idx) = match value {
          "http" => (true, 6),
          "https" => (true, 7),
          _ => (false, 6),
        };
        (has_value, idx, name)
      }
      HpackHeaderBasic::StatusCode(status) => {
        let name = ":status";
        let (has_value, idx) = match status {
          StatusCode::Ok => (true, 8),
          StatusCode::NoContent => (true, 9),
          StatusCode::PartialContent => (true, 10),
          StatusCode::NotModified => (true, 11),
          StatusCode::BadRequest => (true, 12),
          StatusCode::NotFound => (true, 13),
          StatusCode::InternalServerError => (true, 14),
          _ => (false, 8),
        };
        (has_value, idx, name)
      }
      HpackHeaderBasic::Field | HpackHeaderBasic::Protocol(_) => return None,
    };
    Some(StaticHeader { has_value, idx, name })
  }

  fn shi_user((name, value): (&str, &str)) -> Option<StaticHeader> {
    let (has_value, idx, local_name) = match KnownHeaderName::try_from(name.as_bytes()) {
      Ok(KnownHeaderName::AcceptCharset) => (false, 15, KnownHeaderName::AcceptCharset.into()),
      Ok(KnownHeaderName::AcceptEncoding) => {
        if value == "gzip, deflate" {
          (true, 16, KnownHeaderName::AcceptEncoding.into())
        } else {
          (false, 16, KnownHeaderName::AcceptEncoding.into())
        }
      }
      Ok(KnownHeaderName::AcceptLanguage) => (false, 17, KnownHeaderName::AcceptLanguage.into()),
      Ok(KnownHeaderName::AcceptRanges) => (false, 18, KnownHeaderName::AcceptRanges.into()),
      Ok(KnownHeaderName::Accept) => (false, 19, KnownHeaderName::Accept.into()),
      Ok(KnownHeaderName::AccessControlAllowOrigin) => {
        (false, 20, KnownHeaderName::AccessControlAllowOrigin.into())
      }
      Ok(KnownHeaderName::Age) => (false, 21, KnownHeaderName::Age.into()),
      Ok(KnownHeaderName::Allow) => (false, 22, KnownHeaderName::Allow.into()),
      Ok(KnownHeaderName::Authorization) => (false, 23, KnownHeaderName::Authorization.into()),
      Ok(KnownHeaderName::CacheControl) => (false, 24, KnownHeaderName::CacheControl.into()),
      Ok(KnownHeaderName::ContentDisposition) => {
        (false, 25, KnownHeaderName::ContentDisposition.into())
      }
      Ok(KnownHeaderName::ContentEncoding) => (false, 26, KnownHeaderName::ContentEncoding.into()),
      Ok(KnownHeaderName::ContentLanguage) => (false, 27, KnownHeaderName::ContentLanguage.into()),
      Ok(KnownHeaderName::ContentLength) => (false, 28, KnownHeaderName::ContentLength.into()),
      Ok(KnownHeaderName::ContentLocation) => (false, 29, KnownHeaderName::ContentLocation.into()),
      Ok(KnownHeaderName::ContentRange) => (false, 30, KnownHeaderName::ContentRange.into()),
      Ok(KnownHeaderName::ContentType) => (false, 31, KnownHeaderName::ContentType.into()),
      Ok(KnownHeaderName::Cookie) => (false, 32, KnownHeaderName::Cookie.into()),
      Ok(KnownHeaderName::Date) => (false, 33, KnownHeaderName::Date.into()),
      Ok(KnownHeaderName::Etag) => (false, 34, KnownHeaderName::Etag.into()),
      Ok(KnownHeaderName::Expect) => (false, 35, KnownHeaderName::Expect.into()),
      Ok(KnownHeaderName::Expires) => (false, 36, KnownHeaderName::Expires.into()),
      Ok(KnownHeaderName::From) => (false, 37, KnownHeaderName::From.into()),
      Ok(KnownHeaderName::Host) => (false, 38, KnownHeaderName::Host.into()),
      Ok(KnownHeaderName::IfMatch) => (false, 39, KnownHeaderName::IfMatch.into()),
      Ok(KnownHeaderName::IfModifiedSince) => (false, 40, KnownHeaderName::IfModifiedSince.into()),
      Ok(KnownHeaderName::IfNoneMatch) => (false, 41, KnownHeaderName::IfNoneMatch.into()),
      Ok(KnownHeaderName::IfRange) => (false, 42, KnownHeaderName::IfRange.into()),
      Ok(KnownHeaderName::IfUnmodifiedSince) => {
        (false, 43, KnownHeaderName::IfUnmodifiedSince.into())
      }
      Ok(KnownHeaderName::LastModified) => (false, 44, KnownHeaderName::LastModified.into()),
      Ok(KnownHeaderName::Link) => (false, 45, KnownHeaderName::Link.into()),
      Ok(KnownHeaderName::Location) => (false, 46, KnownHeaderName::Location.into()),
      Ok(KnownHeaderName::MaxForwards) => (false, 47, KnownHeaderName::MaxForwards.into()),
      Ok(KnownHeaderName::ProxyAuthenticate) => {
        (false, 48, KnownHeaderName::ProxyAuthenticate.into())
      }
      Ok(KnownHeaderName::ProxyAuthorization) => {
        (false, 49, KnownHeaderName::ProxyAuthorization.into())
      }
      Ok(KnownHeaderName::Range) => (false, 50, KnownHeaderName::Range.into()),
      Ok(KnownHeaderName::Referer) => (false, 51, KnownHeaderName::Referer.into()),
      Ok(KnownHeaderName::Refresh) => (false, 52, KnownHeaderName::Refresh.into()),
      Ok(KnownHeaderName::RetryAfter) => (false, 53, KnownHeaderName::RetryAfter.into()),
      Ok(KnownHeaderName::Server) => (false, 54, KnownHeaderName::Server.into()),
      Ok(KnownHeaderName::SetCookie) => (false, 55, KnownHeaderName::SetCookie.into()),
      Ok(KnownHeaderName::StrictTransportSecurity) => {
        (false, 56, KnownHeaderName::StrictTransportSecurity.into())
      }
      Ok(KnownHeaderName::TransferEncoding) => {
        (false, 57, KnownHeaderName::TransferEncoding.into())
      }
      Ok(KnownHeaderName::UserAgent) => (false, 58, KnownHeaderName::UserAgent.into()),
      Ok(KnownHeaderName::Vary) => (false, 59, KnownHeaderName::Vary.into()),
      Ok(KnownHeaderName::Via) => (false, 60, KnownHeaderName::Via.into()),
      Ok(KnownHeaderName::WwwAuthenticate) => (false, 61, KnownHeaderName::WwwAuthenticate.into()),
      _ => return None,
    };
    Some(StaticHeader { has_value, idx, name: local_name })
  }

  fn should_not_index(
    &self,
    (name, value, is_sensitive): (&str, &str, bool),
    hhb: HpackHeaderBasic,
  ) -> bool {
    is_sensitive
      || Self::header_is_naturally_sensitive(hhb, name)
      || self.header_is_very_large(hhb, name, value)
  }

  fn remove_outdated_indices(indcs: &mut HashMap<u64, u32>, metadata: Metadata) {
    if let Some(elem) = metadata.name_hash {
      let _ = indcs.remove(&elem);
    }
    let _ = indcs.remove(&metadata.pair_hash);
  }

  fn store_header_with_ref_name<const HAS_STATIC_NAME: bool>(
    &mut self,
    (name, value, is_sensitive): (&str, &str, bool),
    name_idx: u32,
    pair_hash: u64,
  ) -> crate::Result<EncodeIdx> {
    let before = self.dyn_headers.headers_len();
    self.push_dyn_headers((name, value, is_sensitive), (None, pair_hash))?;
    let _ = self.indcs.insert(pair_hash, self.next_dyn_idx());
    if !HAS_STATIC_NAME {
      let after = self.dyn_headers.headers_len();
      let diff = before.wrapping_sub(after.wrapping_sub(1));
      let name_idx_has_been_removed = diff > *Usize::from(name_idx);
      if name_idx_has_been_removed {
        return Ok(EncodeIdx::SavedNameSavedValue);
      }
    }
    Ok(EncodeIdx::RefNameSavedValue(name_idx))
  }
}

/// <https://datatracker.ietf.org/doc/html/rfc7541#section-6.2>
///
/// Elements already stored (Ref) are encoded using their referenced indexes. Elements that were
/// recently stored (Saved) or must not be stored (Unsaved) are encoded using their literal
/// contents (Literal).
#[derive(Clone, Copy, Debug)]
enum EncodeIdx {
  /// Both elements are already stored and the common referenced index is used for encoding.
  RefNameRefValue(u32),
  /// The name is already stored and the referenced index is used for encoding. The value has been
  /// stored and the literal contents are used for encoding.
  RefNameSavedValue(u32),
  /// Both "Never Indexed" and "Without Indexing" variants.
  ///
  /// The name is already stored and the referenced index is used for encoding. The value is not stored
  /// and the literal contents are used for encoding.
  RefNameUnsavedValue(u32),
  /// The name has been stored and the literal contents are used for encoding. The value has been
  /// stored the literal contents are used for encoding.
  SavedNameSavedValue,
  /// Both "Never Indexed" and "Without Indexing" variants.
  ///
  /// The name is not stored and the literal contents are used for encoding. The value is not stored
  /// and the literal contents are used for encoding.
  UnsavedNameUnsavedValue,
}

#[derive(Clone, Copy, Debug)]
struct Metadata {
  name_hash: Option<u64>,
  pair_hash: u64,
}

#[derive(Clone, Copy, Debug)]
struct StaticHeader {
  has_value: bool,
  idx: u32,
  name: &'static str,
}

#[cfg(test)]
mod tests {
  use crate::{
    collection::Vector,
    http::{Method, StatusCode},
    http2::{
      hpack_encoder::HpackEncoder, hpack_header::HpackHeaderBasic,
      hpack_static_headers::HpackStaticResponseHeaders,
    },
    rng::{Xorshift64, simple_seed},
  };

  #[test]
  fn duplicated_is_indexed() {
    let headers = [(HpackHeaderBasic::Method(Method::Patch), Method::Patch.strings().custom[0])];
    let mut buffer = Vector::new();
    let mut hpack_enc = HpackEncoder::new(&mut Xorshift64::from(simple_seed()));
    hpack_enc.dyn_headers.set_max_bytes(4096, |_| {});
    hpack_enc.encode(&mut buffer, headers, []).unwrap();
    assert_eq!(buffer[0], 66);
    assert_eq!(buffer[1], 133);
    buffer.clear();
    hpack_enc.encode(&mut buffer, headers, []).unwrap();
    assert_eq!(buffer[0], 190);
    assert_eq!(buffer.len(), 1);
  }

  #[test]
  fn encodes_status_code() {
    let mut buffer = Vector::new();
    let mut hpack_enc = HpackEncoder::new(&mut Xorshift64::from(simple_seed()));
    hpack_enc
      .encode(
        &mut buffer,
        HpackStaticResponseHeaders { status_code: Some(StatusCode::Unauthorized) }.iter(),
        [],
      )
      .unwrap();
    assert_eq!(buffer.as_slice(), &[24, 130, 104, 1]);
  }

  #[test]
  fn encodes_methods_that_are_not_get_or_post() {
    let mut buffer = Vector::new();
    let mut hpack_enc = HpackEncoder::new(&mut Xorshift64::from(simple_seed()));
    hpack_enc.dyn_headers.set_max_bytes(4096, |_| {});
    hpack_enc
      .encode(
        &mut buffer,
        [(HpackHeaderBasic::Method(Method::Delete), Method::Delete.strings().custom[0])],
        [],
      )
      .unwrap();
    assert_eq!(&buffer, &[66, 134, 191, 131, 62, 13, 248, 63][..])
  }
}
