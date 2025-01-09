/// Database Error
#[derive(Debug)]
pub enum DatabaseError {
  /// A "null" field received from the database was decoded as a non-nullable type or value.
  MissingFieldDataInDecoding(&'static str),
}
