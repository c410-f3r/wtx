/// TLS errror
#[derive(Debug)]
pub enum TlsError {
  /// Invalid client hello
  InvalidClientHello,
  /// Invalid server name
  InvalidServerName,
  /// Invalid server name list
  InvalidServerNameList,
  /// Invalid u16 prefix
  InvalidU16Prefix,
  /// Unknown name type
  UnknownNameType,
  /// Unknown Webpki Signature Scheme
  UnknownWebpkiSignatureScheme,
}
