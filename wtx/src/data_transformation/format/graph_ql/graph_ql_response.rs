use crate::{data_transformation::format::GraphQlResponseError, misc::Vector};

/// Replied from an issued [`crate::data_transformation::format::GraphQlRequest`].
#[derive(Debug)]
pub struct GraphQlResponse<D, E> {
  /// Content depends if request was successful or not.
  pub result: Result<D, Vector<GraphQlResponseError<E>>>,
}

#[cfg(feature = "serde")]
mod serde {
  use crate::{
    data_transformation::format::{GraphQlResponse, GraphQlResponseError},
    misc::Vector,
  };
  use core::marker::PhantomData;
  use serde::{de::Visitor, ser::SerializeStruct};

  impl<'de, D, E> serde::Deserialize<'de> for GraphQlResponse<D, E>
  where
    D: serde::Deserialize<'de>,
    E: serde::Deserialize<'de>,
  {
    #[inline]
    fn deserialize<DE>(deserializer: DE) -> Result<GraphQlResponse<D, E>, DE::Error>
    where
      DE: serde::de::Deserializer<'de>,
    {
      struct CustomVisitor<'de, D, E>(PhantomData<(D, E)>, PhantomData<&'de ()>)
      where
        D: serde::Deserialize<'de>,
        E: serde::Deserialize<'de>;

      impl<'de, D, E> Visitor<'de> for CustomVisitor<'de, D, E>
      where
        D: serde::Deserialize<'de>,
        E: serde::Deserialize<'de>,
      {
        type Value = GraphQlResponse<D, E>;

        fn expecting(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
          formatter.write_str("struct GraphQlResponse")
        }

        #[inline]
        fn visit_map<V>(self, mut map: V) -> Result<GraphQlResponse<D, E>, V::Error>
        where
          V: serde::de::MapAccess<'de>,
        {
          let mut data = None;
          let mut errors = None;

          while let Some(key) = map.next_key()? {
            match key {
              Field::Data => {
                if data.is_some() {
                  return Err(serde::de::Error::duplicate_field("data"));
                }
                data = Some(map.next_value()?);
              }
              Field::Errors => {
                if errors.is_some() {
                  return Err(serde::de::Error::duplicate_field("errors"));
                }
                errors = Some(map.next_value::<Vector<GraphQlResponseError<E>>>()?);
              }
            }
          }

          Ok(GraphQlResponse {
            result: if let Some(elem) = errors {
              Err(elem)
            } else {
              Ok(data.ok_or_else(|| serde::de::Error::missing_field("data"))?)
            },
          })
        }
      }

      const FIELDS: &[&str] = &["data", "errors"];
      deserializer.deserialize_struct(
        "GraphQlResponse",
        FIELDS,
        CustomVisitor(PhantomData, PhantomData),
      )
    }
  }

  impl<D, E> serde::Serialize for GraphQlResponse<D, E>
  where
    D: serde::Serialize,
    E: serde::Serialize,
  {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
      S: serde::ser::Serializer,
    {
      let mut state = serializer.serialize_struct("GraphQlResponse", 1)?;
      match self.result {
        Err(ref err) => {
          state.serialize_field("errors", err)?;
        }
        Ok(ref el) => state.serialize_field("data", &el)?,
      }
      state.end()
    }
  }

  #[derive(Debug, serde::Deserialize)]
  #[serde(field_identifier, rename_all = "lowercase")]
  enum Field {
    Data,
    Errors,
  }
}

#[cfg(feature = "serde_json")]
mod serde_json {
  use crate::{
    data_transformation::{
      dnsn::SerdeJson,
      format::{misc::collect_using_serde_json, GraphQlResponse},
    },
    misc::Vector,
  };
  use serde::{Deserialize, Serialize};

  impl<'de, D, E> crate::data_transformation::dnsn::Deserialize<'de, SerdeJson>
    for GraphQlResponse<D, E>
  where
    D: Deserialize<'de>,
    E: Deserialize<'de>,
  {
    #[inline]
    fn from_bytes(bytes: &'de [u8], _: &mut SerdeJson) -> crate::Result<Self> {
      Ok(serde_json::from_slice(bytes)?)
    }

    #[inline]
    fn seq_from_bytes(
      buffer: &mut Vector<Self>,
      bytes: &'de [u8],
      _: &mut SerdeJson,
    ) -> crate::Result<()> {
      collect_using_serde_json(buffer, bytes)
    }
  }

  impl<D, E> crate::data_transformation::dnsn::Serialize<SerdeJson> for GraphQlResponse<D, E>
  where
    D: Serialize,
    E: Serialize,
  {
    #[inline]
    fn to_bytes(&mut self, bytes: &mut Vector<u8>, _: &mut SerdeJson) -> crate::Result<()> {
      if size_of::<Self>() == 0 {
        return Ok(());
      }
      serde_json::to_writer(bytes, &self.result)?;
      Ok(())
    }
  }
}
