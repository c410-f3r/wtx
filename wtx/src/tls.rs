//! Transport Layer Security (TLS)

#[macro_use]
mod macros;

mod handshake_path;
mod handshake_ty;
mod key_schedule;
#[cfg(all(feature = "std", target_os = "linux"))]
mod ktls_bindings;
#[cfg(all(feature = "std", target_os = "linux"))]
mod ktls_stream;
mod misc;
mod protocol;
mod public_keys;
mod read_record_info;
mod record_content_ty;
#[cfg(test)]
mod tests;
mod tls_acceptor;
mod tls_buffer;
mod tls_cc;
mod tls_cc_wrappers;
mod tls_config;
mod tls_connector;
mod tls_connector_builder;
mod tls_ctx;
mod tls_error;
mod tls_hash;
mod tls_hkdf;
mod tls_hmac;
mod tls_mode;
mod tls_stream;
mod tls_stream_bridge;
pub(crate) mod tls_stream_common;
mod tls_stream_reader;
mod tls_stream_writer;

use crate::{collections::ArrayVectorCopy, crypto::MAX_HASH_LEN};
pub use handshake_path::HandshakePath;
pub use handshake_ty::HandshakeTy;
pub use key_schedule::KeySchedule;
#[cfg(all(feature = "std", target_os = "linux"))]
pub use ktls_stream::KtlsStream;
pub use protocol::{
  alert::{Alert, AlertDescription, AlertLevel},
  alpn::Alpn,
  cipher_suite::CipherSuite,
  max_fragment_length::MaxFragmentLength,
  named_group::{NamedGroup, NamedGroupParam},
  new_session_ticket::NewSessionTicket,
  protocol_version::ProtocolVersion,
  server_name::ServerName,
  server_name_list::ServerNameList,
  signature_algorithms::SignatureAlgorithms,
  signature_scheme::SignatureScheme,
  supported_groups::SupportedGroups,
};
pub use public_keys::{PublicKeyRef, PublicKeys, PublicKeysIter};
pub use read_record_info::ReadRecordInfo;
pub use tls_acceptor::{TlsAcceptOutput, TlsAcceptor};
pub use tls_buffer::TlsBuffer;
pub use tls_config::TlsConfig;
pub use tls_connector::{
  ClientRecordsState, ManageRemainingServerRecordsInput, ServerRecordsState, TlsConnectOutput,
  TlsConnector,
};
pub use tls_connector_builder::TlsConnectorBuilder;
pub use tls_ctx::{
  TlsCtx, TlsCtxSk, TlsCtxSkInput, TlsCtxSkLoader, hardened_sk_ctx::HardenedSkCtx,
  plaintext_ctx::PlaintextCtx, sk_ctx::SkCtx, trusted_ctx::TrustedCtx,
  unverified_ctx::UnverifiedCtx,
};
pub use tls_error::TlsError;
pub use tls_mode::TlsMode;
pub use tls_stream::TlsStream;
pub use tls_stream_bridge::{TlsStreamBridge, TlsStreamBridgeData};
pub use tls_stream_reader::TlsStreamReader;
pub use tls_stream_writer::TlsStreamWriter;

const CHANGE_CIPHER_SPEC: [u8; 6] = [20, 3, 3, 0, 1, 1];
const DLFT_MAX_FRAGMENT_LENGTH: u16 = 1 << 14;
const HELLO_RETRY_REQUEST: [u8; 32] = [
  207, 33, 173, 116, 229, 154, 97, 17, 190, 29, 140, 2, 30, 101, 184, 145, 194, 162, 17, 22, 122,
  187, 140, 94, 7, 158, 9, 226, 200, 168, 51, 156,
];
const IV_LEN: usize = 12;
const MAX_ALPN_LEN: usize = 4;
const MAX_CERTIFICATES: usize = 3;
const MAX_CERTS: usize = 3;
const MAX_CIPHER_KEY_LEN: usize = 32;
const MAX_KEYS: usize = 3;
const MAX_LABEL_LEN: usize = 22 + MAX_HASH_LEN;
const MAX_KEY_UPDATES: usize = 11;
const MAX_WARNING_ALERTS: usize = 5;
const RECORD_HEADER_LEN: usize = 5;
const SERVER_SIG_CTX: &str = "TLS 1.3, server CertificateVerify\0";

/// The hash of the server's leaf certificate.
pub type TlsServerEndPoint = ArrayVectorCopy<u8, { crate::crypto::HashTy::MAX.len() }>;
