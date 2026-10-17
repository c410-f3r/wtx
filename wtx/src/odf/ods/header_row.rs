/// Starting header row
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum HeaderRow {
  /// First non-empty row
  #[default]
  FirstNonEmptyRow,
  /// Row number
  Row(u16),
}
