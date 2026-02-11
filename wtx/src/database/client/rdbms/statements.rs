use crate::{
  collection::BlocksDeque,
  database::client::rdbms::{
    statement::{Statement, StatementMut},
    statement_builder::StatementBuilder,
    statements_misc::StatementsMisc,
  },
  misc::{FnMutFut, random_state},
  rng::Rng,
};
use foldhash::fast::FixedState;
use hashbrown::HashMap;

/// Statements
#[derive(Debug)]
pub(crate) struct Statements<A, C, T> {
  max_stmts: usize,
  rs: FixedState,
  stmts: BlocksDeque<(C, T), StatementsMisc<A>>,
  stmts_indcs: HashMap<u64, usize>,
}

impl<A, C, T> Statements<A, C, T> {
  pub(crate) fn new<RNG>(max_stmts: usize, rng: &mut RNG) -> Self
  where
    RNG: Rng,
  {
    Self {
      max_stmts: max_stmts.max(1),
      rs: random_state(rng),
      stmts: BlocksDeque::new(),
      stmts_indcs: HashMap::new(),
    }
  }

  pub(crate) fn with_capacity<RNG>(
    columns: usize,
    max_stmts: usize,
    rng: &mut RNG,
    stmts: usize,
  ) -> crate::Result<Self>
  where
    RNG: Rng,
  {
    Ok(Self {
      max_stmts: max_stmts.max(1),
      rs: random_state(rng),
      stmts: BlocksDeque::with_capacity(stmts, columns)?,
      stmts_indcs: HashMap::with_capacity(stmts),
    })
  }

  pub(crate) async fn builder<AUX>(
    &mut self,
    mut aux: AUX,
    mut stmt_cb: impl for<'any> FnMutFut<(&'any mut AUX, StatementsMisc<A>), Result = crate::Result<()>>,
  ) -> crate::Result<StatementBuilder<'_, A, C, T>>
  where
    A: Default,
  {
    if self.stmts.blocks_len() >= self.max_stmts {
      let to_remove = (self.max_stmts / 2).max(1);
      for _ in 0..to_remove {
        if let Some(stmt) = self.stmts.pop_front() {
          stmt_cb.call((&mut aux, stmt)).await?;
        }
      }
      self.stmts_indcs.retain(|_, value| {
        if *value < to_remove {
          return false;
        }
        *value = value.wrapping_sub(to_remove);
        true
      })
    }
    Ok(StatementBuilder::new(&mut self.stmts_indcs, &mut self.stmts))
  }

  pub(crate) fn clear(&mut self) {
    let Self { max_stmts: _, rs: _, stmts, stmts_indcs } = self;
    stmts.clear();
    stmts_indcs.clear();
  }

  pub(crate) fn get_by_idx(&self, idx: usize) -> Option<Statement<'_, A, C, T>>
  where
    A: Clone,
  {
    let stmt = self.stmts.get(idx)?;
    Some(Statement::new(
      stmt.misc._aux.clone(),
      stmt.misc.columns_len,
      stmt.misc.rows_len,
      stmt.misc.types_len,
      stmt.data,
    ))
  }

  pub(crate) fn get_by_idx_mut(&mut self, idx: usize) -> Option<StatementMut<'_, A, C, T>>
  where
    A: Clone,
  {
    let stmt = self.stmts.get_mut(idx)?;
    Some(StatementMut::new(
      stmt.misc._aux.clone(),
      &mut stmt.misc.columns_len,
      &mut stmt.misc.rows_len,
      &mut stmt.misc.types_len,
      stmt.data,
    ))
  }

  pub(crate) fn get_by_stmt_cmd_id_mut(
    &mut self,
    stmt_cmd_id: u64,
  ) -> Option<StatementMut<'_, A, C, T>>
  where
    A: Clone,
  {
    self.get_by_idx_mut(*self.stmts_indcs.get(&stmt_cmd_id)?)
  }

  pub(crate) const fn hasher_mut(&mut self) -> &mut FixedState {
    &mut self.rs
  }

  pub(crate) fn len(&self) -> usize {
    self.stmts.blocks_len()
  }
}

#[cfg(test)]
mod tests {
  use crate::{
    database::client::rdbms::{
      statements::Statements,
      statements_misc::StatementsMisc,
      tests::{_column0, _column1, _column2, _column3},
    },
    executor::Runtime,
    rng::{SeedableRng, Xorshift64},
  };

