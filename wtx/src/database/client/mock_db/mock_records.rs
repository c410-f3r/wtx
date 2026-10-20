use crate::{
  collections::{ShortBoxSliceU8, ShortBoxStrU8, VectorU8},
  database::{
    Records,
    client::mock_db::{MockDb, MockRecord},
  },
  misc::Lease,
};

/// Mock records
#[derive(Debug, Default, PartialEq)]
pub struct MockRecords<'client> {
  pub(crate) affected_rows: u64,
  pub(crate) records: &'client [VectorU8<(ShortBoxStrU8, ShortBoxSliceU8<u8>)>],
}

impl<'client> Lease<MockRecords<'client>> for MockRecords<'client> {
  #[inline]
  fn lease(&self) -> &MockRecords<'client> {
    self
  }
}

impl<'client> Records<'client> for MockRecords<'client> {
  type Database = MockDb;

  #[inline]
  fn affected_rows(&self) -> u64 {
    self.affected_rows
  }

  #[inline]
  fn get(&self, record_idx: usize) -> Option<MockRecord<'client>> {
    self.records.get(record_idx).map(|columns| MockRecord { columns })
  }

  #[inline]
  fn iter(&self) -> impl Iterator<Item = MockRecord<'client>> {
    self.records.iter().map(|columns| MockRecord { columns })
  }

  #[inline]
  fn len(&self) -> usize {
    self.records.len()
  }
}
