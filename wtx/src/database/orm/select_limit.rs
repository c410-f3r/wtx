/// Sql select `LIMIT` clause
#[derive(Clone, Copy, Debug)]
pub enum SelectLimit {
  /// LIMIT ALL
  All,
  /// LIMIT `n`
  Count(u32),
}
