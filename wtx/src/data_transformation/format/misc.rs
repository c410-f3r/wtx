#[cfg(feature = "serde_json")]
pub(crate) use serde_json::collect_using_serde_json;

#[cfg(feature = "serde_json")]
mod serde_json {
  use crate::misc::Vector;
  use core::{any::type_name, fmt::Formatter};
  use serde::{
    de::{Deserializer, Error, SeqAccess, Visitor},
    Deserialize,
  };

  pub(crate) fn collect_using_serde_json<'de, T>(
    buffer: &mut Vector<T>,
    bytes: &'de [u8],
  ) -> crate::Result<()>
  where
    T: Deserialize<'de>,
  {
    struct Buffer<'any, T>(&'any mut Vector<T>);

    impl<'de, T> Visitor<'de> for Buffer<'_, T>
    where
      T: Deserialize<'de>,
    {
      type Value = ();

      #[inline]
      fn expecting(&self, formatter: &mut Formatter<'_>) -> core::fmt::Result {
        formatter.write_fmt(format_args!("a sequence of `{}`", type_name::<T>()))
      }

      #[inline]
      fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
      where
        A: SeqAccess<'de>,
      {
        if let Some(elem) = seq.size_hint() {
          self.0.reserve(elem).map_err(A::Error::custom)?;
        }
        while let Some(elem) = seq.next_element()? {
          self.0.push(elem).map_err(A::Error::custom)?;
        }
        Ok(())
      }
    }

    serde_json::Deserializer::from_slice(bytes).deserialize_seq(Buffer(buffer))?;
    Ok(())
  }

  #[cfg(test)]
  mod tests {
    use crate::{
      data_transformation::format::misc::serde_json::collect_using_serde_json, misc::Vector,
    };

    #[derive(Debug, PartialEq, serde::Deserialize)]
    struct Foo {
      a: u8,
      b: u64,
    }

    #[test]
    fn array_is_deserialized() {
      let json = r#"[{"a":1,"b":90},{"a":7,"b":567}]"#;
      let mut vector = Vector::<Foo>::new();
      collect_using_serde_json(&mut vector, json.as_bytes()).unwrap();
      assert_eq!(vector.as_slice(), &[Foo { a: 1, b: 90 }, Foo { a: 7, b: 567 }]);
    }
  }
}
