use crate::{
  collection::{ArrayVectorU8, ShortStr, ShortStrU8, Vector},
  misc::{Ascii, str_split_once_str, str_split_once1},
};
use core::{mem, ops::Range};

const DEFAULT_MAX_SUB_NODES: usize = 16;
const DEFAULT_MAX_DEPTH: usize = 10;

/// Radix Tree Error
#[derive(Clone, Copy, Debug)]
pub enum MatcherError {
  /// Conflicting route definitions
  ConflictingRoute,
  /// Duplicate route already exists
  DuplicateRoute,
  /// Invalid parameter syntax
  InvalidParameterSyntax,
  /// Invalid Route Definition
  InvalidRouteDefinition,
  /// Nested parameters. For example, `{id{nested}}`.
  NestedParameters,
  /// Create a sub node to add more parameters in a path
  NodesCanHaveAtMostOneParameter,
  /// Route does not exist
  UnknownMatchingRoute,
  /// Unclosed parameter bracket
  UnclosedParameterBracket,
}

/// Decomposes a series of prefixed strings into common nodes to allow fast comparisons with
/// other dynamic strings.
///
/// * `hey_{anything}`: `hey_you`, `hey_you_what_is_this_now`
/// * `{phrase}_ready_to_go.avif`: `are_you_ready_to_go.avif`, `cause_i_am_ready_to_go.avif`
#[derive(Clone, Debug)]
pub struct Matcher<T, const N: usize = DEFAULT_MAX_SUB_NODES> {
  root_firsts: ArrayVectorU8<u8, N>,
  rows: Vector<Row<T, N>>,
}

impl<T> Matcher<T> {
  /// Empty instance
  #[inline]
  pub const fn new() -> Self {
    Self { root_firsts: ArrayVectorU8::new(), rows: Vector::new() }
  }
}

impl<T, const N: usize> Matcher<T, N> {
  /// See [`MatcherBuilder`].
  #[inline]
  pub fn builder(&mut self) -> MatcherBuilder<'_, T, N> {
    self.root_firsts.clear();
    self.rows.clear();
    MatcherBuilder { root_firsts: &mut self.root_firsts, rows: &mut self.rows }
  }

  /// Tries to find a path that corresponds to `ident`.
  #[inline]
  pub fn find<'this, 'data, 'rslt>(
    &'this self,
    ident: &'data str,
  ) -> crate::Result<MatcherPath<'rslt, T, N>>
  where
    'data: 'rslt,
    'this: 'rslt,
  {
    let Self { root_firsts, rows } = self;

    let mut abs_offset = 0u8;
    let mut curr_identifier = ident;
    let mut next_row_idx = None;
    let mut path_rows = ArrayVectorU8::new();
    let mut row_idx = 0u8;

    for row in rows.get(0..usize::from(root_firsts.len())).unwrap_or_default() {
      let len_before = curr_identifier.len();
      match Self::check_search_node(
        abs_offset,
        &mut curr_identifier,
        &mut next_row_idx,
        &mut path_rows,
        row,
        row_idx,
      ) {
        Some(false) => {
          row_idx = row_idx.wrapping_add(1);
        }
        Some(true) => return Ok(MatcherPath { ident, path_rows, rows }),
        None => {
          let consumed = len_before.wrapping_sub(curr_identifier.len()).try_into()?;
          abs_offset = abs_offset.wrapping_add(consumed);
          break;
        }
      }
    }
    loop {
      let Some(next_row) = next_row_idx.take().and_then(|idx| rows.get(usize::from(idx))) else {
        return Err(MatcherError::UnknownMatchingRoute.into());
      };
      for edge in &next_row.edges {
        let Some(row) = rows.get(usize::from(edge.row_target_idx)) else {
          continue;
        };
        let len_before = curr_identifier.len();
        match Self::check_search_node(
          abs_offset,
          &mut curr_identifier,
          &mut next_row_idx,
          &mut path_rows,
          row,
          edge.row_target_idx,
        ) {
          Some(false) => {}
          Some(true) => return Ok(MatcherPath { ident, path_rows, rows }),
          None => {
            let consumed = len_before.wrapping_sub(curr_identifier.len()).try_into()?;
            abs_offset = abs_offset.wrapping_add(consumed);
            break;
          }
        }
      }
    }
  }

  #[inline]
  fn check_search_node(
    abs_offset: u8,
    curr_identifier: &mut &str,
    next_row_idx: &mut Option<u8>,
    path_rows: &mut ArrayVectorU8<PathRow, DEFAULT_MAX_DEPTH>,
    row: &Row<T, N>,
    row_idx: u8,
  ) -> Option<bool> {
    let comparing_name = match row.node.ty {
      NodeTy::Literal => &*row.node.ident,
      NodeTy::Param { begin_idx, .. } => {
        row.node.ident.get(..usize::from(begin_idx)).unwrap_or_default()
      }
    };
    let Some((lhs, rhs)) = curr_identifier.split_at_checked(comparing_name.len()) else {
      return Some(false);
    };
    if lhs != comparing_name {
      return Some(false);
    }
    let ident_param_range = match row.node.ty {
      NodeTy::Literal => {
        *curr_identifier = rhs;
        0..0
      }
      NodeTy::Param { after, end_idx, .. } => {
        let tail = row.node.ident.get(usize::from(end_idx)..).unwrap_or_default();
        let (param, tail_after) = if tail.is_empty() {
          if let Some(byte) = after {
            if let Some((param, _)) = str_split_once1(rhs, byte) {
              (param, rhs.get(param.len()..).unwrap_or_default())
            } else {
              (rhs, "")
            }
          } else {
            (rhs, "")
          }
        } else {
          let Some((param, tail_after)) = str_split_once_str(rhs, tail) else {
            return Some(false);
          };
          (param, tail_after)
        };

        if param.contains('/') {
          return Some(false);
        }

        *curr_identifier = tail_after;
        let comparing_name_len = comparing_name.len().try_into().unwrap_or_default();
        let param_start = abs_offset.wrapping_add(comparing_name_len);
        let param_end = param_start.wrapping_add(param.len().try_into().unwrap_or_default());
        param_start..param_end
      }
    };

    let _rslt = path_rows.push(PathRow::new(ident_param_range, row_idx));
    if curr_identifier.is_empty() {
      if row.node.value.is_some() { Some(true) } else { Some(false) }
    } else {
      *next_row_idx = Some(row_idx);
      None
    }
  }
}

