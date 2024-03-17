use alloc::{string::String, vec::Vec};
use core::{fmt::Formatter, marker::PhantomData};
use serde::{
  de::{Deserializer, MapAccess, Visitor},
  Deserialize,
};
use std::{
  fs::{read_dir, File},
  io::Read,
  path::Path,
  process::Command,
};

use crate::{
  http2::{HpackDecoder, HpackEncoder, HpackHeaderBasic, DEFAULT_MAX_COMPRESSED_HEADER_LEN},
  misc::{from_utf8_basic, ByteVector},
  rng::StaticRng,
};

#[test]
fn hpack_test_case() {
  fetch_hpack_test_case();
  let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("hpack-test-case");
  for root_entry_rslt in read_dir(root).unwrap() {
    let root_entry = root_entry_rslt.unwrap();
    if root_entry.file_type().unwrap().is_dir() {
      let root_entry_path = root_entry.path();
      for impl_entry_rslt in read_dir(&root_entry_path).unwrap() {
        let impl_entry = impl_entry_rslt.unwrap();
        if impl_entry.file_name().as_os_str().to_str().unwrap().starts_with("story_") {
          test_case(&root_entry_path, &impl_entry.path());
        }
      }
    }
  }
}

#[derive(Debug, serde::Deserialize)]
struct Case {
  header_table_size: Option<u16>,
  headers: Vec<CaseHeader>,
  seqno: u16,
  wire: String,
}

#[derive(Clone, Debug)]
struct CaseHeader {
  name: String,
  value: String,
}

#[derive(Debug, serde::Deserialize)]
struct Root {
  cases: Vec<Case>,
}

impl<'de> Deserialize<'de> for CaseHeader {
  fn deserialize<D>(deserializer: D) -> Result<CaseHeader, D::Error>
  where
    D: Deserializer<'de>,
  {
    struct CustomVisitor<'de>(PhantomData<&'de ()>);

    impl<'de> Visitor<'de> for CustomVisitor<'de> {
      type Value = CaseHeader;

      fn expecting(&self, formatter: &mut Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("struct CaseHeader")
      }

      fn visit_map<V>(self, mut map: V) -> Result<CaseHeader, V::Error>
      where
        V: MapAccess<'de>,
      {
        let mut key: Option<&str> = None;
        let mut value: Option<&str> = None;
        if let Some(local_key) = map.next_key()? {
          key = Some(local_key);
          value = Some(map.next_value()?);
        }
        Ok(CaseHeader {
          name: key.map(|el| el.into()).ok_or_else(|| serde::de::Error::missing_field("key"))?,
          value: value
            .map(|el| el.into())
            .ok_or_else(|| serde::de::Error::missing_field("value"))?,
        })
      }
    }

    deserializer.deserialize_struct("CaseHeader", &["key", "value"], CustomVisitor(PhantomData))
  }
}

fn fetch_hpack_test_case() {
  let _output = Command::new("git")
    .arg("clone")
    .arg("https://github.com/http2jp/hpack-test-case")
    .output()
    .unwrap();
}

fn parse_hex(hex: &[u8]) -> Vec<u8> {
  let mut hex_bytes = hex
    .iter()
    .filter_map(|b| match b {
      b'0'..=b'9' => Some(b - b'0'),
      b'a'..=b'f' => Some(b - b'a' + 10),
      b'A'..=b'F' => Some(b - b'A' + 10),
      _ => None,
    })
    .fuse();
  let mut bytes = Vec::new();
  while let (Some(h), Some(l)) = (hex_bytes.next(), hex_bytes.next()) {
    bytes.push(h << 4 | l)
  }
  bytes
}

fn print_case(seqno: u16, dir_path: &Path, case_path: &Path) {
  std::println!(
    "***** Testing case {:?} of story {:?} of implementation {:?} *****",
    seqno,
    case_path.file_name().unwrap(),
    dir_path.file_name().unwrap(),
  );
}

