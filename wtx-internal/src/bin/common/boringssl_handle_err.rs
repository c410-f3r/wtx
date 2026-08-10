use crate::boringssl_options::{Options, quit};
use wtx::tls::{AlertDescription, TlsError};

pub fn handle_err(_opts: &Options, rslt: wtx::Result<()>) {
  let reason = match &rslt {
    Ok(_) => return,
    Err(wtx::Error::TlsError(err)) => match err {
      TlsError::AbortedHandshake(alert)
        if alert.description() == AlertDescription::HandshakeFailure =>
      {
        ":HANDSHAKE_FAILURE_ON_CLIENT_HELLO:"
      }
      TlsError::BadSignature => ":BAD_SIGNATURE:",
      TlsError::DigestCheckFailed => ":DIGEST_CHECK_FAILED:",
      TlsError::DuplicatedKeyShares => ":DUPLICATE_KEY_SHARE:",
      TlsError::InvalidAesData => ":BAD_DECRYPT:",
      TlsError::InvalidCertificateRequest => ":DECODE_ERROR:",
      TlsError::MismatchedCertificatePkAndSignature => ":WRONG_SIGNATURE_TYPE:",
      TlsError::MissingDigitalSignatureInKeyUsage => ":KEY_USAGE_BIT_INCORRECT:",
      TlsError::MissingSignatureAlgorithms => ":NO_COMMON_SIGNATURE_ALGORITHMS:",
      TlsError::NoCertificate => ":PEER_DID_NOT_RETURN_A_CERTIFICATE:",
      TlsError::SecretMismatch => ":WRONG_CURVE:",
      TlsError::TrailingDataInExtension => ":DECODE_ERROR:",
      TlsError::UnexpectedAfterHandshakeOuterRecord => ":INVALID_OUTER_RECORD_TYPE:",
      TlsError::UnknownKeyUpdateRequest => ":DECODE_ERROR:",
      TlsError::UnknownNamedGroup => ":WRONG_CURVE:",
      TlsError::UnknownProtocolVersion => ":WRONG_VERSION_NUMBER:",
      TlsError::UnknownSignatureScheme => ":WRONG_SIGNATURE_TYPE:",
      TlsError::UnsupportedCipherSuite => ":WRONG_CIPHER_RETURNED:",
      _ => ":FIXME:",
    },
    Err(wtx::Error::TlsErrorReply(err, _)) => match err {
      TlsError::ClientExpectedFinished => ":UNEXPECTED_MESSAGE:",
      TlsError::DiffieHellmanError => ":WRONG_CURVE:",
      TlsError::EmptyCertificateAuthorities => ":ERROR_PARSING_EXTENSION:",
      TlsError::EmptyNegotiatedAlpnClient => ":PARSE_TLSEXT:",
      TlsError::EmptyNewSessionTicket => ":DECODE_ERROR:",
      TlsError::ExcessHandshakeData(_) => ":EXCESS_HANDSHAKE_DATA:",
      TlsError::IncompleteHandshake => ":UNEXPECTED_MESSAGE:",
      TlsError::InvalidExtensionTy => ":UNEXPECTED_EXTENSION:",
      TlsError::InvalidLegacyCompressionMethod => ":DECODE_ERROR:",
      TlsError::InvalidLegacyCompressionMethods => ":INVALID_COMPRESSION_LIST:",
      TlsError::InvalidLegacySessionId => ":DECODE_ERROR:",
      TlsError::InvalidNegotiatedServerName => ":UNEXPECTED_EXTENSION:",
      TlsError::InvalidServerNameList => ":ERROR_PARSING_EXTENSION:",
      TlsError::InvalidX509 => ":CANNOT_PARSE_LEAF_CERT:",
      TlsError::MismatchedExtension => ":UNEXPECTED_EXTENSION:",
      TlsError::MismatchedNegotiatedAlpnClient => ":INVALID_ALPN_PROTOCOL:",
      TlsError::MismatchedNegotiatedAlpnServer => ":NO_APPLICATION_PROTOCOL:",
      TlsError::MissingKeyShares => ":MISSING_KEY_SHARE:",
      TlsError::MissingSupportedGroups => ":NO_SHARED_GROUP:",
      TlsError::PostHandshakeDecError(handshake_ty) => {
        if handshake_ty.is_finished() {
          ":DIGEST_CHECK_FAILED:"
        } else {
          ":DECODE_ERROR:"
        }
      }
      TlsError::PreHandshakeDecError => ":EXCESS_HANDSHAKE_DATA:",
      TlsError::ReceivedRecordIsTooLarge => ":DATA_LENGTH_TOO_LONG:",
      TlsError::ServerHasNoCompatibleSignatureScheme => ":NO_COMMON_SIGNATURE_ALGORITHMS:",
      TlsError::TooManyKeyUpdates => ":TOO_MANY_KEY_UPDATES:",
      TlsError::TooManyWarningAlerts => ":TOO_MANY_WARNING_ALERTS:",
      TlsError::TrailingDataInExtension => ":ERROR_PARSING_EXTENSION:",
      TlsError::UnencryptedRecord => ":BAD_DECRYPT:",
      TlsError::UnexpectedAfterHandshakeInnerRecord => ":UNEXPECTED_RECORD:",
      TlsError::UnknownHandshakeTy(_) => ":UNEXPECTED_MESSAGE:",
      TlsError::UnknownRecordContentType => ":BAD_DECRYPT:",
      TlsError::UnofferedExtension => ":UNEXPECTED_EXTENSION:",
      TlsError::WrongAlert => ":BAD_ALERT:",
      _ => ":FIXME:",
    },
    _ => ":FIXME:",
  };
  eprintln!("ERROR: {rslt:?}");
  quit(reason);
}