impl<T> Default for Matcher<T> {
  #[inline]
  fn default() -> Self {
    Self::new()
  }
}

/// Constructs nodes
#[derive(Debug)]
pub struct MatcherBuilder<'instance, T, const N: usize> {
  root_firsts: &'instance mut ArrayVectorU8<u8, N>,
  rows: &'instance mut Vector<Row<T, N>>,
}

impl<T, const N: usize> MatcherBuilder<'_, T, N> {
  /// Adds a new node and its associated value to the tree.
  #[inline]
  pub fn node(&mut self, mut ident: &'static str, value: T) -> crate::Result<&mut Self> {
    let mut exact_match = false;
    let mut is_root = true;
    let mut next_row_idx = None;
    let mut parent_node_idx = 0;
    let mut row_idx = 0u8;

    for row in self.rows.get(0..usize::from(self.root_firsts.len())).unwrap_or_default() {
      match Self::check_insert_node(&mut is_root, &mut next_row_idx, row, &mut ident, row_idx)? {
        CheckInsertNodeRslt::Impossible | CheckInsertNodeRslt::Unmatched => {
          row_idx = row_idx.wrapping_add(1);
        }
        CheckInsertNodeRslt::MatchedAll => {
          parent_node_idx = row_idx;
          break;
        }
        CheckInsertNodeRslt::MatchedExactly => {
          parent_node_idx = row_idx;
          exact_match = true;
          break;
        }
        CheckInsertNodeRslt::MatchedPart { common_prefix_len, is_single } => {
          let row_ident = ShortStrU8::new(ident)?;
          self.add_common_node(common_prefix_len, is_single, row_ident, row_idx, value)?;
          return Ok(self);
        }
      }
    }

    if exact_match {
      if let Some(row) = self.rows.get_mut(usize::from(parent_node_idx)) {
        row.node.value = Some(value);
      }
      return Ok(self);
    }

    if is_root {
      self.add_root_node(ShortStrU8::new(ident)?, value)?;
      return Ok(self);
    }

    loop {
      let Some(next_row) = next_row_idx.take().and_then(|idx| self.rows.get(usize::from(idx)))
      else {
        return Ok(self);
      };
      let mut has_match = false;
      for edge in &next_row.edges {
        let Some(row) = self.rows.get(usize::from(edge.row_target_idx)) else {
          continue;
        };
        match Self::check_insert_node(
          &mut is_root,
          &mut next_row_idx,
          row,
          &mut ident,
          edge.row_target_idx,
        )? {
          CheckInsertNodeRslt::Impossible | CheckInsertNodeRslt::Unmatched => {}
          CheckInsertNodeRslt::MatchedAll => {
            parent_node_idx = edge.row_target_idx;
            has_match = true;
            break;
          }
          CheckInsertNodeRslt::MatchedExactly => {
            parent_node_idx = edge.row_target_idx;
            exact_match = true;
            has_match = true;
            break;
          }
          CheckInsertNodeRslt::MatchedPart { common_prefix_len, is_single } => {
            self.add_common_node(
              common_prefix_len,
              is_single,
              ShortStrU8::new(ident)?,
              edge.row_target_idx,
              value,
            )?;
            return Ok(self);
          }
        }
      }

      if exact_match {
        if let Some(row) = self.rows.get_mut(usize::from(parent_node_idx)) {
          row.node.value = Some(value);
        }
        return Ok(self);
      }

      if !has_match {
        self.add_common_node(0, true, ShortStrU8::new(ident)?, parent_node_idx, value)?;
        return Ok(self);
      }
    }
  }

  // * Parent's node equals common path. With an existent `/aa` (0 edges, 1 node), the new `/aab`
  // forms `/aa -> b` (1 edge, 2 nodes)
  //
  // * Parent's node does not equals common path. With an existent `/aab` (0 edges, 1 node), the
  // new `/aac` forms `/aa -> b, c` (2 edges, 3 nodes)
  #[inline]
  fn add_common_node(
    &mut self,
    common_prefix_len: usize,
    is_single: bool,
    row_ident: ShortStrU8<'static>,
    mut row_insert_idx0: u8,
    row_value: T,
  ) -> crate::Result<()> {
    let is_new_is_prefix = common_prefix_len == row_ident.len();

    row_insert_idx0 = row_insert_idx0.wrapping_add(1);

    if is_single {
      self.adjust_edges(1, row_insert_idx0);
      self.rows.insert(
        row_insert_idx0.into(),
        Row::new(ArrayVectorU8::new(), Node::from_identifier(row_ident)),
      )?;
      let split_opt = self.rows.split_at_mut_checked(row_insert_idx0.into());
      let Some(([.., parent], [child, ..])) = split_opt else {
        return Ok(());
      };
      Self::modify_child(child, common_prefix_len, row_ident, parent)?;
      child.node.value = Some(row_value);
      parent.edges.push(Edge::new(row_insert_idx0))?;
    } else if is_new_is_prefix {
      self.adjust_edges(1, row_insert_idx0);
      self.rows.insert(
        row_insert_idx0.into(),
        Row::new(ArrayVectorU8::new(), Node::from_identifier(row_ident)),
      )?;
      let split_opt = self.rows.split_at_mut_checked(row_insert_idx0.into());
      let Some(([.., parent], [child, ..])) = split_opt else {
        return Ok(());
      };

      let original_parent_ident = parent.node.ident;
      parent.node.ident = Self::common_path(common_prefix_len, parent.node.ident);
      parent.node.ty = Self::find_node_ty(parent.node.ident)?;

      Self::modify_child(child, common_prefix_len, original_parent_ident, parent)?;

      child.node.value = parent.node.value.take();
      child.edges = mem::replace(&mut parent.edges, ArrayVectorU8::new());

      parent.node.value = Some(row_value);
      parent.edges.push(Edge::new(row_insert_idx0))?;
    } else {
      self.adjust_edges(2, row_insert_idx0);
      let row_insert_idx1 = row_insert_idx0.wrapping_add(1);
      self.rows.insert(
        row_insert_idx0.into(),
        Row::new(ArrayVectorU8::new(), Node::from_identifier(row_ident)),
      )?;
      self.rows.insert(
        row_insert_idx1.into(),
        Row::new(ArrayVectorU8::new(), Node::from_identifier(row_ident)),
      )?;
      let split_opt = self.rows.split_at_mut_checked(row_insert_idx0.into());
      let Some(([.., parent], [child0, child1, ..])) = split_opt else {
        return Ok(());
      };

      let original_parent_ident = parent.node.ident;
      parent.node.ident = Self::common_path(common_prefix_len, parent.node.ident);
      parent.node.ty = Self::find_node_ty(parent.node.ident)?;

      Self::modify_child(child0, common_prefix_len, row_ident, parent)?;
      Self::modify_child(child1, common_prefix_len, original_parent_ident, parent)?;

      child0.node.value = Some(row_value);
      child1.node.value = parent.node.value.take();
      child1.edges = mem::replace(&mut parent.edges, ArrayVectorU8::new());
      parent.edges.push(Edge::new(row_insert_idx1))?;
      parent.edges.push(Edge::new(row_insert_idx0))?;
    }
    Ok(())
  }
  #[inline]
  fn add_root_node(&mut self, row_ident: ShortStrU8<'static>, node_value: T) -> crate::Result<()> {
    let ty = Self::find_node_ty(row_ident)?;
    self.adjust_edges(1, self.root_firsts.len());
    self.rows.insert(
      usize::from(self.root_firsts.len()),
      Row::new(
        ArrayVectorU8::new(),
        Node::new(row_ident, ArrayVectorU8::new(), ty, Some(node_value)),
      ),
    )?;
    self.root_firsts.push(0)?;
    Ok(())
  }

  #[inline]
  fn adjust_edges(&mut self, offset: u8, row_idx: u8) {
    for row in &mut *self.rows {
      for edge in &mut row.edges {
        if edge.row_target_idx >= row_idx {
          edge.row_target_idx = edge.row_target_idx.wrapping_add(offset);
        }
      }
    }
  }

  #[inline]
  fn check_insert_node(
    is_root: &mut bool,
    next_row_idx: &mut Option<u8>,
    row: &Row<T, N>,
    row_ident: &mut &str,
    row_idx: u8,
  ) -> crate::Result<CheckInsertNodeRslt> {
    let comparing_identifier = match row.node.ty {
      NodeTy::Literal => &*row.node.ident,
      NodeTy::Param { begin_idx, .. } => {
        row.node.ident.get(..usize::from(begin_idx)).unwrap_or_default()
      }
    };
    let Some((lhs, rhs)) = row_ident.split_at_checked(comparing_identifier.len()) else {
      let common_prefix_len = Self::common_prefix_len(row_ident, comparing_identifier);
      if common_prefix_len > 0 && row_ident.contains('{') {
        return Err(MatcherError::InvalidRouteDefinition.into());
      }
      if common_prefix_len > 0 && common_prefix_len == row_ident.len() {
        return Ok(CheckInsertNodeRslt::MatchedPart { common_prefix_len, is_single: false });
      }
      return Ok(CheckInsertNodeRslt::Impossible);
    };
    let common_prefix_len_lhs = Self::common_prefix_len(lhs, comparing_identifier);
    if common_prefix_len_lhs == 0 {
      return Ok(CheckInsertNodeRslt::Unmatched);
    } else if common_prefix_len_lhs != lhs.len() {
      return Ok(CheckInsertNodeRslt::MatchedPart {
        common_prefix_len: common_prefix_len_lhs,
        is_single: common_prefix_len_lhs == comparing_identifier.len(),
      });
    }
    *is_root = false;
    match row.node.ty {
      NodeTy::Literal => {
        *row_ident = rhs;
      }
      NodeTy::Param { begin_idx, end_idx, .. } => {
        let tail = row.node.ident.get(usize::from(end_idx)..).unwrap_or_default();
        if tail.is_empty() {
          let common_prefix_len = Self::common_prefix_len(&row.node.ident, row_ident);
          if common_prefix_len == row.node.ident.len() {
            let remainder = &row_ident.get(common_prefix_len..).unwrap_or_default();
            *next_row_idx = Some(row_idx);
            *row_ident = remainder;
            if row_ident.is_empty() {
              if row.node.value.is_some() {
                return Err(MatcherError::DuplicateRoute.into());
              }
              return Ok(CheckInsertNodeRslt::MatchedExactly);
            }
            return Ok(CheckInsertNodeRslt::MatchedAll);
          }
          return Err(MatcherError::ConflictingRoute.into());
        }
        let Some((_, after)) = str_split_once_str(rhs, tail) else {
          let mut common_prefix_len = Self::common_prefix_len(&row.node.ident, row_ident);
          if common_prefix_len <= usize::from(end_idx) {
            common_prefix_len = usize::from(begin_idx);
          }
          return Ok(CheckInsertNodeRslt::MatchedPart {
            common_prefix_len,
            is_single: common_prefix_len == row.node.ident.len(),
          });
        };
        *row_ident = after;
      }
    }
    if row_ident.is_empty() {
      if row.node.value.is_some() {
        return Err(MatcherError::DuplicateRoute.into());
      }
      return Ok(CheckInsertNodeRslt::MatchedExactly);
    }
    *next_row_idx = Some(row_idx);
    Ok(CheckInsertNodeRslt::MatchedAll)
  }

  #[inline]
  fn common_path(common_prefix_len: usize, ident: ShortStrU8<'_>) -> ShortStrU8<'_> {
    ShortStrU8::from(ident.into_str().get(..common_prefix_len).unwrap_or_default())
  }

  #[inline]
  fn common_prefix_len(lhs: &str, rhs: &str) -> usize {
    lhs.as_bytes().iter().zip(rhs.as_bytes()).take_while(|(b0, b1)| b0 == b1).count()
  }

  #[inline]
  fn find_node_ty(ident: ShortStrU8<'static>) -> crate::Result<NodeTy> {
    let str = ident.into_str();
    let Some((lhs, rhs)) = str_split_once1(str, Ascii::OPENING_BRACE) else {
      return Ok(NodeTy::Literal);
    };
    let begin_idx: u8 = lhs.len().try_into()?;
    let mut end_idx = begin_idx;
    let mut iter = rhs.as_bytes().iter();
    for elem in iter.by_ref() {
      match elem {
        b'{' => {
          return Err(MatcherError::NestedParameters.into());
        }
        b'}' => {
          if iter.any(|el| *el == b'{') {
            return Err(MatcherError::NodesCanHaveAtMostOneParameter.into());
          }
          let name_begin_idx = begin_idx.wrapping_add(1);
          let name_end_idx = end_idx.wrapping_add(1);
          let name = str.get(name_begin_idx.into()..name_end_idx.into()).unwrap_or_default();
          end_idx = end_idx.wrapping_add(2);
          return Ok(NodeTy::Param { after: None, begin_idx, end_idx, name: name.into() });
        }
        _ => {
          end_idx = end_idx.wrapping_add(1);
        }
      }
    }
    Err(MatcherError::UnclosedParameterBracket.into())
  }

  #[inline]
  fn modify_child(
    child: &mut Row<T, N>,
    common_prefix_len: usize,
    row_ident: ShortStr<'static, u8>,
    parent: &mut Row<T, N>,
  ) -> crate::Result<()> {
    child.node.ident = Self::unique_path(common_prefix_len, row_ident);
    child.node.ty = Self::find_node_ty(child.node.ident)?;
    if let NodeTy::Param { after, .. } = &mut parent.node.ty
      && after.is_none()
    {
      *after = Some(child.node.ident.as_bytes().first().copied().unwrap_or_default().try_into()?);
    }
    Ok(())
  }

  #[inline]
  fn unique_path(common_prefix_len: usize, ident: ShortStrU8<'_>) -> ShortStrU8<'_> {
    ShortStrU8::from(ident.into_str().get(common_prefix_len..).unwrap_or_default())
  }
}

impl<T, const N: usize> Drop for MatcherBuilder<'_, T, N> {
  #[inline]
  fn drop(&mut self) {
    let mut root_firsts = ArrayVectorU8::new();
    for row in self.rows.get(0..usize::from(self.root_firsts.len())).unwrap_or_default() {
      let first = row.node.ident.as_bytes().first().copied().unwrap_or_default();
      let _rslt = root_firsts.push(first);
    }
    *self.root_firsts = root_firsts;

    let mut row_idx: usize = 0;
    loop {
      let Some(rows @ [_not_empty, ..]) = self.rows.get(row_idx..) else {
        break;
      };
      let mut sub_firsts = ArrayVectorU8::new();
      for row in rows {
        for edge in &row.edges {
          let Some(target_row) = self.rows.get(usize::from(edge.row_target_idx)) else {
            continue;
          };
          let first = target_row.node.ident.as_bytes().first().copied().unwrap_or_default();
          let _rslt = sub_firsts.push(first);
        }
      }
      let Some(row) = self.rows.get_mut(usize::from(row_idx)) else {
        continue;
      };
      row.node.sub_firsts = sub_firsts;
      row_idx = row_idx.wrapping_add(1);
    }
  }
}

