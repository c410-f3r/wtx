use crate::{
  database::{client::postgres::ty::Ty, Identifier},
  misc::{Rng, _random_state, _unreachable},
};
use alloc::collections::VecDeque;
use foldhash::fast::FixedState;
use hashbrown::HashMap;

const AVG_STMT_COLUMNS_LEN: usize = 4;
const AVG_STMT_PARAMS_LEN: usize = 4;
const DFLT_MAX_STMTS: usize = 128;
const INITIAL_ELEMENTS_CAP: usize = 8;
const NUM_OF_ELEMENTS_TO_REMOVE_WHEN_FULL: u8 = 8;

/// Statements
#[derive(Debug)]
pub struct Statements {
  columns: VecDeque<Column>,
  columns_start: usize,
  info_by_cmd_hash: HashMap<u64, usize>,
  info_by_cmd_hash_start: usize,
  info: VecDeque<StatementInfo>,
  max_stmts: usize,
  num_of_elements_to_remove_when_full: u8,
  params: VecDeque<Ty>,
  params_start: usize,
  rs: FixedState,
}

impl Statements {
  pub(crate) fn new<RNG>(max_stmts: usize, rng: RNG) -> Self
  where
    RNG: Rng,
  {
    Self {
      columns: VecDeque::with_capacity(INITIAL_ELEMENTS_CAP.saturating_mul(AVG_STMT_COLUMNS_LEN)),
      columns_start: 0,
      info_by_cmd_hash: HashMap::with_capacity(INITIAL_ELEMENTS_CAP),
      info_by_cmd_hash_start: 0,
      info: VecDeque::with_capacity(INITIAL_ELEMENTS_CAP),
      max_stmts,
      num_of_elements_to_remove_when_full: NUM_OF_ELEMENTS_TO_REMOVE_WHEN_FULL,
      params: VecDeque::with_capacity(INITIAL_ELEMENTS_CAP.saturating_mul(AVG_STMT_PARAMS_LEN)),
      params_start: 0,
      rs: _random_state(rng),
    }
  }

  pub(crate) fn _empty() -> Self {
    Self {
      columns: VecDeque::new(),
      columns_start: 0,
      info: VecDeque::new(),
      info_by_cmd_hash: HashMap::new(),
      info_by_cmd_hash_start: 0,
      rs: FixedState::with_seed(0),
      max_stmts: 0,
      num_of_elements_to_remove_when_full: 0,
      params: VecDeque::new(),
      params_start: 0,
    }
  }

  pub(crate) fn with_default_params<RNG>(rng: &mut RNG) -> Self
  where
    RNG: Rng,
  {
    Self::new(DFLT_MAX_STMTS, rng)
  }

  pub(crate) fn clear(&mut self) {
    let Self {
      columns,
      columns_start,
      rs: _,
      info_by_cmd_hash,
      info_by_cmd_hash_start,
      info,
      max_stmts: _,
      num_of_elements_to_remove_when_full: _,
      params,
      params_start,
    } = self;
    columns.clear();
    *columns_start = 0;
    info_by_cmd_hash.clear();
    *info_by_cmd_hash_start = 0;
    info.clear();
    params.clear();
    *params_start = 0;
  }

