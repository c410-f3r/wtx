#[derive(Debug)]
pub struct TrustAnchor<'bytes> {
  pub(crate) name_constraints: Option<&'bytes [u8]>,
  pub(crate) subject: &'bytes [u8],
  pub(crate) subject_public_key_info: &'bytes [u8],
}