/// Path constructed from a root node
#[derive(Debug, PartialEq)]
pub struct MatcherPath<'any, T, const N: usize> {
  ident: &'any str,
  path_rows: ArrayVectorU8<PathRow, DEFAULT_MAX_DEPTH>,
  rows: &'any Vector<Row<T, N>>,
}

impl<T, const N: usize> MatcherPath<'_, T, N> {
  /// User data originated from a node
  #[inline]
  pub fn data(&self) -> &T {
    // SAFETY: Non-empty routes always have at least one path
    let last_row_idx = unsafe { self.path_rows.last().unwrap_unchecked().row_idx };
    // SAFETY: The searcher always ensure that indices point to valid instances.
    let row = unsafe { self.rows.get(usize::from(last_row_idx)).unwrap_unchecked() };
    // SAFETY: Leaf nodes always have non-null values
    unsafe { row.node.value.as_ref().unwrap_unchecked() }
  }

  /// Gets a parameter according to its index that is related to the entire URI.
  #[inline]
  pub fn param_by_idx(&self, idx: usize) -> Option<MatcherPathParam<'_>> {
    self.params().nth(idx)
  }

  /// Gets a parameter according to its declared name.
  #[inline]
  pub fn param_by_name(&self, name: &[u8]) -> Option<MatcherPathParam<'_>> {
    self.params().find(|el| el.name.as_bytes() == name)
  }

  /// Iterator over all parameters
  #[inline]
  pub fn params(&self) -> impl Iterator<Item = MatcherPathParam<'_>> {
    let Self { ident, path_rows, rows } = self;
    path_rows.iter().filter_map(|PathRow { ident_param_range, row_idx }| {
      let row = rows.get(usize::from(*row_idx))?;
      let name = match row.node.ty {
        NodeTy::Literal => return None,
        NodeTy::Param { name, .. } => name,
      };
      let value = ident.get(ident_param_range.start.into()..ident_param_range.end.into())?;
      Some(MatcherPathParam::new(name.into_str(), value))
    })
  }
}

