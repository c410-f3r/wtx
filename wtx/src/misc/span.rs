#[derive(Debug)]
pub(crate) struct _Entered<'span> {
  #[cfg(feature = "tracing")]
  _elem: tracing::span::Entered<'span>,
  #[cfg(not(feature = "tracing"))]
  _elem: core::marker::PhantomData<&'span ()>,
}

#[derive(Debug)]
pub(crate) struct _Span {
  #[cfg(feature = "tracing")]
  _elem: tracing::span::Span,
  #[cfg(not(feature = "tracing"))]
  _elem: (),
}

impl _Span {
  pub(crate) fn _new(
    #[cfg(feature = "tracing")] _elem: tracing::span::Span,
    #[cfg(not(feature = "tracing"))] _elem: (),
  ) -> Self {
    Self { _elem }
  }

  pub(crate) fn _enter(&self) -> _Entered<'_> {
    _Entered {
      #[cfg(feature = "tracing")]
      _elem: self._elem.enter(),
      #[cfg(not(feature = "tracing"))]
      _elem: core::marker::PhantomData,
    }
  }
}