fn strs<'key, 'value>(
  hhb: HpackHeaderBasic,
  name: &'key [u8],
  value: &'value [u8],
) -> (&'key str, &'value str) {
  match hhb {
    HpackHeaderBasic::Authority => (":authority", from_utf8_basic(value).unwrap()),
    HpackHeaderBasic::Field => (from_utf8_basic(name).unwrap(), from_utf8_basic(value).unwrap()),
    HpackHeaderBasic::Method(elem) => (":method", elem.strings().custom),
    HpackHeaderBasic::Path => (":path", from_utf8_basic(value).unwrap()),
    HpackHeaderBasic::Protocol(elem) => (":protocol", elem.strings().custom),
    HpackHeaderBasic::Scheme => (":scheme", from_utf8_basic(value).unwrap()),
    HpackHeaderBasic::Status(elem) => (":status", elem.strings().custom),
  }
}

fn test_case(dir_path: &Path, case_path: &Path) {
  let mut file = File::open(case_path).unwrap();
  let mut data = String::new();
  file.read_to_string(&mut data).unwrap();
  let root: Root = serde_json::from_str(&data).unwrap();

  let mut cases = root.cases;
  cases.sort_unstable_by_key(|case| case.seqno);

  let mut decoder = HpackDecoder::with_capacity(0, 0, DEFAULT_MAX_COMPRESSED_HEADER_LEN);
  let mut encoder =
    HpackEncoder::with_capacity(0, 0, DEFAULT_MAX_COMPRESSED_HEADER_LEN, StaticRng::default());

  for case in &cases {
    print_case(case.seqno, dir_path, case_path);
    let mut buffer = ByteVector::with_capacity(0);
    if let Some(size) = case.header_table_size {
      encoder.set_max_dyn_sub_bytes(size).unwrap();
      decoder.set_max_dyn_sub_bytes(size).unwrap();
    }
    let mut pseudo_headers = case
      .headers
      .iter()
      .filter_map(|header| {
        Some(match header.name.as_str() {
          ":authority" => (HpackHeaderBasic::Authority, header.value.as_bytes()),
          ":method" => {
            (HpackHeaderBasic::Method(header.value.as_str().try_into().unwrap()), &[][..])
          }
          ":path" => (HpackHeaderBasic::Path, header.value.as_bytes()),
          ":protocol" => {
            (HpackHeaderBasic::Protocol(header.value.as_str().try_into().unwrap()), &[][..])
          }
          ":scheme" => (HpackHeaderBasic::Scheme, header.value.as_bytes()),
          ":status" => {
            (HpackHeaderBasic::Status(header.value.as_str().try_into().unwrap()), &[][..])
          }
          _ => return None,
        })
      })
      .collect::<Vec<_>>();
    let mut user_headers = case
      .headers
      .iter()
      .filter_map(|header| {
        if header.name.starts_with(":") {
          None
        } else {
          Some((HpackHeaderBasic::Field, header.name.as_bytes(), header.value.as_bytes(), false))
        }
      })
      .collect::<Vec<_>>();
    encoder
      .encode(
        &mut buffer,
        pseudo_headers.iter().copied(),
        user_headers.iter().map(|el| (el.1, el.2, el.3)),
      )
      .unwrap();
    decoder
      .decode(&buffer, |(hhb, name, value)| {
        if let HpackHeaderBasic::Field = hhb {
          assert_eq!((hhb, name, value, false), user_headers.remove(0));
        } else {
          assert_eq!((hhb, value), pseudo_headers.remove(0));
        }
      })
      .unwrap();
    assert_eq!(0, pseudo_headers.len());
    assert_eq!(0, user_headers.len());
  }

  decoder._clear();

  for mut case in cases {
    print_case(case.seqno, dir_path, case_path);
    if let Some(size) = case.header_table_size {
      decoder.set_max_dyn_sub_bytes(size).unwrap();
    }
    decoder
      .decode(&parse_hex(case.wire.as_bytes()), |(hhb, name, value)| {
        let case_header = case.headers.remove(0);
        let (name, value) = strs(hhb, name, value);
        assert_eq!(case_header.name, name);
        assert_eq!(case_header.value, value);
      })
      .unwrap();
    assert_eq!(0, case.headers.len());
  }
}