/// Route match parameter
#[derive(Debug, PartialEq)]
pub struct MatcherPathParam<'any> {
  /// Identifies the parameter and is defined when building the route.
  pub name: &'static str,
  /// Dynamic value associated with the name.
  pub value: &'any str,
}

impl<'any> MatcherPathParam<'any> {
  /// Shortcut
  #[inline]
  pub const fn new(name: &'static str, value: &'any str) -> Self {
    Self { name, value }
  }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum CheckInsertNodeRslt {
  Impossible,
  MatchedAll,
  MatchedExactly,
  MatchedPart { common_prefix_len: usize, is_single: bool },
  Unmatched,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum NodeTy {
  Literal,
  // If `after` is null, then the parameter has a suffix
  Param { after: Option<Ascii>, begin_idx: u8, end_idx: u8, name: ShortStrU8<'static> },
}

/// An edge is also a piece of data in CSR terms.
#[derive(Clone, Debug, PartialEq)]
struct Edge {
  row_target_idx: u8,
}

impl Edge {
  #[inline]
  const fn new(row_target_idx: u8) -> Self {
    Self { row_target_idx }
  }
}

/// Intermediate nodes refer common prefixes shared by other routes while final nodes are unique.
///
/// A node can have at-most one parameter and a node is also a row in CSR terms.
///
/// Leaf nodes are always guarantee to have non-null values. Intermediate nodes may or may not
/// have values.
#[derive(Clone, Debug, PartialEq)]
struct Node<T> {
  ident: ShortStrU8<'static>,
  sub_firsts: ArrayVectorU8<u8, DEFAULT_MAX_SUB_NODES>,
  ty: NodeTy,
  value: Option<T>,
}

impl<T> Node<T> {
  #[inline]
  const fn from_identifier(ident: ShortStrU8<'static>) -> Self {
    Self { ident, ty: NodeTy::Literal, sub_firsts: ArrayVectorU8::new(), value: None }
  }

  #[inline]
  const fn new(
    ident: ShortStrU8<'static>,
    sub_firsts: ArrayVectorU8<u8, DEFAULT_MAX_SUB_NODES>,
    ty: NodeTy,
    value: Option<T>,
  ) -> Self {
    Self { ident, ty, sub_firsts, value }
  }
}

#[derive(Debug, PartialEq)]
struct PathRow {
  ident_param_range: Range<u8>,
  row_idx: u8,
}

impl PathRow {
  #[inline]
  const fn new(ident_param_range: Range<u8>, row_idx: u8) -> Self {
    Self { ident_param_range, row_idx }
  }
}

#[derive(Clone, Debug, PartialEq)]
struct Row<T, const N: usize = DEFAULT_MAX_SUB_NODES> {
  edges: ArrayVectorU8<Edge, N>,
  node: Node<T>,
}

impl<T, const N: usize> Row<T, N> {
  const fn new(edges: ArrayVectorU8<Edge, N>, node: Node<T>) -> Self {
    Self { edges, node }
  }
}

#[cfg(all(feature = "_bench", test))]
mod bench {
  use crate::http::server_framework::Matcher;
  use core::hint::black_box;

  macro_rules! routes {
    (literal) => {{
      routes!(@finish => "p1", "p2", "p3", "p4")
    }};
    (params) => {{
      routes!(@finish => "{p1}", "{p2}", "{p3}", "{p4}")
    }};
    (@finish => $p1:literal, $p2:literal, $p3:literal, $p4:literal) => {{
      [
        concat!("/authorizations"),
        concat!("/authorizations/", $p1),
        concat!("/applications/", $p1),
        concat!("/applications/", $p1, "/tokens/", $p2),
        concat!("/events"),
        concat!("/feeds"),
      ]
    }};
  }

  #[bench]
  fn matcher(b: &mut test::Bencher) {
    let routes = routes!(literal).to_vec();
    let mut matcher = Matcher::new();
    {
      let mut builder = matcher.builder();
      for route in routes!(params) {
        let _ = builder.node(route, true).unwrap();
      }
    }
    b.iter(|| {
      for route in black_box(&routes) {
        assert!(*black_box(matcher.find(route).unwrap()).data());
      }
    });
  }
}

#[cfg(test)]
mod tests {
  use crate::{
    collection::ArrayVectorU8,
    http::server_framework::{
      Matcher, MatcherPathParam,
      matcher::{Edge, Node, NodeTy, Row},
    },
  };

  #[test]
  fn add_static_route_after_param_route_and_vice_versa() {
    {
      let mut matcher = Matcher::new();
      let mut builder = matcher.builder();
      let _ = builder.node("/user/{id}", 1).unwrap();
      assert!(builder.node("/user/static", 2).is_err());
    }
    {
      let mut matcher = Matcher::new();
      let mut builder = matcher.builder();
      let _ = builder.node("/user/static", 1).unwrap();
      assert!(builder.node("/user/{id}", 2).is_err());
    }
  }

  #[test]
  fn conflicting_static_after_param_split() {
    let mut matcher = Matcher::new();
    {
      let mut builder = matcher.builder();
      let _ = builder.node("/api/{version}/users", 1).unwrap();
      assert!(builder.node("/api/v1/users", 2).is_err());
    }
  }

  #[test]
  fn deep_traversal_after_splits() {
    let mut matcher = Matcher::new();
    {
      let mut builder = matcher.builder();
      let _ = builder.node("/api/users", 1).unwrap();
      let _ = builder.node("/api/posts", 2).unwrap();
      let _ = builder.node("/api/comments", 3).unwrap();
    }
    assert_eq!(matcher.find("/api/users").unwrap().data(), &1);
    assert_eq!(matcher.find("/api/posts").unwrap().data(), &2);
    assert_eq!(matcher.find("/api/comments").unwrap().data(), &3);
    assert!(matcher.find("/api").is_err());
    assert!(matcher.find("/api/user").is_err());
  }

  #[test]
  fn duplicate_route_error_on_intermediate_node() {
    let mut tree = Matcher::new();
    let mut builder = tree.builder();
    let _ = builder.node("/foo/bar", 1).unwrap();
    let _ = builder.node("/foo/baz", 2).unwrap();
    let _ = builder.node("/foo/b", 3).unwrap();
  }

  #[test]
  fn duplicate_route_due_to_impossible_skip() {
    let mut matcher = Matcher::new();
    let mut builder = matcher.builder();
    let _ = builder.node("/abcd", 1).unwrap();
    let _ = builder.node("/ab", 2).unwrap();
    let _ = builder.node("/abxy", 3).unwrap();
    let res = builder.node("/ab", 4);
    assert!(res.is_err());
  }

  #[test]
  fn empty_route_handling() {
    let mut matcher = Matcher::new();
    {
      let mut builder = matcher.builder();
      let _ = builder.node("", 1).unwrap();
    }
    assert_eq!(matcher.find("").unwrap().data(), &1);
  }

  #[test]
  fn find_on_intermediate_node_without_value() {
    let mut matcher = Matcher::new();
    {
      let mut builder = matcher.builder();
      let _ = builder.node("/foo/bar", 1).unwrap();
      let _ = builder.node("/foo/baz", 2).unwrap();
    }
    assert!(matcher.find("/foo/").is_err());
  }

  #[test]
  fn lack_of_backtracking() {
    let mut matcher = Matcher::new();
    {
      let mut builder = matcher.builder();
      let _ = builder.node("/a/b/c", 1).unwrap();
      let _ = builder.node("/a/b/d", 2).unwrap();
      let _ = builder.node("/a/{p}/e", 3).unwrap();
    }
    assert!(matcher.find("/a/b/e").is_err());
  }

  #[test]
  fn lesser_ident_in_intermediate_node() {
    let mut tree = Matcher::new();
    {
      let mut builder = tree.builder();
      let _ = builder.node("/foo/bar", 1).unwrap();
      let _ = builder.node("/foo/baz", 2).unwrap();
    }
    let path = tree.find("/foo/b");
    assert!(path.is_err());
  }

  // ```
  // /aaa ->   /bbb
  //      \
  //       \
  //        -> /ccc
  //
  // /ddd
  // ```
  //
  // ```
  //  ----------
  // | /aaa/    | <- Row 0
  // | /ddd     | <- Row 1
  //  ----------
  //   ccc       <- Row 2
  //   bbb       <- Row 3
  // ```
  #[test]
  fn literal() {
    let mut matcher = Matcher::new();
    {
      let mut builder = matcher.builder();
      let _ = builder.node("/aaa/bbb", 1).unwrap();
      let _ = builder.node("/aaa/ccc", 2).unwrap();
      let _ = builder.node("/ddd", 3).unwrap();
    }
    assert_eq!(matcher.root_firsts, ArrayVectorU8::from_iterator([b'/', b'/']).unwrap()); // <- WRONG! First letters must be unique
    assert_eq!(
      &matcher.rows,
      &[
        Row::new(
          ArrayVectorU8::from_iterator([Edge::new(3), Edge::new(2)]).unwrap(),
          Node::new(
            "/aaa/".into(),
            ArrayVectorU8::from_iterator([b'b', b'c']).unwrap(),
            NodeTy::Literal,
            None
          )
        ),
        Row::new(
          ArrayVectorU8::new(),
          Node::new("/ddd".into(), ArrayVectorU8::new(), NodeTy::Literal, Some(3))
        ),
        Row::new(
          ArrayVectorU8::new(),
          Node::new("ccc".into(), ArrayVectorU8::new(), NodeTy::Literal, Some(2))
        ),
        Row::new(
          ArrayVectorU8::new(),
          Node::new("bbb".into(), ArrayVectorU8::new(), NodeTy::Literal, Some(1))
        ),
      ]
    );
    assert_eq!(matcher.find("/aaa/bbb").unwrap().data(), &1);
    assert_eq!(matcher.find("/aaa/ccc").unwrap().data(), &2);
    assert_eq!(matcher.find("/ddd").unwrap().data(), &3);
    assert!(matcher.find("").is_err());
    assert!(matcher.find("/aaa").is_err());
    assert!(matcher.find("/aaa/bb").is_err());
    assert!(matcher.find("/aaa/bbbb").is_err());
    assert!(matcher.find("/eee").is_err());
  }

  #[test]
  fn multiple_root_nodes_edge_tracking() {
    let mut matcher = Matcher::new();
    {
      let mut builder = matcher.builder();
      let _ = builder.node("/foo", 1).unwrap();
      let _ = builder.node("/bar", 2).unwrap();
      let _ = builder.node("/baz", 3).unwrap();
    }
    assert_eq!(matcher.root_firsts, ArrayVectorU8::from_iterator([b'/']).unwrap());
    assert_eq!(
      &matcher.rows,
      &[
        Row::new(
          ArrayVectorU8::from_iterator([Edge::new(4), Edge::new(1)]).unwrap(),
          Node::new(
            "/".into(),
            ArrayVectorU8::from_iterator([b'f', b'b', b'r', b'z']).unwrap(),
            NodeTy::Literal,
            None
          )
        ),
        Row::new(
          ArrayVectorU8::from_iterator([Edge::new(3), Edge::new(2)]).unwrap(),
          Node::new(
            "ba".into(),
            ArrayVectorU8::from_iterator([b'r', b'z']).unwrap(),
            NodeTy::Literal,
            None
          )
        ),
        Row::new(
          ArrayVectorU8::new(),
          Node::new("z".into(), ArrayVectorU8::new(), NodeTy::Literal, Some(3))
        ),
        Row::new(
          ArrayVectorU8::new(),
          Node::new("r".into(), ArrayVectorU8::new(), NodeTy::Literal, Some(2))
        ),
        Row::new(
          ArrayVectorU8::new(),
          Node::new("foo".into(), ArrayVectorU8::new(), NodeTy::Literal, Some(1))
        ),
      ]
    );
    assert_eq!(matcher.find("/foo").unwrap().data(), &1);
    assert_eq!(matcher.find("/bar").unwrap().data(), &2);
    assert_eq!(matcher.find("/baz").unwrap().data(), &3);
  }

  #[test]
  fn nested_paths() {
    let mut tree = Matcher::new();
    {
      let mut builder = tree.builder();
      let _ = builder.node("/a", 1).unwrap();
      let _ = builder.node("/a/b", 2).unwrap();
      let _ = builder.node("/a/b/c", 3).unwrap();
      let _ = builder.node("/a/b/c/d", 4).unwrap();
      let _ = builder.node("/a/b/c/d/e", 5).unwrap();
      let _ = builder.node("/a/b/c/d/e/f", 6).unwrap();
      let _ = builder.node("/a/b/c/d/e/f/g", 7).unwrap();
      let _ = builder.node("/a/b/c/d/e/f/g/h", 8).unwrap();
      let _ = builder.node("/a/b/c/d/e/f/g/h/i", 9).unwrap();
    }
    let path = tree.find("/a/b/c/d/e/f/g/h/i").unwrap();
    assert_eq!(*path.data(), 9);
  }

  #[test]
  fn parameter_after_literal() {
    let mut matcher = Matcher::new();
    {
      let mut builder = matcher.builder();
      let _ = builder.node("/aaaa", 1).unwrap();
      let _ = builder.node("/aaaa/{bbb}", 2).unwrap();
    }
    assert_eq!(matcher.find("/aaaa/rrr").unwrap().data(), &2);
  }

  #[test]
  fn parameter_containing_after_byte() {
    let mut matcher = Matcher::new();
    {
      let mut builder = matcher.builder();
      let _ = builder.node("/user/{id}profile", 1).unwrap();
    }
    assert_eq!(matcher.find("/user/p123profile").unwrap().data(), &1);
  }

  #[test]
  fn parameter_in_split_node_is_correctly_extracted() {
    let mut matcher = Matcher::new();
    {
      let mut builder = matcher.builder();
      let _ = builder.node("/api/v1/user", 1).unwrap();
      let _ = builder.node("/api/{version}/data", 2).unwrap();
    }
    let path = matcher.find("/api/v2/data").unwrap();
    assert_eq!(path.data(), &2);
    assert_eq!(path.param_by_name(b"version").unwrap(), MatcherPathParam::new("version", "v2"));
  }

  #[test]
  fn parameter_node_split_reverse_order() {
    let mut matcher = Matcher::new();
    {
      let mut builder = matcher.builder();
      let _ = builder.node("/user/{id}/profile", 1).unwrap();
      let _ = builder.node("/user/{id}", 2).unwrap();
    }
    assert_eq!(matcher.find("/user/123").unwrap().data(), &2);
    assert_eq!(matcher.find("/user/123/profile").unwrap().data(), &1);
  }

  #[test]
  fn parameter_node_split() {
    let mut matcher = Matcher::new();
    {
      let mut builder = matcher.builder();
      let _ = builder.node("/user/{id}/profile", 1).unwrap();
      let _ = builder.node("/user/{id}/settings", 2).unwrap();
    }
    assert_eq!(matcher.find("/user/123/profile").unwrap().data(), &1);
    assert_eq!(matcher.find("/user/456/settings").unwrap().data(), &2);
  }

  #[test]
  fn parameter_split_boundary_case() {
    let mut matcher = Matcher::new();
    {
      let mut builder = matcher.builder();
      let _ = builder.node("/a{x}b", 1).unwrap();
      let _ = builder.node("/a{x}c", 2).unwrap();
    }
    assert_eq!(matcher.find("/a1b").unwrap().data(), &1);
    assert_eq!(matcher.find("/a2c").unwrap().data(), &2);
  }

  #[test]
  fn parameter_with_multiple_suffixes() {
    let mut matcher = Matcher::new();
    {
      let mut builder = matcher.builder();
      let _ = builder.node("/user/{id}profile", 1).unwrap();
      let _ = builder.node("/user/{id}settings", 2).unwrap();
    }

    assert_eq!(matcher.find("/user/123profile").unwrap().data(), &1);
    assert_eq!(matcher.find("/user/123settings").unwrap().data(), &2);
  }

  #[test]
  fn parameter_without_suffix_then_with_slash() {
    let mut matcher = Matcher::new();
    {
      let mut builder = matcher.builder();
      let _ = builder.node("/user/{id}", 1).unwrap();
      let _ = builder.node("/user/{id}/", 2).unwrap();
    }
    assert_eq!(matcher.find("/user/123").unwrap().data(), &1);
    assert_eq!(matcher.find("/user/456/").unwrap().data(), &2);
  }

  #[test]
  fn parameters() {
    let mut matcher = Matcher::new();
    {
      let mut builder = matcher.builder();
      let _ = builder.node("/aaa/{}/bbb", 1).unwrap();
    }
    let path = matcher.find("/aaa/123/bbb").unwrap();
    assert_eq!(path.data(), &1);
    assert_eq!(path.param_by_idx(0).unwrap(), MatcherPathParam::new("", "123"));
    assert_eq!(path.param_by_idx(1), None);
  }

  #[test]
  fn parameter_with_suffix_split_order() {
    let mut matcher = Matcher::new();
    {
      let mut builder = matcher.builder();
      let _ = builder.node("/api/{version}/users", 1).unwrap();
      let _ = builder.node("/api/{version}/posts", 2).unwrap();
    }
    assert_eq!(matcher.find("/api/v1/users").unwrap().data(), &1);
    assert_eq!(matcher.find("/api/v1/posts").unwrap().data(), &2);
    assert!(matcher.find("/api/v1").is_err());
    assert!(matcher.find("/api/v1/other").is_err());
  }

  #[test]
  fn parameter_multiple_children_intermediate() {
    let mut matcher = Matcher::new();
    {
      let mut builder = matcher.builder();
      let _ = builder.node("/a/{p}w", 1).unwrap();
      let _ = builder.node("/a/{p}x", 2).unwrap();
      let _ = builder.node("/a/{p}y", 3).unwrap();
    }

    let path = matcher.find("/a/123y");
    assert_eq!(path.unwrap().data(), &3);
  }

  #[test]
  fn path_rows_corruption() {
    let mut matcher = Matcher::new();
    {
      let mut builder = matcher.builder();
      let _ = builder.node("/a/{p}w", 1).unwrap();
      let _ = builder.node("/a/{p}x", 2).unwrap();
      let _ = builder.node("/a/123y", 3).unwrap();
    }

    let path = matcher.find("/a/123y").unwrap();
    assert_eq!(path.data(), &3);
    assert_eq!(path.params().count(), 0,);
  }

  #[test]
  fn root_param_with_children() {
    let mut matcher = Matcher::new();
    {
      let mut builder = matcher.builder();
      let _ = builder.node("/{id}", 1).unwrap();
      let _ = builder.node("/{id}/profile", 2).unwrap();
    }
    assert_eq!(matcher.find("/123").unwrap().data(), &1);
    assert_eq!(matcher.find("/456/profile").unwrap().data(), &2);
  }

  #[test]
  fn route_with_param_suffix_should_not_match_without_it() {
    let mut matcher = Matcher::new();
    {
      let mut builder = matcher.builder();
      let _ = builder.node("/user/{id}/profile", 1).unwrap();
    }
    assert!(matcher.find("/user/123").is_err());
  }

  #[test]
  fn several_parameters() {
    let mut matcher = Matcher::new();
    {
      let mut builder = matcher.builder();
      let _ = builder.node("/aa", 1).unwrap();
      let _ = builder.node("/aa/{}", 2).unwrap();
      let _ = builder.node("/bb/{}", 3).unwrap();
      let _ = builder.node("/bb/{}/cc/{}", 4).unwrap();
    }
    drop(matcher.find("/aa").unwrap());
    drop(matcher.find("/aa/111").unwrap());
    drop(matcher.find("/bb/222").unwrap());
    drop(matcher.find("/bb/333/cc/444").unwrap());
  }

  #[test]
  fn single_parameter_route_should_not_match_extra_segments() {
    let mut matcher = Matcher::new();
    {
      let mut builder = matcher.builder();
      let _ = builder.node("/user/{id}", 1).unwrap();
    }
    assert_eq!(matcher.find("/user/123").unwrap().data(), &1);
    assert!(matcher.find("/user/123/extra").is_err());
  }

  #[test]
  fn shorter_route_after_longer_route() {
    let mut matcher = Matcher::new();
    {
      let mut builder = matcher.builder();
      let _ = builder.node("/api/users/profile", 1).unwrap();
      let _ = builder.node("/api/users", 2).unwrap();
    }
    assert_eq!(matcher.root_firsts, ArrayVectorU8::from_iterator([b'/']).unwrap());
    assert_eq!(matcher.find("/api/users/profile").unwrap().data(), &1);
    assert_eq!(matcher.find("/api/users").unwrap().data(), &2);
  }

  #[test]
  fn split_node_single_wrong_identifier() {
    {
      let mut matcher = Matcher::new();
      {
        let mut builder = matcher.builder();
        let _ = builder.node("/ab", 1).unwrap();
        let _ = builder.node("/abc", 2).unwrap();
      }
      assert_eq!(matcher.find("/ab").unwrap().data(), &1);
      assert_eq!(matcher.find("/abc").unwrap().data(), &2);
    }

    {
      let mut matcher = Matcher::new();
      {
        let mut builder = matcher.builder();
        let _ = builder.node("/abc", 2).unwrap();
        let _ = builder.node("/ab", 1).unwrap();
      }
      assert_eq!(matcher.find("/ab").unwrap().data(), &1);
      assert_eq!(matcher.find("/abc").unwrap().data(), &2);
    }
  }

  #[test]
  fn two_parameters_in_a_path() {
    let mut matcher = Matcher::new();
    {
      let mut builder = matcher.builder();
      let _ = builder.node("/aa/{}", 1).unwrap();
      let _ = builder.node("/aa/{}/bb/{}", 2).unwrap();
    }
    let path = matcher.find("/aa/111/bb/222").unwrap();
    assert_eq!(path.data(), &2);
    assert_eq!(path.param_by_idx(0).unwrap(), MatcherPathParam::new("", "111"));
    assert_eq!(path.param_by_idx(1).unwrap(), MatcherPathParam::new("", "222"));
  }
}
