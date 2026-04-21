#[macro_use]
mod macros;

mod certificate_revocation_list;
mod cipher_suite;
mod client_verifier;
mod de;
mod decode_wrapper;
mod encode_wrapper;
mod key_schedule;
#[cfg(target_os = "linux")]
#[expect(non_camel_case_types, unsafe_op_in_unsafe_fn, unused, reason = "generated code")]
mod ktls_bindings;
#[cfg(target_os = "linux")]
mod ktls_stream;
mod misc;
mod protocol;
mod psk;
mod psk_ty;
mod revocation_reason;
mod revoked_certificate;
mod server_verifier;
mod shared_secret;
mod signed_certificate_data;
mod state;
mod tls_acceptor;
mod tls_buffer;
mod tls_config;
mod tls_connector;
mod tls_crypto;
mod tls_error;
mod tls_mode;
mod tls_stream;
mod tls_stream_connector;
mod tls_stream_reader;
mod tls_stream_writer;
mod trust_anchor;

use crate::crypto::MAX_HASH_LEN;
pub use certificate_revocation_list::CertificateRevocationList;
pub use client_verifier::ClientVerifier;
pub use protocol::{
  cipher_suite_ty::CipherSuiteTy,
  max_fragment_length::MaxFragmentLength,
  named_group::{NamedGroup, NamedGroupParam},
  protocol_version::ProtocolVersion,
};
pub use psk::Psk;
pub use psk_ty::PskTy;
pub use revocation_reason::RevocationReasonCode;
pub use revoked_certificate::RevokedCertificate;
pub use server_verifier::ServerVerifier;
pub use signed_certificate_data::SignedCertificateData;
pub use tls_acceptor::TlsAcceptor;
pub use tls_buffer::TlsBuffer;
pub use tls_config::TlsConfig;
pub use tls_connector::TlsConnector;
pub use tls_crypto::*;
pub use tls_error::TlsError;
pub use tls_mode::*;
pub use tls_stream::TlsStream;
pub use tls_stream_connector::TlsStreamConnector;
pub use tls_stream_reader::TlsStreamReader;
pub use tls_stream_writer::TlsStreamWriter;
pub use trust_anchor::TrustAnchor;

const MAX_ALPN_LEN: usize = 4;
const MAX_CIPHER_KEY_LEN: usize = 32;
const HELLO_RETRY_REQUEST: [u8; 32] = [
  207, 33, 173, 116, 229, 154, 97, 17, 190, 29, 140, 2, 30, 101, 184, 145, 194, 162, 17, 22, 122,
  187, 140, 94, 7, 158, 9, 226, 200, 168, 51, 156,
];
const IV_LEN: usize = 12;
const MAX_CERTIFICATES: usize = 3;
const MAX_LABEL_LEN: usize = 22 + MAX_HASH_LEN;
const MAX_KEY_SHARES_LEN: usize = 2;

/// Identifier of a certificate
pub type SerialNumber = crate::collection::ArrayVectorU8<u8, 20>;

#[cfg(feature = "tls-aws-lc-rs")]
type CurrTlsCrypto = tls_crypto::AwsLcRs;
#[cfg(all(feature = "tls-ring", not(any(feature = "tls-aws-lc-rs", feature = "tls-rust-crypto"))))]
type CurrTlsCrypto = tls_crypto::RustCrypto;
#[cfg(all(feature = "tls-rust-crypto", not(any(feature = "tls-aws-lc-rs", feature = "tls-ring"))))]
type CurrTlsCrypto = tls_crypto::RustCrypto;
#[cfg(not(any(feature = "tls-aws-lc-rs", feature = "tls-ring", feature = "tls-rust-crypto")))]
type CurrTlsCrypto = ();

type CurrAgreement = <CurrTlsCrypto as TlsCrypto>::Agreement;
type CurrCipherSuite = <CurrTlsCrypto as TlsCrypto>::CipherSuite;
type CurrEphemeralSecretKey = <CurrAgreement as crate::crypto::Agreement>::EphemeralSecretKey;
type CurrHash = <CurrCipherSuite as crate::tls::cipher_suite::CipherSuite>::Hash;
type CurrHkdf = <CurrTlsCrypto as TlsCrypto>::Hkdf;
