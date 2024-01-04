create_enum! {
  /// HTTP method
  #[derive(Clone, Copy, Debug, Eq, PartialEq)]
  pub enum Method<u8> {
    /// Connect
    Connect = (0, "connect"),
    /// Delete
    Delete = (1, "delete"),
    /// Get
    Get = (2, "get"),
    /// Head
    Head = (3, "head"),
    /// Options
    Options = (4, "options"),
    /// Patch
    Patch = (5, "patch"),
    /// Post
    Post = (6, "post"),
    /// Put
    Put = (7, "put"),
    /// Trace
    Trace = (8, "trace"),
  }
}

#[cfg(feature = "serde")]
mod serde {
  use crate::http::Method;
  use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

  impl<'de> Deserialize<'de> for Method {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Method, D::Error>
    where
      D: Deserializer<'de>,
    {
      let s = <&str>::deserialize(deserializer)?;
      Self::try_from(s).map_err(|err| de::Error::custom(err))
    }
  }

  impl Serialize for Method {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
      S: Serializer,
    {
      serializer.serialize_str(self.strings().custom)
    }
  }
}
