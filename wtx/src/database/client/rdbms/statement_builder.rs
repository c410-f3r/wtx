use crate::{
  collection::{BlocksDeque, ExpansionTy},
  database::{DatabaseError, client::rdbms::statements_misc::StatementsMisc},
};
use hashbrown::HashMap;

#[derive(Debug)]
pub(crate) struct StatementBuilder<'stmts, A, C, T> {
  indcs: &'stmts mut HashMap<u64, usize>,
  stmts: &'stmts mut BlocksDeque<(C, T), StatementsMisc<A>>,
  stmts_idx: usize,
}

impl<'stmts, A, C, T> StatementBuilder<'stmts, A, C, T>
where
  A: Default,
{
  pub(crate) fn new(
    indcs: &'stmts mut HashMap<u64, usize>,
    stmts: &'stmts mut BlocksDeque<(C, T), StatementsMisc<A>>,
  ) -> Self {
    let stmts_idx = stmts.blocks_len();
    Self { indcs, stmts, stmts_idx }
  }

  pub(crate) fn build(self, hash: u64, mut sm: StatementsMisc<A>) -> crate::Result<usize> {
    let Some(stmt) = self.stmts.get_mut(self.stmts_idx) else {
      return Err(DatabaseError::InconsistentStatementBuilder.into());
    };
    sm.columns_len = sm.columns_len.min(stmt.data.len());
    sm.types_len = sm.types_len.min(stmt.data.len());
    *stmt.misc = sm;
    let _ = self.indcs.insert(hash, self.stmts_idx);
    Ok(self.stmts_idx)
  }

  pub(crate) fn expand(&mut self, additional: usize, elem: (C, T)) -> crate::Result<&mut [(C, T)]>
  where
    C: Clone,
    T: Clone,
  {
    if self.stmts.blocks_len() > self.stmts_idx {
      return Err(DatabaseError::InconsistentStatementBuilder.into());
    };
    let _ = self.stmts.expand_back(
      ExpansionTy::Additional(additional),
      StatementsMisc::new(A::default(), 0, 0, 0),
      elem,
    )?;
    // SAFETY: `self.idx` refers to the elements that were inserted above.
    Ok(unsafe { self.stmts.get_mut(self.stmts_idx).unwrap_unchecked().data })
  }
}
