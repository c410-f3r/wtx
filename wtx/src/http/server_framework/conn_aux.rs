/// Auxiliary structure for connections
pub trait ConnAux: Sized {
  /// Initialization
  type Init;

  /// Creates a new instance with [`ConnAux::Init`].
  fn conn_aux(init: Self::Init) -> crate::Result<Self>;
}
