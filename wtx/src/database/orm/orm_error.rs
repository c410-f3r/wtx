/// Errors for the ORM feature
#[derive(Debug)]
pub enum OrmError {
  /// Some internal operation found a hash collision of two table ids (likely) or a hash collision
  /// due to a number of nested associations larger than `MAX_NODES_NUM` (unlikely).
  TableHashCollision(&'static str),
}
