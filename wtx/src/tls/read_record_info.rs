use crate::tls::protocol::record_content_type::RecordContentType;

/// Returned by methods that fetch an external TLS record.
#[derive(Debug)]
pub struct ReadRecordInfo {
  pub(crate) inner_ty: RecordContentType,
  pub(crate) outer_ty: RecordContentType,
  pub(crate) plaintext_len: usize,
}
