/// Used to specify the data type that is going to be sent to a counterpart.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Mime {
  /// Opaque bytes
  Bytes,
  /// Anything
  Custom(&'static str),
  /// JSON
  Json,
  /// JSON:API
  JsonApi,
  /// Protocol buffer
  Protobuf,
  /// Plain text
  Text,
  /// XML
  Xml,
  /// YAML
  Yaml,
}

impl Mime {
  /// Common string representation.
  #[inline]
  pub fn as_str(&self) -> &'static str {
    match self {
      Self::Bytes => "application/octet-stream",
      Self::Custom(el) => el,
      Self::Json => "application/json",
      Self::JsonApi => "application/vnd.api+json",
      Self::Protobuf => "application/vnd.google.protobuf",
      Self::Text => "text/plain",
      Self::Xml => "application/xml",
      Self::Yaml => "application/yaml",
    }
  }
}
