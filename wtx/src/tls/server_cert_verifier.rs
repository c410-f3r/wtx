/// Something that can verify a server certificate chain, and verify
/// signatures made by certificates.
#[allow(unreachable_pub)]
pub trait ServerCertVerifier {
  /// Verify the end-entity certificate `end_entity` is valid for the
  /// hostname `dns_name` and chains to at least one trust anchor.
  ///
  /// `intermediates` contains all certificates other than `end_entity` that
  /// were sent as part of the server's [Certificate] message. It is in the
  /// same order that the server sent them and may be empty.
  ///
  /// Note that none of the certificates have been parsed yet, so it is the responsibility of
  /// the implementer to handle invalid data. It is recommended that the implementer returns
  /// [`Error::InvalidCertificate(CertificateError::BadEncoding)`] when these cases are encountered.
  ///
  /// [Certificate]: https://datatracker.ietf.org/doc/html/rfc8446#section-4.4.2
  fn verify_server_cert(
    &self,
    end_entity: &CertificateDer<'_>,
    intermediates: &[CertificateDer<'_>],
    server_name: &ServerName<'_>,
    ocsp_response: &[u8],
    now: UnixTime,
  ) -> Result<ServerCertVerified, Error>;

  /// Verify a signature allegedly by the given server certificate.
  ///
  /// This method is only called for TLS1.3 handshakes.
  ///
  /// This method is very similar to `verify_tls12_signature`: but note the
  /// tighter ECDSA SignatureScheme semantics -- e.g. `SignatureScheme::ECDSA_NISTP256_SHA256`
  /// must only validate signatures using public keys on the right curve --
  /// rustls does not enforce this requirement for you.
  ///
  /// `cert` has already been validated by [`ServerCertVerifier::verify_server_cert`].
  ///
  /// If and only if the signature is valid, return `Ok(HandshakeSignatureValid)`.
  /// Otherwise, return an error -- rustls will send an alert and abort the
  /// connection.
  fn verify_server_signature(
    &self,
    message: &[u8],
    cert: &CertificateDer<'_>,
    dss: &DigitallySignedStruct,
  ) -> Result<HandshakeSignatureValid, Error>;
}
