use crate::{
  collection::{IndexedStorageMut, Vector},
  http::{Header, StatusCode},
  http2::{
    MAX_HPACK_LEN, hpack_decoder::HpackDecoder, hpack_encoder::HpackEncoder,
    hpack_header::HpackHeaderBasic,
  },
  rng::{Xorshift64, simple_seed},
};
use alloc::string::String;
use core::{fmt::Formatter, marker::PhantomData};
use serde::{
  Deserialize,
  de::{Deserializer, MapAccess, Visitor},
};
use std::{
  fs::{File, read_dir},
  io::Read,
  path::Path,
  process::Command,
};

// Looks like some `wire` contents were stored assuming 16384 bytes.
const MAX_HEADER_LEN: u32 = 16384;

#[test]
fn hpack_test_cases() {
  fetch_project();
  let mut buffer = Vector::new();
  let mut decoder = HpackDecoder::new();
  let mut encoder = HpackEncoder::new(&mut Xorshift64::from(simple_seed()));
  decoder.set_max_bytes(MAX_HEADER_LEN);
  encoder.set_max_dyn_super_bytes(MAX_HEADER_LEN);
  let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("hpack-test-case");
  for impl_entry_rslt in read_dir(root).unwrap() {
    let impl_entry = impl_entry_rslt.unwrap();
    if impl_entry.file_type().unwrap().is_dir() {
      let impl_entry_path = impl_entry.path();
      for story_entry_rslt in read_dir(&impl_entry_path).unwrap() {
        let story_entry = story_entry_rslt.unwrap();
        if story_entry.file_name().as_os_str().to_str().unwrap().starts_with("story_") {
          test_story(
            &mut buffer,
            (&impl_entry_path, &story_entry.path()),
            (&mut decoder, &mut encoder),
          );
        }
      }
    }
  }
}

#[derive(Debug, serde::Deserialize)]
struct Case {
  header_table_size: Option<u32>,
  headers: Vector<CaseHeader>,
  seqno: Option<u16>,
  wire: Option<String>,
}

#[derive(Clone, Debug)]
struct CaseHeader {
  name: String,
  value: String,
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
        let mut key: Option<String> = None;
        let mut value: Option<String> = None;
        if let Some(local_key) = map.next_key()? {
          key = Some(local_key);
          value = Some(map.next_value()?);
        }
        Ok(CaseHeader {
          name: key.ok_or_else(|| serde::de::Error::missing_field("key"))?,
          value: value.ok_or_else(|| serde::de::Error::missing_field("value"))?,
        })
      }
    }

    deserializer.deserialize_struct("CaseHeader", &["key", "value"], CustomVisitor(PhantomData))
  }
}

#[derive(Debug, serde::Deserialize)]
struct Root {
  cases: Vector<Case>,
}

fn fetch_project() {
  let _output = Command::new("git")
    .arg("clone")
    .arg("https://github.com/http2jp/hpack-test-case")
    .output()
    .unwrap();
}

pub(crate) const fn hhb_name(hhb: HpackHeaderBasic, name: &str) -> &str {
  match hhb {
    HpackHeaderBasic::Authority => ":authority",
    HpackHeaderBasic::Field => name,
    HpackHeaderBasic::Method(_) => ":method",
    HpackHeaderBasic::Path => ":path",
    HpackHeaderBasic::Protocol(_) => ":protocol",
    HpackHeaderBasic::Scheme => ":scheme",
    HpackHeaderBasic::StatusCode(_) => ":status",
  }
}

fn parse_hex(hex: &[u8]) -> Vector<u8> {
  let mut hex_bytes = hex
    .iter()
    .filter_map(|b| match b {
      b'0'..=b'9' => Some(b - b'0'),
      b'a'..=b'f' => Some(b - b'a' + 10),
      b'A'..=b'F' => Some(b - b'A' + 10),
      _ => None,
    })
    .fuse();
  let mut bytes = Vector::new();
  while let (Some(h), Some(l)) = (hex_bytes.next(), hex_bytes.next()) {
    bytes.push(h << 4 | l).unwrap();
  }
  bytes
}

