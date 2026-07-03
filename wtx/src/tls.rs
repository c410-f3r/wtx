//! Transport Layer Security (TLS)

#[macro_use]
mod macros;

mod de;
mod handshake_path;
mod key_schedule;
#[cfg(all(feature = "std", target_os = "linux"))]
#[expect(
  clippy::allow_attributes_without_reason,
  clippy::arithmetic_side_effects,
  clippy::as_conversions,
  clippy::as_pointer_underscore,
  clippy::indexing_slicing,
  clippy::ptr_as_ptr,
  clippy::ref_as_ptr,
  clippy::renamed_function_params,
  clippy::std_instead_of_core,
  non_camel_case_types,
  trivial_casts,
  unreachable_pub,
  unsafe_op_in_unsafe_fn,
  unused,
  unused_results,
  reason = "generated code"
)]
mod ktls_bindings;
#[cfg(all(feature = "std", target_os = "linux"))]
mod ktls_stream;
mod misc;
mod protocol;
mod psk;
mod psk_ty;
mod read_record_info;
#[cfg(test)]
mod tests;
mod tls_acceptor;
mod tls_buffer;
mod tls_certificate;
mod tls_config;
mod tls_connector;
mod tls_decode_wrapper;
mod tls_encode_wrapper;
mod tls_error;
mod tls_hash;
mod tls_hkdf;
mod tls_hmac;
mod tls_mode;
mod tls_stream;
mod tls_stream_bridge;
mod tls_stream_reader;
mod tls_stream_writer;

use crate::{
  collections::ArrayVectorCopy,
  crypto::MAX_HASH_LEN,
  sync::{Arc, SyncMutex},
};
pub use handshake_path::HandshakePath;
use hashbrown::HashMap;
pub use key_schedule::KeySchedule;
#[cfg(all(feature = "std", target_os = "linux"))]
pub use ktls_stream::KtlsStream;
pub use protocol::{
  alpn::Alpn,
  cipher_suite::CipherSuite,
  max_fragment_length::MaxFragmentLength,
  named_group::{NamedGroup, NamedGroupParam},
  new_session_ticket::NewSessionTicket,
  protocol_version::ProtocolVersion,
};
pub use psk::Psk;
pub use psk_ty::PskTy;
pub use read_record_info::ReadRecordInfo;
pub use tls_acceptor::{TlsAcceptor, TlsAcceptorRslt};
pub use tls_buffer::TlsBuffer;
pub use tls_certificate::TlsCertificateTy;
pub use tls_config::TlsConfig;
pub use tls_connector::{
  ManageClientRecordsState, ManageRemainingServerRecordsInput, ManageRemainingServerRecordsState,
  TlsConnectRslt, TlsConnector,
};
pub use tls_error::TlsError;
pub use tls_mode::*;
pub use tls_stream::TlsStream;
pub use tls_stream_bridge::{TlsStreamBridge, TlsStreamBridgeData};
pub use tls_stream_reader::TlsStreamReader;
pub use tls_stream_writer::TlsStreamWriter;

const DLFT_MAX_FRAGMENT_LENGTH: u16 = (1 << 14) - 1;
pub(crate) const MAX_ALPN_LEN: usize = 4;
const MAX_CIPHER_KEY_LEN: usize = 32;
const HELLO_RETRY_REQUEST: [u8; 32] = [
  207, 33, 173, 116, 229, 154, 97, 17, 190, 29, 140, 2, 30, 101, 184, 145, 194, 162, 17, 22, 122,
  187, 140, 94, 7, 158, 9, 226, 200, 168, 51, 156,
];
const IV_LEN: usize = 12;
const MAX_CERTIFICATES: usize = 3;
const MAX_LABEL_LEN: usize = 22 + MAX_HASH_LEN;
const MAX_KEY_SHARES_LEN: usize = 2;
const SERVER_SIG_CTX: &str = "TLS 1.3, server CertificateVerify\0";

/// Pre Shared Keys
pub type Psks = Arc<SyncMutex<HashMap<ArrayVectorCopy<u8, MAX_HASH_LEN>, Psk>>>;
/// Identifier of a certificate
pub type SerialNumber = ArrayVectorCopy<u8, 20>;
/// The hash of the server's leaf certificate.
pub type TlsServerEndPoint = ArrayVectorCopy<u8, { MAX_HASH_LEN }>;

mod crypto {
  use crate::{
    codec::{Decode, Encode},
    crypto::SignatureTy,
    tls::{de::De, tls_decode_wrapper::TlsDecodeWrapper, tls_encode_wrapper::TlsEncodeWrapper},
  };

  impl SignatureTy {
    pub(crate) const TLS_PRIORITY: [Self; Self::len()] = [
      Self::Ed25519,
      Self::EcdsaSecp256r1Sha256,
      Self::EcdsaSecp384r1Sha384,
      Self::RsaPssRsaeSha256,
      Self::RsaPssRsaeSha384,
    ];
  }

  impl<'de> Decode<'de, De> for SignatureTy {
    #[inline]
    fn decode(dw: &mut TlsDecodeWrapper<'de>) -> crate::Result<Self> {
      Self::try_from(<u16 as Decode<De>>::decode(dw)?)
    }
  }

  impl Encode<De> for SignatureTy {
    #[inline]
    fn encode(&self, ew: &mut TlsEncodeWrapper<'_>) -> crate::Result<()> {
      ew.buffer().extend_from_copyable_slice(&u16::from(*self).to_be_bytes())?;
      Ok(())
    }
  }
}
