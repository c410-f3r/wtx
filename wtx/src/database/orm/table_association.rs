/// Table association and its associated Rust type
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TableAssociation {
  from_id_name: &'static str,
  has_inverse_flow: bool,
  skip_insert: bool,
  to_id_name: &'static str,
}

impl TableAssociation {
  /// Creates a new instance from all parameters
  #[inline]
  pub const fn new(
    from_id: &'static str,
    has_inverse_flow: bool,
    skip_insert: bool,
    to_id_name: &'static str,
  ) -> Self {
    Self { from_id_name: from_id, has_inverse_flow, skip_insert, to_id_name }
  }

  /// Caller id filed name
  #[inline]
  pub const fn from_id_name(&self) -> &'static str {
    self.from_id_name
  }

  /// A "one to many" relationship is expected by default but such behavior can be changed
  /// using the parameter.
  #[inline]
  pub const fn has_inverse_flow(&self) -> bool {
    self.has_inverse_flow
  }

  /// If `false`, then only a shallow insertion will be performed or in other words, only adds
  /// the referenced primary key.
  #[inline]
  pub const fn skip_insert(&self) -> bool {
    self.skip_insert
  }

  /// Callee id filed name
  #[inline]
  pub const fn to_id_name(&self) -> &'static str {
    self.to_id_name
  }
}
