//! Mock DB

mod mock_client;
mod mock_record;
mod mock_records;
mod mock_ty;
mod tys;

use crate::{
  codec::{CodecController, DecodeWrapper, EncodeWrapper},
  database::{Database, DatabaseTy},
};
pub use mock_client::MockClient;
pub use mock_record::MockRecord;
pub use mock_records::MockRecords;
pub use mock_ty::MockTy;

/// Mock database
#[derive(Debug)]
pub struct MockDb;

impl Database for MockDb {
  const TY: DatabaseTy = DatabaseTy::Mock;

  type Record<'client> = MockRecord<'client>;
  type Records<'client> = MockRecords<'client>;
  type Ty = MockTy;
}

impl CodecController for MockDb {
  type DecodeWrapper<'inner, 'outer, 'rem>
    = DecodeWrapper<'inner, ()>
  where
    'inner: 'outer;
  type Error = crate::Error;
  type EncodeWrapper<'inner, 'outer, 'rem>
    = EncodeWrapper<'inner, ()>
  where
    'inner: 'outer;
}
