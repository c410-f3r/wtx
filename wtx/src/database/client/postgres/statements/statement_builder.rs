use crate::{
  database::{
    client::postgres::{
      statements::{column::Column, statements_misc::StatementsMisc},
      Ty,
    },
    Identifier,
  },
  misc::{BlocksDeque, BlocksDequeBuilder, BufferMode},
};
use hashbrown::HashMap;

#[derive(Debug)]
pub(crate) struct StatementBuilder<'stmts> {
  pub(crate) builder: BlocksDequeBuilder<'stmts, (Column, Ty), StatementsMisc, true>,
  pub(crate) curr_len: usize,
  pub(crate) indcs: &'stmts mut HashMap<u64, usize>,
}

impl<'stmts> StatementBuilder<'stmts> {
  #[inline]
  pub(crate) fn new(
    stmts: &'stmts mut BlocksDeque<(Column, Ty), StatementsMisc>,
    stmts_indcs: &'stmts mut HashMap<u64, usize>,
  ) -> Self {
    let curr_len = stmts.blocks_len();
    Self { builder: stmts.builder_back(), curr_len, indcs: stmts_indcs }
  }

  #[inline]
  pub(crate) fn build(mut self, hash: u64, mut sm: StatementsMisc) -> crate::Result<usize> {
    let len = self.builder.inserted_elements().len();
    sm.columns_len = sm.columns_len.min(len);
    sm.types_len = sm.types_len.min(len);
    let _ = self.indcs.insert(hash, self.curr_len);
    self.builder.build(sm)?;
    Ok(self.curr_len)
  }

  #[inline]
  pub(crate) fn expand(&mut self, additional: usize) -> crate::Result<&mut Self> {
    let _ = self.builder.expand(
      BufferMode::Additional(additional),
      (Column::new(Identifier::new(), Ty::Any), Ty::Any),
    )?;
    Ok(self)
  }

  #[inline]
  pub(crate) fn inserted_elements(&mut self) -> &mut [(Column, Ty)] {
    self.builder.inserted_elements()
  }
}
