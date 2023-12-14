use crate::client_api_framework::{
  dnsn::{Deserialize, Serialize},
  Id,
};
use alloc::{string::String, vec::Vec};
use core::{
  borrow::Borrow,
  cmp::{Ord, Ordering},
  hash::{Hash, Hasher},
};

/// Replied from an issued [crate::data_format::JsonRpcRequest].
///
/// The `jsonrpc` field is not included because `2.0` is always expected.
#[derive(Debug)]
pub struct JsonRpcResponse<R> {
  /// The same value specified in the request.
  pub id: Id,
  /// Optional parameter returns by the counterpart.
  pub method: Option<String>,
  /// Contains the `result` or the `error` field.
  pub result: crate::Result<R>,
}

impl<D> Deserialize<()> for JsonRpcResponse<D>
where
  D: Default,
{
  #[inline]
  fn from_bytes(_: &[u8], _: &mut ()) -> crate::Result<Self> {
    Ok(Self { id: 0, method: None, result: Ok(D::default()) })
  }

  #[inline]
  fn seq_from_bytes<E>(_: &[u8], _: &mut (), _: impl FnMut(Self) -> Result<(), E>) -> Result<(), E>
  where
    E: From<crate::Error>,
  {
    Ok(())
  }
}

impl<D> Serialize<()> for JsonRpcResponse<D> {
  #[inline]
  fn to_bytes(&mut self, _: &mut Vec<u8>, _: &mut ()) -> crate::Result<()> {
    Ok(())
  }
}

impl<P> Borrow<Id> for JsonRpcResponse<P> {
  #[inline]
  fn borrow(&self) -> &Id {
    &self.id
  }
}

impl<R> Eq for JsonRpcResponse<R> {}

impl<R> Hash for JsonRpcResponse<R> {
  #[inline]
  fn hash<H>(&self, state: &mut H)
  where
    H: Hasher,
  {
    self.id.hash(state);
  }
}

impl<R> Ord for JsonRpcResponse<R> {
  #[inline]
  fn cmp(&self, other: &Self) -> Ordering {
    self.id.cmp(&other.id)
  }
}

impl<R> PartialEq for JsonRpcResponse<R> {
  #[inline]
  fn eq(&self, other: &Self) -> bool {
    self.id == other.id
  }
}

impl<R> PartialOrd for JsonRpcResponse<R> {
  #[inline]
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

#[cfg(feature = "serde")]
mod serde {
  use crate::client_api_framework::data_format::{JsonRpcResponse, JsonRpcResponseError};
  use core::marker::PhantomData;
  use serde::{de::Visitor, ser::SerializeStruct};

  impl<'de, R> serde::Deserialize<'de> for JsonRpcResponse<R>
  where
    R: serde::Deserialize<'de>,
  {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<JsonRpcResponse<R>, D::Error>
    where
      D: serde::de::Deserializer<'de>,
    {
      struct CustomVisitor<'de, R>(PhantomData<R>, PhantomData<&'de ()>)
      where
        R: serde::Deserialize<'de>;

      impl<'de, R> Visitor<'de> for CustomVisitor<'de, R>
      where
        R: serde::Deserialize<'de>,
      {
        type Value = JsonRpcResponse<R>;

        #[inline]
        fn expecting(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
          formatter.write_str("struct JsonRpcResponse")
        }

        #[inline]
        fn visit_map<V>(self, mut map: V) -> Result<JsonRpcResponse<R>, V::Error>
        where
          V: serde::de::MapAccess<'de>,
        {
          let mut error = None;
          let mut id = None;
          let mut jsonrpc = None;
          let mut method = None;
          let mut result = None;

          while let Some(key) = map.next_key()? {
            match key {
              Field::Error => {
                if error.is_some() {
                  return Err(serde::de::Error::duplicate_field("error"));
                }
                error = Some(map.next_value::<JsonRpcResponseError>()?);
              }
              Field::Id => {
                if id.is_some() {
                  return Err(serde::de::Error::duplicate_field("id"));
                }
                id = Some(map.next_value()?);
              }
              Field::JsonRpc => {
                if jsonrpc.is_some() {
                  return Err(serde::de::Error::duplicate_field("jsonrpc"));
                }
                jsonrpc = Some(map.next_value::<&str>()?);
              }
              Field::Method => {
                if method.is_some() {
                  return Err(serde::de::Error::duplicate_field("method"));
                }
                method = Some(map.next_value()?);
              }
              Field::Result => {
                if result.is_some() {
                  return Err(serde::de::Error::duplicate_field("result"));
                }
                result = Some(map.next_value()?);
              }
            }
          }

          if let Some(elem) = jsonrpc {
            if elem != "2.0" {
              return Err(serde::de::Error::custom("JsonRpc version must be 2.0"));
            }
          } else {
            return Err(serde::de::Error::missing_field("jsonrpc"));
          }

          Ok(JsonRpcResponse {
            id: if let Some(elem) = id {
              elem
            } else {
              return Err(serde::de::Error::missing_field("id"));
            },
            method,
            result: if let Some(elem) = error {
              Err(crate::Error::JsonRpcResultErr(elem.into()))
            } else {
              Ok(result.ok_or_else(|| serde::de::Error::missing_field("result"))?)
            },
          })
        }
      }

      const FIELDS: &[&str] = &["error", "result"];
      deserializer.deserialize_struct(
        "JsonRpcResponse",
        FIELDS,
        CustomVisitor(PhantomData, PhantomData),
      )
    }
  }

