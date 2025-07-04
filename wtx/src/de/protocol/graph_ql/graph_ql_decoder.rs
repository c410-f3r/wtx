use crate::{
  collection::Vector,
  de::{
    Decode, DecodeSeq, Encode,
    format::{De, DecodeWrapper, EncodeWrapper},
    protocol::GraphQlResponseError,
  },
};

/// Replied from an issued [`crate::de::protocol::GraphQlEncoder`].
#[derive(Debug)]
pub struct GraphQlDecoder<D, E> {
  /// Content depends if request was successful or not.
  pub result: Result<D, Vector<GraphQlResponseError<E>>>,
}

impl<'de, D, E> Decode<'de, De<()>> for GraphQlDecoder<D, E>
where
  D: Default,
{
  #[inline]
  fn decode(_: &mut (), _: &mut DecodeWrapper<'de>) -> crate::Result<Self> {
    Ok(Self { result: Ok(D::default()) })
  }
}

impl<'de, D, E> DecodeSeq<'de, De<()>> for GraphQlDecoder<D, E>
where
  D: Default,
{
  #[inline]
  fn decode_seq(_: &mut (), _: &mut Vector<Self>, _: &mut DecodeWrapper<'de>) -> crate::Result<()> {
    Ok(())
  }
}

impl<D, E> Encode<De<()>> for GraphQlDecoder<D, E> {
  #[inline]
  fn encode(&self, _: &mut (), _: &mut EncodeWrapper<'_>) -> crate::Result<()> {
    Ok(())
  }
}

#[cfg(feature = "serde")]
mod serde {
  use crate::{
    collection::Vector,
    de::protocol::{GraphQlDecoder, GraphQlResponseError},
  };
  use core::marker::PhantomData;
  use serde::{
    Deserialize, Serialize,
    de::{Deserializer, MapAccess, Visitor},
    ser::{SerializeStruct, Serializer},
  };

  impl<'de, D, E> Deserialize<'de> for GraphQlDecoder<D, E>
  where
    D: Deserialize<'de>,
    E: Deserialize<'de>,
  {
    #[inline]
    fn deserialize<DE>(deserializer: DE) -> Result<GraphQlDecoder<D, E>, DE::Error>
    where
      DE: Deserializer<'de>,
    {
      #[derive(Debug, serde::Deserialize)]
      #[serde(field_identifier, rename_all = "lowercase")]
      enum Field {
        Data,
        Errors,
      }

      struct CustomVisitor<'de, D, E>(PhantomData<(D, E)>, PhantomData<&'de ()>)
      where
        D: Deserialize<'de>,
        E: Deserialize<'de>;

      impl<'de, D, E> Visitor<'de> for CustomVisitor<'de, D, E>
      where
        D: Deserialize<'de>,
        E: Deserialize<'de>,
      {
        type Value = GraphQlDecoder<D, E>;

        fn expecting(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
          formatter.write_str("struct GraphQlDecoder")
        }

        #[inline]
        fn visit_map<V>(self, mut map: V) -> Result<GraphQlDecoder<D, E>, V::Error>
        where
          V: MapAccess<'de>,
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

          Ok(GraphQlDecoder {
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
        "GraphQlDecoder",
        FIELDS,
        CustomVisitor(PhantomData, PhantomData),
      )
    }
  }

  impl<D, E> Serialize for GraphQlDecoder<D, E>
  where
    D: Serialize,
    E: Serialize,
  {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
      S: Serializer,
    {
      let mut state = serializer.serialize_struct("GraphQlDecoder", 1)?;
      match self.result {
        Err(ref err) => {
          state.serialize_field("errors", err)?;
        }
        Ok(ref el) => state.serialize_field("data", &el)?,
      }
      state.end()
    }
  }
}

#[cfg(feature = "serde_json")]
mod serde_json {
  use crate::de::{
    format::SerdeJson,
    protocol::{GraphQlDecoder, misc::collect_using_serde_json},
  };
  use serde::{Deserialize, Serialize};

  _impl_dec! {
    GraphQlDecoder<(D): Deserialize<'de>, (E): Deserialize<'de>>,
    SerdeJson,
    |_aux, dw| {
      Ok(serde_json::from_slice(dw.bytes)?)
    }
  }

  _impl_dec_seq! {
    GraphQlDecoder<D: Deserialize<'de>, E: Deserialize<'de>>,
    SerdeJson,
    |_aux, buffer, dw| {
      collect_using_serde_json(buffer, &mut dw.bytes)
    }
  }

  _impl_enc! {
    GraphQlDecoder<D: Serialize, E: Serialize>,
    SerdeJson,
    |this, _aux, ew| {
      serde_json::to_writer(&mut *ew.vector, &this.result)?;
    }
  }
}
