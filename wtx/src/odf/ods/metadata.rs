use crate::collections::VectorUsize;

#[derive(Debug, Default)]
pub struct Metadata<S> {
  /// Map of sheet names/sheet path within zip archive
  pub(crate) names: VectorUsize<(S, S)>,
  /// See [`MetadataSheet`].
  pub(crate) sheets: VectorUsize<MetadataSheet<S>>,
}

/// Metadata of sheet
#[derive(Debug, Clone, PartialEq)]
pub struct MetadataSheet<S> {
  /// Visible
  pub is_visible: bool,
  /// Name
  pub name: S,
}
