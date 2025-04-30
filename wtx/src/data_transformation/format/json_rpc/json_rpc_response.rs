use crate::{
  collection::Vector,
  data_transformation::{
    Id,
    dnsn::{De, DecodeWrapper, EncodeWrapper},
  },
  misc::{Decode, DecodeSeq, Encode, Lease},
};
use alloc::boxed::Box;
use core::{
  borrow::Borrow,
  cmp::Ordering,
  hash::{Hash, Hasher},
};

/// Replied from an issued [`crate::data_transformation::format::JsonRpcRequest`].
///
/// The `jsonrpc` field is not included because `2.0` is always expected.
#[derive(Debug)]
pub struct JsonRpcResponse<R> {
  /// The same value specified in the request.
  pub id: Id,
  /// Optional parameter returns by the counterpart.
  pub method: Option<Box<str>>,
  /// Contains the `result` or the `error` field.
  pub result: crate::Result<R>,
}

impl<P> Borrow<Id> for JsonRpcResponse<P> {
  #[inline]
  fn borrow(&self) -> &Id {
    &self.id
  }
}

impl<'de, R> Decode<'de, De<()>> for JsonRpcResponse<R>
where
  R: Default,
{
  #[inline]
  fn decode(_: &mut (), _: &mut DecodeWrapper<'de>) -> crate::Result<Self> {
    Ok(Self { id: 0, method: None, result: Ok(R::default()) })
  }
}

impl<'de, R> DecodeSeq<'de, De<()>> for JsonRpcResponse<R>
where
  R: Default,
{
  #[inline]
  fn decode_seq(_: &mut (), _: &mut Vector<Self>, _: &mut DecodeWrapper<'de>) -> crate::Result<()> {
    Ok(())
  }
}

impl<D> Encode<De<()>> for JsonRpcResponse<D> {
  #[inline]
  fn encode(&self, _: &mut (), _: &mut EncodeWrapper<'_>) -> crate::Result<()> {
    Ok(())
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

impl<P> Lease<Id> for JsonRpcResponse<P> {
  #[inline]
  fn lease(&self) -> &Id {
    &self.id
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
  use crate::data_transformation::{
    DataTransformationError,
    format::{JsonRpcResponse, JsonRpcResponseError},
  };
  use core::marker::PhantomData;
  use serde::{
    Deserialize, Serialize,
    de::{Deserializer, MapAccess, Visitor},
    ser::{SerializeStruct, Serializer},
  };

  impl<'de, R> Deserialize<'de> for JsonRpcResponse<R>
  where
    R: Deserialize<'de>,
  {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<JsonRpcResponse<R>, D::Error>
    where
      D: Deserializer<'de>,
    {
      struct CustomVisitor<'de, R>(PhantomData<R>, PhantomData<&'de ()>)
      where
        R: Deserialize<'de>;

      impl<'de, R> Visitor<'de> for CustomVisitor<'de, R>
      where
        R: Deserialize<'de>,
      {
        type Value = JsonRpcResponse<R>;

        #[inline]
        fn expecting(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
          formatter.write_str("struct JsonRpcResponse")
        }

        #[inline]
        fn visit_map<V>(self, mut map: V) -> Result<JsonRpcResponse<R>, V::Error>
        where
          V: MapAccess<'de>,
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
              Err(DataTransformationError::JsonRpcResultErr(elem.into()).into())
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

  impl<R> Serialize for JsonRpcResponse<R>
  where
    R: Serialize,
  {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
      S: Serializer,
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
  use crate::data_transformation::{
    dnsn::SerdeJson,
    format::{JsonRpcResponse, misc::collect_using_serde_json},
  };
  use serde::{Deserialize, Serialize};

  _impl_dec! {
    JsonRpcResponse<R: Deserialize<'de>>,
    SerdeJson,
    |_aux, dw| {
      Ok(serde_json::from_slice(dw.bytes)?)
    }
  }

  _impl_dec_seq! {
    JsonRpcResponse<R: Deserialize<'de>>,
    SerdeJson,
    |_aux, buffer, dw| {
      collect_using_serde_json(buffer, &mut dw.bytes)
    }
  }

  _impl_enc! {
    JsonRpcResponse<R: Serialize>,
    SerdeJson,
    |this, _aux, ew| {
      serde_json::to_writer(&mut *ew.vector, this)?;
    }
  }
}
