#[cfg(feature = "serde_json")]
pub(crate) use serde_json::collect_using_serde_json;

#[cfg(feature = "serde_json")]
mod serde_json {
  use crate::{collection::Vector, misc::deserialize_seq_into_buffer_with_serde};
  use serde::Deserialize;

  pub(crate) fn collect_using_serde_json<'de, T>(
    buffer: &mut Vector<T>,
    bytes: &mut &'de [u8],
  ) -> crate::Result<()>
  where
    T: Deserialize<'de>,
  {
    deserialize_seq_into_buffer_with_serde(&mut serde_json::Deserializer::from_slice(bytes), buffer)
  }

  #[cfg(test)]
  mod tests {
    use crate::{collection::Vector, de::protocol::misc::serde_json::collect_using_serde_json};

    #[derive(Debug, PartialEq, serde::Deserialize)]
    struct Foo {
      a: u8,
      b: u64,
    }

    #[test]
    fn array_is_deserialized() {
      let json = r#"[{"a":1,"b":90},{"a":7,"b":567}]"#;
      let mut vector = Vector::<Foo>::new();
      collect_using_serde_json(&mut vector, &mut json.as_bytes()).unwrap();
      assert_eq!(vector.as_slice(), &[Foo { a: 1, b: 90 }, Foo { a: 7, b: 567 }]);
    }
  }
}
