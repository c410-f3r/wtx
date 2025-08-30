/// A root CA with the minimum required information to verify certificates.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TrustAnchor<'any> {
  /// Value of the `subject` field.
  pub subject: &'any [u8],
  /// Value of the `subjectPublicKeyInfo` field.
  pub subject_public_key_info: &'any [u8],
  /// Value of the `NameConstraints` field.
  pub name_constraints: Option<&'any [u8]>,
}
