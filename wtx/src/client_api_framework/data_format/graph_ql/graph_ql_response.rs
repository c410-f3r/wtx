use crate::client_api_framework::data_format::GraphQlResponseError;
use alloc::vec::Vec;

/// Replied from an issued [crate::data_format::GraphQlRequest].
#[derive(Debug)]
pub struct GraphQlResponse<D, E> {
  /// Content depends if request was successful or not.
  pub result: Result<D, Vec<GraphQlResponseError<E>>>,
}

#[cfg(feature = "serde")]
mod serde {
  use crate::client_api_framework::data_format::{GraphQlResponse, GraphQlResponseError};
  use alloc::vec::Vec;
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
                errors = Some(map.next_value::<Vec<GraphQlResponseError<E>>>()?);
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
  use crate::client_api_framework::{
    data_format::GraphQlResponse, dnsn::SerdeJson, misc::seq_visitor::_SeqVisitor,
  };
  use core::fmt::Display;
  use serde::de::Deserializer;

  impl<D, E> crate::client_api_framework::dnsn::Deserialize<SerdeJson> for GraphQlResponse<D, E>
  where
    D: for<'de> serde::Deserialize<'de>,
    E: for<'de> serde::Deserialize<'de>,
  {
    #[inline]
    fn from_bytes(bytes: &[u8], _: &mut SerdeJson) -> crate::Result<Self> {
      Ok(serde_json::from_slice(bytes)?)
    }

    #[inline]
    fn seq_from_bytes<ERR>(
      bytes: &[u8],
      _: &mut SerdeJson,
      cb: impl FnMut(Self) -> Result<(), ERR>,
    ) -> Result<(), ERR>
    where
      ERR: Display + From<crate::Error>,
    {
      let mut de = serde_json::Deserializer::from_slice(bytes);
      de.deserialize_seq(_SeqVisitor::_new(cb)).map_err(Into::into)?;
      Ok(())
    }
  }

  impl<D, E> crate::client_api_framework::dnsn::Serialize<SerdeJson> for GraphQlResponse<D, E>
  where
    D: serde::Serialize,
    E: serde::Serialize,
  {
    #[inline]
    fn to_bytes(&mut self, bytes: &mut Vec<u8>, _: &mut SerdeJson) -> crate::Result<()> {
      if core::mem::size_of::<Self>() == 0 {
        return Ok(());
      }
      serde_json::to_writer(bytes, &self.result)?;
      Ok(())
    }
  }
}

#[cfg(feature = "simd-json")]
mod simd_json {
  use crate::client_api_framework::{data_format::GraphQlResponse, dnsn::SimdJson};
  use core::fmt::Display;

  impl<D, E> crate::client_api_framework::dnsn::Deserialize<SimdJson> for GraphQlResponse<D, E>
  where
    D: for<'de> serde::Deserialize<'de>,
    E: for<'de> serde::Deserialize<'de>,
  {
    fn from_bytes(bytes: &[u8], _: &mut SimdJson) -> crate::Result<Self> {
      Ok(simd_json::from_reader(bytes)?)
    }

    fn seq_from_bytes<ERR>(
      _: &[u8],
      _: &mut SimdJson,
      _: impl FnMut(Self) -> Result<(), ERR>,
    ) -> Result<(), ERR>
    where
      ERR: Display + From<crate::Error>,
    {
      Err(crate::Error::UnsupportedOperation.into())
    }
  }

  impl<D, E> crate::client_api_framework::dnsn::Serialize<SimdJson> for GraphQlResponse<D, E>
  where
    D: serde::Serialize,
    E: serde::Serialize,
  {
    fn to_bytes(&mut self, bytes: &mut Vec<u8>, _: &mut SimdJson) -> crate::Result<()> {
      if core::mem::size_of::<Self>() == 0 {
        return Ok(());
      }
      simd_json::to_writer(bytes, &self.result)?;
      Ok(())
    }
  }
}
