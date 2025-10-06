/// Used to specify the data type that is going to be sent to a counterpart.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Mime {
  /// application/grpc
  ApplicationGrpc,
  /// application/json
  ApplicationJson,
  /// application/octet-stream
  ApplicationOctetStream,
  /// application/vnd.google.protobuf
  ApplicationVndGoogleProtobuf,
  /// application/xml
  ApplicationXml,
  /// application/x-www-form-urlencoded
  ApplicationXWwwFormUrlEncoded,
  /// application/yaml
  ApplicationYaml,
  /// Anything
  Custom(&'static str),
  /// multipart/form-data
  MultipartFormData,
  /// text/plain
  TextPlain,
}

impl Mime {
  /// Common string representation.
  #[inline]
  pub const fn as_str(&self) -> &'static str {
    match self {
      Self::ApplicationGrpc => "application/grpc",
      Self::ApplicationJson => "application/json",
      Self::ApplicationOctetStream => "application/octet-stream",
      Self::ApplicationVndGoogleProtobuf => "application/vnd.google.protobuf",
      Self::ApplicationXml => "application/xml",
      Self::ApplicationXWwwFormUrlEncoded => "application/x-www-form-urlencoded",
      Self::ApplicationYaml => "application/yaml",
      Self::Custom(el) => el,
      Self::MultipartFormData => "multipart/form-data",
      Self::TextPlain => "text/plain",
    }
  }
}
