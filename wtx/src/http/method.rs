create_enum! {
  /// HTTP method
  #[derive(Clone, Copy, Debug, Eq, PartialEq)]
  pub enum Method<u8> {
    /// Connect
    Connect = (0, "CONNECT" | "connect"),
    /// Delete
    Delete = (1, "DELETE" | "delete"),
    /// Get
    Get = (2, "GET" | "get"),
    /// Head
    Head = (3, "HEAD" | "head"),
    /// Options
    Options = (4, "OPTIONS" | "options"),
    /// Patch
    Patch = (5, "PATCH" | "path"),
    /// Post
    Post = (6, "POST" | "post"),
    /// Put
    Put = (7, "PUT" | "put"),
    /// Trace
    Trace = (8, "TRACE" | "trace"),
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
      Self::try_from(s).map_err(de::Error::custom)
    }
  }

  impl Serialize for Method {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
      S: Serializer,
    {
      serializer.serialize_str(self.strings().custom[0])
    }
  }
}