  // FIXME(MIRI): The modification of the vector's length makes MIRI think that there is an
  // invalid pointer using stacked borrows.
  //
  // | A | B |   | <- Push back one block of 2 elements. Length is 2
  // | A | B | C | <- Push back one block of 1 element. Length is 3
  // |   |   | C | <- Pop front two bloc0sk. Length is 1
  //
  // Such a behavior does not occur with "miri-tree-borrows".
  #[cfg_attr(miri, ignore)]
  #[test]
  fn two_statements() {
    Runtime::new().block_on(async {
      let mut stmts = Statements::new(2, &mut Xorshift64::from_std_random().unwrap());

      let stmt_id0 = 123;
      let mut builder = stmts.builder((), builder_fn).await.unwrap();
      {
        let data = builder.expand(2, ("", 0)).unwrap();
        data[0] = (_column0(), 100);
        data[1] = (_column1(), 100);
        let _ = builder.build(stmt_id0, StatementsMisc::new(10, 2, 0, 1)).unwrap();
        let stmt = stmts.get_by_stmt_cmd_id_mut(stmt_id0).unwrap().into_stmt();
        assert_eq!(stmt.columns().len(), 2);
        assert_eq!(stmt.column(0).unwrap(), &_column0());
        assert_eq!(stmt.column(1).unwrap(), &_column1());
        assert_eq!(stmt.tys().len(), 1);
        assert_eq!(stmt.ty(0).unwrap(), &100);
      }

      let stmt_id1 = 456;
      let mut builder = stmts.builder((), builder_fn).await.unwrap();
      {
        let data = builder.expand(1, ("", 0)).unwrap();
        data[0] = (_column2(), 200);
        let _ = builder.build(stmt_id1, StatementsMisc::new(11, 1, 0, 1)).unwrap();
        let stmt = stmts.get_by_stmt_cmd_id_mut(stmt_id0).unwrap().into_stmt();
        assert_eq!(stmt.columns().len(), 2);
        assert_eq!(stmt.column(0).unwrap(), &_column0());
        assert_eq!(stmt.column(1).unwrap(), &_column1());
        assert_eq!(stmt.tys().len(), 1);
        assert_eq!(stmt.ty(0).unwrap(), &100);
      }
      {
        let stmt = stmts.get_by_stmt_cmd_id_mut(stmt_id1).unwrap().into_stmt();
        assert_eq!(stmt.columns().len(), 1);
        assert_eq!(stmt.column(0).unwrap(), &_column2());
        assert_eq!(stmt.tys().len(), 1);
        assert_eq!(stmt.ty(0).unwrap(), &200);
      }

      let stmt_id2 = 789;
      let mut builder = stmts.builder((), builder_fn).await.unwrap();
      {
        let data = builder.expand(1, ("", 0)).unwrap();
        data[0].0 = _column3();
        let _ = builder.build(stmt_id2, StatementsMisc::new(12, 1, 0, 0)).unwrap();
        assert_eq!(stmts.get_by_stmt_cmd_id_mut(stmt_id0), None);
        let stmt = stmts.get_by_stmt_cmd_id_mut(stmt_id1).unwrap().into_stmt();
        assert_eq!(stmt.columns().len(), 1);
        assert_eq!(stmt.column(0).unwrap(), &_column2());
        assert_eq!(stmt.tys().len(), 1);
        assert_eq!(stmt.ty(0).unwrap(), &200);
      }
      {
        let stmt = stmts.get_by_stmt_cmd_id_mut(stmt_id2).unwrap().into_stmt();
        assert_eq!(stmt.columns().len(), 1);
        assert_eq!(stmt.column(0).unwrap(), &_column3());
        assert_eq!(stmt.tys().len(), 0);
      }

      stmts.clear();
      assert_eq!(stmts.get_by_stmt_cmd_id_mut(stmt_id0), None);
      assert_eq!(stmts.get_by_stmt_cmd_id_mut(stmt_id1), None);
      assert_eq!(stmts.get_by_stmt_cmd_id_mut(stmt_id2), None);
    });
  }

  pub(crate) async fn builder_fn(_: &mut (), _: StatementsMisc<i32>) -> crate::Result<()> {
    Ok(())
  }
}
