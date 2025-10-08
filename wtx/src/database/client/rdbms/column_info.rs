pub(crate) trait ColumnInfo {
  type Ty;

  fn name(&self) -> &str;

  fn ty(&self) -> &Self::Ty;
}