  pub(crate) fn get_by_stmt_hash(&self, stmt_hash: u64) -> Option<Statement<'_>> {
    let mut info_idx = *self.info_by_cmd_hash.get(&stmt_hash)?;
    info_idx = info_idx.wrapping_sub(self.info_by_cmd_hash_start);
    let info_slice_opt = self.info.as_slices().0.get(..=info_idx);
    let (columns_range, params_range) = match info_slice_opt {
      None | Some([]) => _unreachable(),
      Some([a]) => (
        0..a.columns_offset.wrapping_sub(self.columns_start),
        0..a.params_offset.wrapping_sub(self.params_start),
      ),
      Some([.., a, b]) => (
        {
          let start = a.columns_offset.wrapping_sub(self.columns_start);
          let end = b.columns_offset.wrapping_sub(self.columns_start);
          start..end
        },
        {
          let start = a.params_offset.wrapping_sub(self.params_start);
          let end = b.params_offset.wrapping_sub(self.params_start);
          start..end
        },
      ),
    };
    let columns = self.columns.as_slices().0;
    let params = self.params.as_slices().0;
    if let (Some(a), Some(b)) = (columns.get(columns_range), params.get(params_range)) {
      Some(Statement::new(a, b))
    } else {
      _unreachable();
    }
  }

  pub(crate) fn hasher_mut(&mut self) -> &mut FixedState {
    &mut self.rs
  }

  pub(crate) fn push(&mut self, stmt_hash: u64) -> PushRslt<'_> {
    if self.info_by_cmd_hash.get(&stmt_hash).is_some() {
      #[expect(clippy::unwrap_used, reason = "borrow checker woes")]
      return PushRslt::Stmt(self.get_by_stmt_hash(stmt_hash).unwrap());
    }
    if self.info.len() >= self.max_stmts {
      let remove = usize::from(self.num_of_elements_to_remove_when_full).min(self.max_stmts / 2);
      for _ in 0..remove {
        self.remove_first_stmt();
      }
    }
    PushRslt::Builder(StatementBuilder { columns_len: 0, params_len: 0, stmt_hash, stmts: self })
  }

  fn remove_first_stmt(&mut self) {
    let Some(info) = self.info.pop_front() else {
      return;
    };

    let columns_len = info.columns_offset.wrapping_sub(self.columns_start);
    for _ in 0..columns_len {
      let _ = self.columns.pop_front();
    }
    self.columns_start = self.columns_start.wrapping_add(columns_len);

    let params_len = info.params_offset.wrapping_sub(self.params_start);
    for _ in 0..params_len {
      let _ = self.params.pop_front();
    }
    self.params_start = self.params_start.wrapping_add(params_len);

    let _ = self.info_by_cmd_hash.remove(&info.stmt_hash);
    self.info_by_cmd_hash_start = self.info_by_cmd_hash_start.wrapping_add(1);
  }
}

#[derive(Debug)]
pub(crate) enum PushRslt<'stmts> {
  Builder(StatementBuilder<'stmts>),
  Stmt(Statement<'stmts>),
}

#[cfg_attr(test, derive(Clone))]
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct Column {
  pub(crate) name: Identifier,
  pub(crate) ty: Ty,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct Statement<'stmts> {
  pub(crate) columns: &'stmts [Column],
  pub(crate) params: &'stmts [Ty],
}

impl<'stmts> Statement<'stmts> {
  pub(crate) const fn new(columns: &'stmts [Column], params: &'stmts [Ty]) -> Self {
    Self { columns, params }
  }
}

#[derive(Debug)]
pub(crate) struct StatementBuilder<'stmts> {
  columns_len: usize,
  params_len: usize,
  stmt_hash: u64,
  stmts: &'stmts mut Statements,
}

impl<'stmts> StatementBuilder<'stmts> {
  // Returning `&'stmts mut Statements` because of borrow checker limitations.
  pub(crate) fn finish(self) -> &'stmts mut Statements {
    let (last_columns_offset, last_params_offset) = self
      .stmts
      .info
      .as_slices()
      .0
      .last()
      .map_or((self.stmts.columns_start, self.stmts.params_start), |el| {
        (el.columns_offset, el.params_offset)
      });
    let _ = self.stmts.info_by_cmd_hash.insert(
      self.stmt_hash,
      self.stmts.info_by_cmd_hash_start.wrapping_add(self.stmts.info.len()),
    );
    self.stmts.info.push_back(StatementInfo {
      columns_offset: last_columns_offset.wrapping_add(self.columns_len),
      params_offset: last_params_offset.wrapping_add(self.params_len),
      stmt_hash: self.stmt_hash,
    });
    self.stmts
  }

  pub(crate) fn push_column(&mut self, column: Column) {
    self.stmts.columns.push_back(column);
    self.columns_len = self.columns_len.wrapping_add(1);
  }

  pub(crate) fn push_param(&mut self, param: Ty) {
    self.stmts.params.push_back(param);
    self.params_len = self.params_len.wrapping_add(1);
  }
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct StatementInfo {
  pub(crate) columns_offset: usize,
  pub(crate) params_offset: usize,
  pub(crate) stmt_hash: u64,
}

#[cfg(test)]
mod tests {
  use crate::{
    database::client::postgres::{
      statements::{Column, PushRslt, Statement},
      tests::{column0, column1, column2, column3},
      ty::Ty,
      Statements,
    },
    misc::{simple_seed, Vector, Xorshift64},
  };

  #[test]
  fn stmt_if_duplicated() {
    let stmt_hash = 123;
    let mut stmts = Statements::new(100, &mut Xorshift64::from(simple_seed()));
    let PushRslt::Builder(builder) = stmts.push(stmt_hash) else { panic!() };
    let _ = builder.finish();
    let PushRslt::Stmt(_) = stmts.push(stmt_hash) else { panic!() };
  }

