/// Metadata of sheet
#[derive(Debug, Clone, PartialEq)]
pub struct Sheet<S> {
  /// Visible
  pub is_visible: bool,
  /// Name
  pub name: S,
}
