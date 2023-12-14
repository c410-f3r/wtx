#![allow(
  // Indices point to valid data
  clippy::unreachable
)]

use crate::{
  database::{client::postgres::ty::Ty, Identifier},
  rng::Rng,
};
use ahash::RandomState;
use alloc::collections::VecDeque;
use hashbrown::HashMap;

const AVG_STMT_COLUMNS_LEN: usize = 4;
const AVG_STMT_PARAMS_LEN: usize = 4;
const INITIAL_ELEMENTS_CAP: usize = 8;
const DFLT_MAX_QUERIES: usize = 128;
const NUM_OF_ELEMENTS_TO_REMOVE_WHEN_FULL: u8 = 8;

/// Statements
#[derive(Debug)]
pub struct Statements {
  columns: VecDeque<Column>,
  columns_start: usize,
  hasher: RandomState,
  info_by_cmd_hash: HashMap<u64, usize>,
  info_by_cmd_hash_start: usize,
  info: VecDeque<StatementInfo>,
  max_queries: usize,
  num_of_elements_to_remove_when_full: u8,
  params: VecDeque<Ty>,
  params_start: usize,
}

impl Statements {
  pub(crate) fn new<RNG>(max_queries: usize, rng: &mut RNG) -> Self
  where
    RNG: Rng,
  {
    let (seed0, seed1) = {
      let [a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p] = rng.u8_16();
      (u64::from_ne_bytes([a, b, c, d, e, f, g, h]), u64::from_ne_bytes([i, j, k, l, m, n, o, p]))
    };
    let (seed2, seed3) = {
      let [a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p] = rng.u8_16();
      (u64::from_ne_bytes([a, b, c, d, e, f, g, h]), u64::from_ne_bytes([i, j, k, l, m, n, o, p]))
    };
    Self {
      columns: VecDeque::with_capacity(INITIAL_ELEMENTS_CAP.saturating_mul(AVG_STMT_COLUMNS_LEN)),
      columns_start: 0,
      info: VecDeque::with_capacity(INITIAL_ELEMENTS_CAP),
      info_by_cmd_hash: HashMap::with_capacity(INITIAL_ELEMENTS_CAP),
      info_by_cmd_hash_start: 0,
      hasher: RandomState::with_seeds(seed0, seed1, seed2, seed3),
      max_queries,
      num_of_elements_to_remove_when_full: NUM_OF_ELEMENTS_TO_REMOVE_WHEN_FULL,
      params: VecDeque::with_capacity(INITIAL_ELEMENTS_CAP.saturating_mul(AVG_STMT_PARAMS_LEN)),
      params_start: 0,
    }
  }

  pub(crate) fn with_default_params<RNG>(rng: &mut RNG) -> Self
  where
    RNG: Rng,
  {
    Self::new(DFLT_MAX_QUERIES, rng)
  }

  pub(crate) fn clear(&mut self) {
    let Self {
      columns,
      columns_start,
      hasher: _,
      info_by_cmd_hash,
      info_by_cmd_hash_start,
      info,
      max_queries: _,
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

  #[cfg(test)]
  pub(crate) fn get_by_query<'this>(&'this self, query: &str) -> Option<Statement<'this>> {
    self.get_by_cmd_hash(self.hasher.hash_one(query))
  }

  pub(crate) fn get_by_cmd_hash(&self, cmd_hash: u64) -> Option<Statement<'_>> {
    let mut info_idx = *self.info_by_cmd_hash.get(&cmd_hash)?;
    info_idx = info_idx.wrapping_sub(self.info_by_cmd_hash_start);
    let info_slice_opt = self.info.as_slices().0.get(..=info_idx);
    let (columns_range, params_range) = match info_slice_opt {
      None | Some([]) => unreachable!(),
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
      unreachable!();
    }
  }

  pub(crate) fn push<'this>(&'this mut self, query: &str) -> PushRslt<'this> {
    let cmd_hash = self.hasher.hash_one(query);
    if self.info_by_cmd_hash.get(&cmd_hash).is_some() {
      #[allow(
        // Borrow checker limitation
        clippy::unwrap_used
      )]
      return PushRslt::Stmt(cmd_hash, self.get_by_cmd_hash(cmd_hash).unwrap());
    }
    if self.info.len() >= self.max_queries {
      let remove = usize::from(self.num_of_elements_to_remove_when_full).min(self.max_queries / 2);
      for _ in 0..remove {
        self.remove_first_stmt();
      }
    }
    PushRslt::Builder(StatementBuilder { columns_len: 0, params_len: 0, cmd_hash, stmts: self })
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

    let _ = self.info_by_cmd_hash.remove(&info.cmd_hash);
    self.info_by_cmd_hash_start = self.info_by_cmd_hash_start.wrapping_add(1);
  }
}