  #[test]
  fn two_statements() {
    let mut stmts = Statements::new(2, &mut Xorshift64::from(simple_seed()));
    stmts.num_of_elements_to_remove_when_full = 1;

    let stmt_id0 = 123;
    let PushRslt::Builder(mut builder) = stmts.push(stmt_id0) else { panic!() };
    builder.push_column(column0());
    builder.push_column(column1());
    builder.push_param(Ty::Int2);
    let _ = builder.finish();
    assert_stmts(
      AssertStatements {
        columns: &[column0(), column1()],
        columns_offset_start: 0,
        info: &[(2, 1)],
        info_by_cmd_hash: &[0],
        params: &[Ty::Int2],
        params_offset_start: 0,
      },
      &stmts,
    );
    assert_eq!(
      stmts.get_by_stmt_hash(stmt_id0),
      Some(Statement::new(&[column0(), column1()], &[Ty::Int2]))
    );

    let stmt_id1 = 456;
    let PushRslt::Builder(mut builder) = stmts.push(stmt_id1) else { panic!() };
    builder.push_column(column2());
    builder.push_param(Ty::Int4);
    let _ = builder.finish();
    assert_stmts(
      AssertStatements {
        columns: &[column0(), column1(), column2()],
        columns_offset_start: 0,
        info: &[(2, 1), (3, 2)],
        info_by_cmd_hash: &[0, 1],
        params: &[Ty::Int2, Ty::Int4],
        params_offset_start: 0,
      },
      &stmts,
    );
    assert_eq!(
      stmts.get_by_stmt_hash(stmt_id0),
      Some(Statement::new(&[column0(), column1()], &[Ty::Int2]))
    );
    assert_eq!(stmts.get_by_stmt_hash(stmt_id1), Some(Statement::new(&[column2()], &[Ty::Int4])));

    let stmt_id2 = 789;
    let PushRslt::Builder(mut builder) = stmts.push(stmt_id2) else { panic!() };
    builder.push_column(column3());
    let _ = builder.finish();
    assert_stmts(
      AssertStatements {
        columns: &[column2(), column3()],
        columns_offset_start: 2,
        info: &[(3, 2), (4, 2)],
        info_by_cmd_hash: &[1, 2],
        params: &[Ty::Int4],
        params_offset_start: 1,
      },
      &stmts,
    );
    assert_eq!(stmts.get_by_stmt_hash(stmt_id0), None);
    assert_eq!(stmts.get_by_stmt_hash(stmt_id1), Some(Statement::new(&[column2()], &[Ty::Int4])));
    assert_eq!(stmts.get_by_stmt_hash(stmt_id2), Some(Statement::new(&[column3()], &[])));

    stmts.clear();
    assert_stmts(
      AssertStatements {
        columns: &[],
        columns_offset_start: 0,
        info: &[],
        info_by_cmd_hash: &[],
        params: &[],
        params_offset_start: 0,
      },
      &stmts,
    );
    assert_eq!(stmts.get_by_stmt_hash(stmt_id0), None);
    assert_eq!(stmts.get_by_stmt_hash(stmt_id1), None);
    assert_eq!(stmts.get_by_stmt_hash(stmt_id2), None);
  }

  #[track_caller]
  fn assert_stmts(cs: AssertStatements<'_>, stmts: &Statements) {
    assert_eq!(stmts.columns.as_slices().0, cs.columns);
    assert_eq!(stmts.columns.as_slices().1, &[]);
    assert_eq!(stmts.columns_start, cs.columns_offset_start);
    assert_eq!(
      Vector::from_iter(stmts.info.iter().map(|el| (el.columns_offset, el.params_offset)))
        .unwrap()
        .as_slice(),
      cs.info
    );
    assert_eq!(
      &{
        let mut vec = Vector::from_iter(stmts.info_by_cmd_hash.iter().map(|el| *el.1)).unwrap();
        vec.sort();
        vec
      }[..],
      cs.info_by_cmd_hash
    );
    assert_eq!(stmts.params.as_slices().0, cs.params);
    assert_eq!(stmts.params.as_slices().1, &[]);
    assert_eq!(stmts.params_start, cs.params_offset_start);
  }

  struct AssertStatements<'data> {
    columns: &'data [Column],
    columns_offset_start: usize,
    info: &'data [(usize, usize)],
    info_by_cmd_hash: &'data [usize],
    params: &'data [Ty],
    params_offset_start: usize,
  }
}
