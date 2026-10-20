use crate::{
  collections::{ShortBoxSliceU8, ShortBoxStrU8, TryExtend, VectorU8},
  database::{
    DbClient, RecordValues, StmtCmd,
    client::mock_db::{MockDb, MockRecord, MockRecords},
  },
  misc::{Lease, LeaseMut},
  net::ConnectionState,
};

/// Mock database
#[derive(Debug)]
pub struct MockClient {
  records: VectorU8<VectorU8<(ShortBoxStrU8, ShortBoxSliceU8<u8>)>>,
}

impl MockClient {
  /// New instance
  #[inline]
  pub fn new(records: VectorU8<VectorU8<(ShortBoxStrU8, ShortBoxSliceU8<u8>)>>) -> Self {
    Self { records }
  }
}

impl DbClient for MockClient {
  type Database = MockDb;

  #[inline]
  fn connection_state(&self) -> ConnectionState {
    ConnectionState::Open
  }

  #[inline]
  async fn execute_many<'this, B>(
    &'this mut self,
    buffer: &mut B,
    _: &str,
    _: impl FnMut(MockRecord<'_>) -> crate::Result<()>,
  ) -> crate::Result<()>
  where
    B: TryExtend<[MockRecords<'this>; 1]>,
  {
    buffer.try_extend([MockRecords {
      affected_rows: self.records.len().into(),
      records: &self.records,
    }])?;
    Ok(())
  }

  #[inline]
  async fn execute_stmt_many<SC, RV>(
    &mut self,
    _: SC,
    _: RV,
    _: impl FnMut(MockRecord<'_>) -> crate::Result<()>,
  ) -> crate::Result<MockRecords<'_>>
  where
    RV: RecordValues<Self::Database>,
    SC: StmtCmd,
  {
    Ok(MockRecords { affected_rows: self.records.len().into(), records: &self.records })
  }

  #[inline]
  async fn ping(&mut self) -> crate::Result<()> {
    Ok(())
  }

  #[inline]
  async fn prepare(&mut self, _: &str) -> crate::Result<u64> {
    Ok(0)
  }
}

impl Lease<MockClient> for MockClient {
  #[inline]
  fn lease(&self) -> &MockClient {
    self
  }
}

impl LeaseMut<MockClient> for MockClient {
  #[inline]
  fn lease_mut(&mut self) -> &mut MockClient {
    self
  }
}
