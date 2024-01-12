/// An enum that can contain two different types.
#[derive(Debug, PartialEq)]
pub enum Either<L, R> {
  /// Left
  Left(L),
  /// Right
  Right(R),
}
