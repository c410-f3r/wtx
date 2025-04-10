#[derive(Debug)]
pub(crate) struct Entered<'span> {
  #[cfg(feature = "tracing")]
  _elem: tracing::span::Entered<'span>,
  #[cfg(not(feature = "tracing"))]
  _elem: &'span (),
}

#[derive(Clone, Debug)]
pub(crate) struct Span {
  #[cfg(feature = "tracing")]
  _elem: tracing::span::Span,
  #[cfg(not(feature = "tracing"))]
  _elem: (),
}

impl Span {
  pub(crate) const fn new(
    #[cfg(feature = "tracing")] _elem: tracing::span::Span,
    #[cfg(not(feature = "tracing"))] _elem: (),
  ) -> Self {
    Self { _elem }
  }

  pub(crate) fn enter(&self) -> Entered<'_> {
    Entered {
      #[cfg(feature = "tracing")]
      _elem: self._elem.enter(),
      #[cfg(not(feature = "tracing"))]
      _elem: &self._elem,
    }
  }
}