#[derive(Debug)]
pub(crate) enum PushRslt<'stmts> {
  Builder(StatementBuilder<'stmts>),
  Stmt(u64, Statement<'stmts>),
}

#[cfg_attr(test, derive(Clone))]
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct Column {
  pub(crate) name: Identifier,
  pub(crate) value: Ty,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct Statement<'stmts> {
  pub(crate) columns: &'stmts [Column],
  pub(crate) params: &'stmts [Ty],
}

impl<'stmts> Statement<'stmts> {
  pub(crate) fn new(columns: &'stmts [Column], params: &'stmts [Ty]) -> Self {
    Self { columns, params }
  }
}

#[derive(Debug)]
pub(crate) struct StatementBuilder<'stmts> {
  columns_len: usize,
  params_len: usize,
  cmd_hash: u64,
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
    let _ = self
      .stmts
      .info_by_cmd_hash
      .insert(self.cmd_hash, self.stmts.info_by_cmd_hash_start.wrapping_add(self.stmts.info.len()));
    self.stmts.info.push_back(StatementInfo {
      columns_offset: last_columns_offset.wrapping_add(self.columns_len),
      params_offset: last_params_offset.wrapping_add(self.params_len),
      cmd_hash: self.cmd_hash,
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

  pub(crate) fn cmd_hash(&self) -> u64 {
    self.cmd_hash
  }
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct StatementInfo {
  pub(crate) columns_offset: usize,
  pub(crate) params_offset: usize,
  pub(crate) cmd_hash: u64,
}

#[cfg(test)]
mod tests {
  use crate::{
    database::client::postgres::{
      statements::{Column, PushRslt, Statement},
      ty::Ty,
      Statements,
    },
    rng::StaticRng,
  };
  use alloc::vec::Vec;

  #[test]
  fn stmt_if_duplicated() {
    let query = "FOO";
    let mut stmts = Statements::new(100, &mut StaticRng::default());
    let PushRslt::Builder(builder) = stmts.push(query) else { panic!() };
    let _ = builder.finish();
    let PushRslt::Stmt(_, _) = stmts.push(query) else { panic!() };
  }

  #[test]
  fn two_statements() {
    let mut stmts = Statements::new(2, &mut StaticRng::default());
    stmts.num_of_elements_to_remove_when_full = 1;

    let query0 = "SELECT a,b FROM foo WHERE id=$1";
    let PushRslt::Builder(mut builder) = stmts.push(query0) else { panic!() };
    builder.push_column(a());
    builder.push_column(b());
    builder.push_param(Ty::Int2);
    let _ = builder.finish();
    assert_stmts(
      AssertStatements {
        columns: &[a(), b()],
        columns_offset_start: 0,
        info: &[(2, 1)],
        info_by_cmd_hash: &[0],
        params: &[Ty::Int2],
        params_offset_start: 0,
      },
      &stmts,
    );
    assert_eq!(stmts.get_by_query(query0), Some(Statement::new(&[a(), b()], &[Ty::Int2])));

    let query1 = "SELECT c FROM bar WHERE id=$1";
    let PushRslt::Builder(mut builder) = stmts.push(query1) else { panic!() };
    builder.push_column(c());
    builder.push_param(Ty::Int4);
    let _ = builder.finish();
    assert_stmts(
      AssertStatements {
        columns: &[a(), b(), c()],
        columns_offset_start: 0,
        info: &[(2, 1), (3, 2)],
        info_by_cmd_hash: &[0, 1],
        params: &[Ty::Int2, Ty::Int4],
        params_offset_start: 0,
      },
      &stmts,
    );
    assert_eq!(stmts.get_by_query(query0), Some(Statement::new(&[a(), b()], &[Ty::Int2])));
    assert_eq!(stmts.get_by_query(query1), Some(Statement::new(&[c()], &[Ty::Int4])));

    let query2 = "SELECT d FROM baz";
    let PushRslt::Builder(mut builder) = stmts.push(query2) else { panic!() };
    builder.push_column(d());
    let _ = builder.finish();
    assert_stmts(
      AssertStatements {
        columns: &[c(), d()],
        columns_offset_start: 2,
        info: &[(3, 2), (4, 2)],
        info_by_cmd_hash: &[1, 2],
        params: &[Ty::Int4],
        params_offset_start: 1,
      },
      &stmts,
    );
    assert_eq!(stmts.get_by_query(query0), None);
    assert_eq!(stmts.get_by_query(query1), Some(Statement::new(&[c()], &[Ty::Int4])));
    assert_eq!(stmts.get_by_query(query2), Some(Statement::new(&[d()], &[])));

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
    assert_eq!(stmts.get_by_query(query0), None);
    assert_eq!(stmts.get_by_query(query1), None);
    assert_eq!(stmts.get_by_query(query2), None);
  }

  fn a() -> Column {
    Column { name: "a".try_into().unwrap(), value: Ty::VarcharArray }
  }

  #[track_caller]
  fn assert_stmts(cs: AssertStatements<'_>, stmts: &Statements) {
    assert_eq!(stmts.columns.as_slices().0, cs.columns);
    assert_eq!(stmts.columns.as_slices().1, &[]);
    assert_eq!(stmts.columns_start, cs.columns_offset_start);
    assert_eq!(
      &stmts.info.iter().map(|el| (el.columns_offset, el.params_offset)).collect::<Vec<_>>(),
      cs.info
    );
    assert_eq!(
      &{
        let mut vec = stmts.info_by_cmd_hash.iter().map(|el| *el.1).collect::<Vec<_>>();
        vec.sort();
        vec
      },
      cs.info_by_cmd_hash
    );
    assert_eq!(stmts.params.as_slices().0, cs.params);
    assert_eq!(stmts.params.as_slices().1, &[]);
    assert_eq!(stmts.params_start, cs.params_offset_start);
  }

  fn b() -> Column {
    Column { name: "b".try_into().unwrap(), value: Ty::Int8 }
  }

  fn c() -> Column {
    Column { name: "c".try_into().unwrap(), value: Ty::Char }
  }

  fn d() -> Column {
    Column { name: "d".try_into().unwrap(), value: Ty::Date }
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
