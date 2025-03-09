/// Express an instance with mutable references instead of their owned representations.
pub type PairMut<'any, PA, T> = Pair<&'any mut PA, &'any mut T>;

/// A convenient wrapper intended for anything that mutable dereferences to
/// [`crate::client_api_framework::pkg::PkgsAux`] along side any given `T: Transport`.
///
/// If desired, you can create your own set of APIs or transports by directly calling
/// their constructors.
///
/// # Types
///
/// * `PA`: PkgsAux
/// * `T`: Transport
#[derive(Debug)]
pub struct Pair<PA, T> {
  /// See [`crate::client_api_framework::pkg::PkgsAux`].
  pub pkgs_aux: PA,
  /// See [`crate::client_api_framework::network::transport::Transport`].
  pub trans: T,
}

impl<PA, T> Pair<PA, T> {
  /// Constructor shortcut.
  #[inline]
  pub const fn new(pkgs_aux: PA, trans: T) -> Self {
    Self { pkgs_aux, trans }
  }

  /// Gives an instance with mutable references.
  #[inline]
  pub fn as_mut(&mut self) -> PairMut<'_, PA, T> {
    PairMut { pkgs_aux: &mut self.pkgs_aux, trans: &mut self.trans }
  }

  /// Owned version of [`Self::parts_mut`].
  #[inline]
  pub fn into_parts(self) -> (PA, T) {
    (self.pkgs_aux, self.trans)
  }

  /// Another shortcut to easy development.
  #[inline]
  pub fn parts_mut(&mut self) -> (&mut PA, &mut T) {
    (&mut self.pkgs_aux, &mut self.trans)
  }
}

impl<PA, T> From<(PA, T)> for Pair<PA, T> {
  #[inline]
  fn from(from: (PA, T)) -> Self {
    Pair::new(from.0, from.1)
  }
}

impl<'this, PA, T> From<&'this mut Pair<PA, T>> for PairMut<'this, PA, T> {
  #[inline]
  fn from(from: &'this mut Pair<PA, T>) -> Self {
    Pair::new(&mut from.pkgs_aux, &mut from.trans)
  }
}

impl<'this, PA, T> From<&'this mut Pair<PA, T>> for (&'this mut PA, &'this mut T) {
  #[inline]
  fn from(from: &'this mut Pair<PA, T>) -> Self {
    (&mut from.pkgs_aux, &mut from.trans)
  }
}

impl<PA, T> From<Pair<PA, T>> for (PA, T) {
  #[inline]
  fn from(from: Pair<PA, T>) -> Self {
    (from.pkgs_aux, from.trans)
  }
}
