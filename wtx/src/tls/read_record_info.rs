use crate::tls::protocol::record_content_ty::RecordContentTy;

/// Returned by methods that fetch an external TLS record.
#[derive(Debug)]
pub struct ReadRecordInfo {
  pub(crate) inner_ty: RecordContentTy,
  pub(crate) outer_ty: RecordContentTy,
  pub(crate) plaintext_len: usize,
}
