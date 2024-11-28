pub(crate) mod column;
pub(crate) mod statement;
pub(crate) mod statement_builder;
pub(crate) mod statements_misc;

use crate::{
  database::client::postgres::ty::Ty,
  misc::{BlocksDeque, Rng, _random_state},
};
use foldhash::fast::FixedState;
use hashbrown::HashMap;

/// Statements
#[derive(Debug)]
pub struct Statements {
  max_stmts: usize,
  rs: FixedState,
  stmts: BlocksDeque<(column::Column, Ty), statements_misc::StatementsMisc>,
  stmts_indcs: HashMap<u64, usize>,
}

impl Statements {
  #[inline]
  pub(crate) fn new<RNG>(max_stmts: usize, rng: RNG) -> Self
  where
    RNG: Rng,
  {
    Self {
      max_stmts: max_stmts.max(1),
      rs: _random_state(rng),
      stmts: BlocksDeque::new(),
      stmts_indcs: HashMap::new(),
    }
  }

  #[inline]
  pub(crate) fn with_capacity<RNG>(
    columns: usize,
    max_stmts: usize,
    rng: RNG,
    stmts: usize,
  ) -> crate::Result<Self>
  where
    RNG: Rng,
  {
    Ok(Self {
      max_stmts: max_stmts.max(1),
      rs: _random_state(rng),
      stmts: BlocksDeque::with_capacity(stmts, columns)?,
      stmts_indcs: HashMap::with_capacity(stmts),
    })
  }

  #[inline]
  pub(crate) fn builder(&mut self) -> statement_builder::StatementBuilder<'_> {
    if self.stmts.blocks_len() >= self.max_stmts {
      let to_remove = (self.max_stmts / 2).max(1);
      for _ in 0..to_remove {
        let _ = self.stmts.pop_front();
      }
      self.stmts_indcs.retain(|_, value| {
        if *value < to_remove {
          return false;
        }
        *value = value.wrapping_sub(to_remove);
        true
      })
    }
    statement_builder::StatementBuilder::new(&mut self.stmts, &mut self.stmts_indcs)
  }

  #[inline]
  pub(crate) fn clear(&mut self) {
    let Self { max_stmts: _, rs: _, stmts, stmts_indcs } = self;
    stmts.clear();
    stmts_indcs.clear();
  }

  #[inline]
  pub(crate) fn get_by_idx(&self, idx: usize) -> Option<statement::Statement<'_>> {
    let stmt = self.stmts.get(idx)?;
    Some(statement::Statement::new(stmt.misc.columns_len, stmt.misc.types_len, stmt.data))
  }

  #[inline]
  pub(crate) fn get_by_stmt_hash(&self, stmt_hash: u64) -> Option<statement::Statement<'_>> {
    self.get_by_idx(*self.stmts_indcs.get(&stmt_hash)?)
  }

  #[inline]
  pub(crate) fn hasher_mut(&mut self) -> &mut FixedState {
    &mut self.rs
  }
}

#[cfg(test)]
mod tests {
  use crate::{
    database::client::postgres::{
      statements::statements_misc::StatementsMisc,
      tests::{column0, column1, column2, column3},
      ty::Ty,
      Statements,
    },
    misc::{simple_seed, Xorshift64},
  };

  // FIXME(MIRI): The modification of the vector's length makes MIRI think that there is an
  // invalid pointer using stacked borrows.
  //
  // | A | B |   | <- Push back one block of 2 elements. Length is 2
  // | A | B | C | <- Push back one block of 1 element. Length is 3
  // |   |   | C | <- Pop front one block. Length is 1
  //
  // Such behaviour does not occur with "miri-tree-borrows".
  #[cfg_attr(miri, ignore)]
  #[test]
  fn two_statements() {
    let mut stmts = Statements::new(2, &mut Xorshift64::from(simple_seed()));

    let stmt_id0 = 123;
    let mut builder = stmts.builder();
    let _ = builder.expand(2).unwrap();
    builder.inserted_elements()[0] = (column0(), Ty::Int2);
    builder.inserted_elements()[1] = (column1(), Ty::Int2);
    let _ = builder.build(stmt_id0, StatementsMisc::new(2, 1)).unwrap();
    {
      let stmt = stmts.get_by_stmt_hash(stmt_id0).unwrap();
      assert_eq!(stmt.columns().count(), 2);
      assert_eq!(stmt.column(0).unwrap(), &column0());
      assert_eq!(stmt.column(1).unwrap(), &column1());
      assert_eq!(stmt.tys().count(), 1);
      assert_eq!(stmt.ty(0).unwrap(), Ty::Int2);
    }

    let stmt_id1 = 456;
    let mut builder = stmts.builder();
    let _ = builder.expand(1).unwrap();
    builder.inserted_elements()[0] = (column2(), Ty::Int4);
    let _ = builder.build(stmt_id1, StatementsMisc::new(1, 1)).unwrap();
    {
      let stmt = stmts.get_by_stmt_hash(stmt_id0).unwrap();
      assert_eq!(stmt.columns().count(), 2);
      assert_eq!(stmt.column(0).unwrap(), &column0());
      assert_eq!(stmt.column(1).unwrap(), &column1());
      assert_eq!(stmt.tys().count(), 1);
      assert_eq!(stmt.ty(0).unwrap(), Ty::Int2);
    }
    {
      let stmt = stmts.get_by_stmt_hash(stmt_id1).unwrap();
      assert_eq!(stmt.columns().count(), 1);
      assert_eq!(stmt.column(0).unwrap(), &column2());
      assert_eq!(stmt.tys().count(), 1);
      assert_eq!(stmt.ty(0).unwrap(), Ty::Int4);
    }

    let stmt_id2 = 789;
    let mut builder = stmts.builder();
    let _ = builder.expand(1).unwrap();
    builder.inserted_elements()[0].0 = column3();
    let _ = builder.build(stmt_id2, StatementsMisc::new(1, 0)).unwrap();
    assert_eq!(stmts.get_by_stmt_hash(stmt_id0), None);
    {
      let stmt = stmts.get_by_stmt_hash(stmt_id1).unwrap();
      assert_eq!(stmt.columns().count(), 1);
      assert_eq!(stmt.column(0).unwrap(), &column2());
      assert_eq!(stmt.tys().count(), 1);
      assert_eq!(stmt.ty(0).unwrap(), Ty::Int4);
    }
    {
      let stmt = stmts.get_by_stmt_hash(stmt_id2).unwrap();
      assert_eq!(stmt.columns().count(), 1);
      assert_eq!(stmt.column(0).unwrap(), &column3());
      assert_eq!(stmt.tys().count(), 0);
    }

    stmts.clear();
    assert_eq!(stmts.get_by_stmt_hash(stmt_id0), None);
    assert_eq!(stmts.get_by_stmt_hash(stmt_id1), None);
    assert_eq!(stmts.get_by_stmt_hash(stmt_id2), None);
  }
}