  impl<R> serde::Serialize for JsonRpcResponse<R>
  where
    R: serde::Serialize,
  {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
      S: serde::ser::Serializer,
    {
      let mut state = serializer.serialize_struct("JsonRpcResponse", 3)?;
      state.serialize_field("jsonrpc", "2.0")?;
      match self.result {
        Err(ref err) => {
          state.serialize_field("error", &alloc::string::ToString::to_string(&err))?;
        }
        Ok(ref el) => state.serialize_field("result", &el)?,
      }
      state.serialize_field("id", &self.id)?;
      state.end()
    }
  }

  #[derive(Debug, serde::Deserialize)]
  #[serde(field_identifier, rename_all = "lowercase")]
  enum Field {
    Error,
    Id,
    JsonRpc,
    Method,
    Result,
  }
}

#[cfg(feature = "serde_json")]
mod serde_json {
  use crate::client_api_framework::{
    data_format::JsonRpcResponse, dnsn::SerdeJson, misc::seq_visitor::_SeqVisitor,
  };
  use core::fmt::Display;

  impl<R> crate::client_api_framework::dnsn::Deserialize<SerdeJson> for JsonRpcResponse<R>
  where
    R: for<'serde_de> serde::Deserialize<'serde_de>,
  {
    #[inline]
    fn from_bytes(bytes: &[u8], _: &mut SerdeJson) -> crate::Result<Self> {
      Ok(serde_json::from_slice(bytes)?)
    }

    #[inline]
    fn seq_from_bytes<E>(
      bytes: &[u8],
      _: &mut SerdeJson,
      cb: impl FnMut(Self) -> Result<(), E>,
    ) -> Result<(), E>
    where
      E: Display + From<crate::Error>,
    {
      use ::serde::de::Deserializer;
      let mut de = serde_json::Deserializer::from_slice(bytes);
      de.deserialize_seq(_SeqVisitor::_new(cb)).map_err(Into::into)?;
      Ok(())
    }
  }

  impl<R> crate::client_api_framework::dnsn::Serialize<SerdeJson> for JsonRpcResponse<R>
  where
    R: serde::Serialize,
  {
    #[inline]
    fn to_bytes(&mut self, bytes: &mut Vec<u8>, _: &mut SerdeJson) -> crate::Result<()> {
      serde_json::to_writer(bytes, self)?;
      Ok(())
    }
  }
}

#[cfg(feature = "simd-json")]
mod simd_json {
  use crate::client_api_framework::{data_format::JsonRpcResponse, dnsn::SimdJson};
  use core::fmt::Display;

  impl<R> crate::client_api_framework::dnsn::Deserialize<SimdJson> for JsonRpcResponse<R>
  where
    R: for<'serde_de> serde::Deserialize<'serde_de>,
  {
    fn from_bytes(bytes: &[u8], _: &mut SimdJson) -> crate::Result<Self> {
      Ok(simd_json::from_reader(bytes)?)
    }

    fn seq_from_bytes<E>(
      _: &[u8],
      _: &mut SimdJson,
      _: impl FnMut(Self) -> Result<(), E>,
    ) -> Result<(), E>
    where
      E: Display + From<crate::Error>,
    {
      Err(crate::Error::UnsupportedOperation.into())
    }
  }

  impl<R> crate::client_api_framework::dnsn::Serialize<SimdJson> for JsonRpcResponse<R>
  where
    R: serde::Serialize,
  {
    fn to_bytes(&mut self, bytes: &mut Vec<u8>, _: &mut SimdJson) -> crate::Result<()> {
      simd_json::to_writer(bytes, self)?;
      Ok(())
    }
  }
}
