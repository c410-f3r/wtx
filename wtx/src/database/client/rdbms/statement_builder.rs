use crate::{
  database::client::rdbms::statements_misc::StatementsMisc,
  misc::{BlocksDeque, BlocksDequeBuilder, BufferMode},
};
use hashbrown::HashMap;

#[derive(Debug)]
pub(crate) struct StatementBuilder<'stmts, A, C, T> {
  pub(crate) builder: BlocksDequeBuilder<'stmts, (C, T), StatementsMisc<A>, true>,
  pub(crate) curr_len: usize,
  pub(crate) indcs: &'stmts mut HashMap<u64, usize>,
}

impl<'stmts, A, C, T> StatementBuilder<'stmts, A, C, T> {
  #[inline]
  pub(crate) fn new(
    stmts: &'stmts mut BlocksDeque<(C, T), StatementsMisc<A>>,
    stmts_indcs: &'stmts mut HashMap<u64, usize>,
  ) -> Self {
    let curr_len = stmts.blocks_len();
    Self { builder: stmts.builder_back(), curr_len, indcs: stmts_indcs }
  }

  #[inline]
  pub(crate) fn build(mut self, hash: u64, mut sm: StatementsMisc<A>) -> crate::Result<usize> {
    let len = self.builder.inserted_elements().len();
    sm.columns_len = sm.columns_len.min(len);
    sm.types_len = sm.types_len.min(len);
    let _ = self.indcs.insert(hash, self.curr_len);
    self.builder.build(sm)?;
    Ok(self.curr_len)
  }

  #[inline]
  pub(crate) fn expand(&mut self, additional: usize, elem: (C, T)) -> crate::Result<&mut Self>
  where
    C: Clone,
    T: Clone,
  {
    let _ = self.builder.expand(BufferMode::Additional(additional), elem)?;
    Ok(self)
  }

  #[inline]
  pub(crate) fn inserted_elements(&mut self) -> &mut [(C, T)] {
    self.builder.inserted_elements()
  }
}