fn strs<'key, 'value>(
  hhb: HpackHeaderBasic,
  name: &'key str,
  value: &'value str,
) -> (&'key str, &'value str) {
  match hhb {
    HpackHeaderBasic::Authority => (":authority", value),
    HpackHeaderBasic::Field => (name, value),
    HpackHeaderBasic::Method(elem) => (":method", elem.strings().custom[0]),
    HpackHeaderBasic::Path => (":path", value),
    HpackHeaderBasic::Protocol(elem) => (":protocol", elem.strings().custom[0]),
    HpackHeaderBasic::Scheme => (":scheme", value),
    HpackHeaderBasic::StatusCode(elem) => (":status", elem.strings().number),
  }
}

fn test_story(
  buffer: &mut Vector<u8>,
  (_impl_path, story_path): (&Path, &Path),
  (decoder, encoder): (&mut HpackDecoder, &mut HpackEncoder),
) {
  let mut file = File::open(story_path).unwrap();
  let mut data = String::new();
  let _ = file.read_to_string(&mut data).unwrap();
  let root: Root = serde_json::from_str(&data).unwrap();

  let mut cases = root.cases;
  cases.sort_unstable_by_key(|case| case.seqno);

  test_story_encoding_and_decoding(buffer, &cases, (decoder, encoder));

  decoder.clear();

  test_story_wired_decoding(&mut cases, decoder);

  buffer.clear();
  decoder.clear();
  encoder.clear();
}

fn test_story_encoding_and_decoding(
  buffer: &mut Vector<u8>,
  cases: &[Case],
  (decoder, encoder): (&mut HpackDecoder, &mut HpackEncoder),
) {
  for case in cases {
    if let Some(size) = case.header_table_size {
      decoder.set_max_bytes(size);
      encoder.set_max_dyn_sub_bytes(size).unwrap();
    } else {
      decoder.set_max_bytes(MAX_HPACK_LEN);
      encoder.set_max_dyn_sub_bytes(MAX_HPACK_LEN).unwrap();
    }

    let mut pseudo_headers = Vector::from_iter(case.headers.iter().filter_map(|header| {
      Some(match header.name.as_str() {
        ":authority" => (HpackHeaderBasic::Authority, header.value.as_str()),
        ":method" => {
          let method = header.value.as_str().try_into().unwrap();
          (HpackHeaderBasic::Method(method), method.strings().custom[0])
        }
        ":path" => (HpackHeaderBasic::Path, header.value.as_str()),
        ":protocol" => {
          let protocol = header.value.as_str().try_into().unwrap();
          (HpackHeaderBasic::Protocol(protocol), protocol.strings().custom[0])
        }
        ":scheme" => (HpackHeaderBasic::Scheme, header.value.as_str()),
        ":status" => {
          let status: StatusCode = header.value.as_str().try_into().unwrap();
          (HpackHeaderBasic::StatusCode(status), status.strings().number)
        }
        _ => return None,
      })
    }))
    .unwrap();

    let mut user_headers = Vector::from_iter(case.headers.iter().filter_map(|header| {
      if header.name.starts_with(":") {
        None
      } else {
        Some((HpackHeaderBasic::Field, header.name.as_str(), header.value.as_str()))
      }
    }))
    .unwrap();

    encoder
      .encode(
        buffer,
        pseudo_headers.iter().copied(),
        user_headers.iter().map(|el| Header::from_name_and_value(el.1, el.2)),
      )
      .unwrap();

    decoder
      .decode(buffer, |(hhb, name, value)| {
        if pseudo_headers.is_empty() {
          assert_eq!((hhb, hhb_name(hhb, name.str()), value), user_headers.remove(0).unwrap());
        } else {
          assert_eq!((hhb, value), pseudo_headers.remove(0).unwrap());
        }
        Ok(())
      })
      .unwrap();

    buffer.clear();
    assert_eq!(0, pseudo_headers.len());
    assert_eq!(0, user_headers.len());
  }
}

fn test_story_wired_decoding(cases: &mut Vector<Case>, decoder: &mut HpackDecoder) {
  for case in cases.iter_mut() {
    if let Some(elem) = case.header_table_size {
      decoder.set_max_bytes(elem);
    }

    let Some(wire) = &case.wire else {
      continue;
    };

    decoder
      .decode(&parse_hex(wire.as_bytes()), |(hhb, name, value)| {
        let case_header = case.headers.remove(0).unwrap();
        let (name, value) = strs(hhb, name.str(), value);
        assert_eq!(case_header.name, name);
        assert_eq!(case_header.value, value);
        Ok(())
      })
      .unwrap();
    assert_eq!(0, case.headers.len());
  }
}
