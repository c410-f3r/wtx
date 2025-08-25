use crate::{
  database::{ValueIdent, client::rdbms::statement::Statement},
  misc::Lease,
};
use core::{marker::PhantomData, ops::Range};

/// Record used by several database implementations
#[derive(Debug)]
pub(crate) struct CommonRecord<'exec, A, C, D, T> {
  pub(crate) phantom: PhantomData<D>,
  pub(crate) record: &'exec [u8],
  pub(crate) stmt: Statement<'exec, A, C, T>,
  pub(crate) values_params: &'exec [(bool, Range<usize>)],
}

impl<'exec, A, C, D, T> CommonRecord<'exec, A, C, D, T> {
  pub(crate) fn new(
    record: &'exec [u8],
    stmt: Statement<'exec, A, C, T>,
    values_params: &'exec [(bool, Range<usize>)],
  ) -> Self {
    Self { phantom: PhantomData, record, stmt, values_params }
  }
}

impl<'exec, A, C, D, T> ValueIdent<CommonRecord<'exec, A, C, D, T>> for &str
where
  C: Lease<str>,
{
  fn idx(&self, input: &CommonRecord<'exec, A, C, D, T>) -> Option<usize> {
    input.stmt.columns().position(|column| column.lease() == *self)
  }
}
