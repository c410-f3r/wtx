/// Auxiliary structures for streams or requests.
pub trait StreamAux: Sized {
  /// Initialization
  type Init;

  /// Creates a new instance with [`StreamAux::Init`] as well as with a request.
  fn stream_aux(init: Self::Init) -> crate::Result<Self>;
}
