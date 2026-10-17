use crate::{
  codec::DecodeWrapper,
  collections::{ShortBoxSliceU8, ShortBoxStrU8},
  database::{Record, ValueIdent, client::mock_db::MockDb},
};

/// Mock record
#[derive(Clone, Debug, PartialEq)]
pub struct MockRecord<'client> {
  pub(crate) columns: &'client [(ShortBoxStrU8, ShortBoxSliceU8<u8>)],
}

impl<'client> Record<'client> for MockRecord<'client> {
  type Database = MockDb;

  #[inline]
  fn len(&self) -> usize {
    self.columns.len()
  }

  #[inline]
  fn value<CI>(&self, ci: CI) -> Option<DecodeWrapper<'client, ()>>
  where
    CI: ValueIdent<Self>,
  {
    let idx = ci.idx(self)?;
    let bytes = self.columns.get(idx)?.1.as_ref();
    Some(DecodeWrapper::new(bytes, ()))
  }
}

impl ValueIdent<MockRecord<'_>> for &str {
  #[inline]
  fn idx(&self, input: &MockRecord<'_>) -> Option<usize> {
    input.columns.iter().position(|el| el.0.as_str() == *self)
  }
}
