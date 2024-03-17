create_enum! {
  /// HTTP method
  #[derive(Clone, Copy, Debug, Eq, PartialEq)]
  pub enum Method<u8> {
    /// Connect
    Connect = (0, "CONNECT"),
    /// Delete
    Delete = (1, "DELETE"),
    /// Get
    Get = (2, "GET"),
    /// Head
    Head = (3, "HEAD"),
    /// Options
    Options = (4, "OPTIONS"),
    /// Patch
    Patch = (5, "PATCH"),
    /// Post
    Post = (6, "POST"),
    /// Put
    Put = (7, "PUT"),
    /// Trace
    Trace = (8, "TRACE"),
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
