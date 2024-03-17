use alloc::{string::String, vec::Vec};
use core::{fmt::Formatter, marker::PhantomData};
use serde::{
  de::{Deserializer, MapAccess, Visitor},
  Deserialize,
};
use std::{
  fs::{read_dir, File},
  io::Read,
  path::{Path, PathBuf},
  process::Command,
};

use crate::http2::HpackDecoder;

#[test]
fn hpack_test_case() {
  fetch_hpack_test_case();
  let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("hpack-test-case");
  for root_entry_rslt in read_dir(root).unwrap() {
    let root_entry = root_entry_rslt.unwrap();
    if root_entry.file_type().unwrap().is_dir() {
      for impl_entry_rslt in read_dir(root_entry.path()).unwrap() {
        let impl_entry = impl_entry_rslt.unwrap();
        if impl_entry.file_name().as_os_str().to_str().unwrap().starts_with("story_") {
          test_case(impl_entry.path());
        }
      }
    }
  }
}

#[derive(Debug, serde::Deserialize)]
struct Case {
  header_table_size: Option<usize>,
  headers: Vec<CaseHeader>,
  seqno: u64,
  wire: Vec<u8>,
}

#[derive(Debug)]
struct CaseHeader {
  key: String,
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

      #[inline]
      fn expecting(&self, formatter: &mut Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("struct CaseHeader")
      }

      #[inline]
      fn visit_map<V>(self, mut map: V) -> Result<CaseHeader, V::Error>
      where
        V: MapAccess<'de>,
      {
        let mut key: Option<&str> = None;
        let mut value: Option<&str> = None;
        if let Some(key) = map.next_key()? {
          value = Some(map.next_value()?);
        }
        Ok(CaseHeader {
          key: key.map(|el| el.into()).ok_or_else(|| serde::de::Error::missing_field("key"))?,
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

fn test_case(path: PathBuf) {
  let mut file = File::open(path).unwrap();
  let mut data = String::new();
  file.read_to_string(&mut data).unwrap();
  let cases: Vec<Case> = serde_json::from_str(&data).unwrap();

  let mut decoder = HpackDecoder::with_capacity(0, 0, 0);

  for case in &cases {
    let mut expect = case.expect.clone();

    if let Some(size) = case.header_table_size {
      decoder.queue_size_update(size);
    }

    let mut buf = BytesMut::with_capacity(case.wire.len());
    buf.extend_from_slice(&case.wire);
    decoder
      .decode(&mut Cursor::new(&mut buf), |e| {
        let (name, value) = expect.remove(0);
        assert_eq!(name, key_str(&e));
        assert_eq!(value, value_str(&e));
      })
      .unwrap();

    assert_eq!(0, expect.len());
  }

  let mut encoder = Encoder::default();
  let mut decoder = Decoder::default();

  // Now, encode the headers
  for case in &cases {
    let limit = 64 * 1024;
    let mut buf = BytesMut::with_capacity(limit);

    if let Some(size) = case.header_table_size {
      encoder.update_max_size(size);
      decoder.queue_size_update(size);
    }

    let mut input: Vec<_> = case
      .expect
      .iter()
      .map(|(name, value)| Header::new(name.clone().into(), value.clone().into()).unwrap().into())
      .collect();

    encoder.encode(&mut input.clone().into_iter(), &mut buf);

    decoder
      .decode(&mut Cursor::new(&mut buf), |e| {
        assert_eq!(e, input.remove(0).reify().unwrap());
      })
      .unwrap();

    assert_eq!(0, input.len());
  }
}
