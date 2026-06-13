#[cfg(all(feature = "_bench", test))]
mod bench;
#[cfg(test)]
mod tests;

use crate::{
  collection::{ArrayVectorU8, ShortBoxStrU8, ShortSliceU8, ShortStrU8, Vector},
  http::{DEFAULT_MAX_CHILDREN, DEFAULT_MAX_DEPTH},
  misc::{bytes_pos1, from_utf8_basic},
};
use core::{
  hint::{cold_path, unreachable_unchecked},
  mem,
  range::Range,
  str,
};

/// Matcher Error
#[derive(Clone, Copy, Debug)]
pub enum RouterError {
  /// Route already exists
  AddErrDuplicate,
  /// Provided adding route is empty
  AddErrEmpty,
  /// Route must start with a forward slash or a parameter bracket
  AddErrInvalidStart,
  /// For example, `/{foo}-{bar}`.
  AddErrMultipleRouteParameters,
  /// Static route parts contain non-ASCII characters
  AddErrNonAsciiLiteral,
  /// Parameter names contain non-ASCII characters
  AddErrNonAsciiParam,
  /// Maximum number of routes or internal indices exceeded
  AddErrOverflow,
  /// Parameters must be followed by a slash or the end of the string
  AddErrParamSuffix,
  /// Parameter bracket `{` was not closed with `}`
  AddErrUnclosedParameterBracket,
  /// The inner contents of the instance is empty because no routes were added or `MD` is too
  /// short to accommodate the maximum depth length.
  FindErrEmpty,
  /// Route does not exist
  FindErrMismatch,
  /// Input string or parameter length exceeds internal limits
  FindErrOverflow,
}

/// Decomposes a series of prefixed routes into common nodes to allow fast comparisons with
/// other dynamic routes.
///
/// * `/hey_{anything}/lyrics`: `/hey_/lyrics`, `/hey_you/lyrics`, `/hey_you_what_is_this_now/lyrics`
#[derive(Clone, Debug)]
pub struct Router<T, const MC: usize = DEFAULT_MAX_CHILDREN, const MD: usize = DEFAULT_MAX_DEPTH> {
  rows: Vector<Row<T, MC>>,
}

impl<T, const MC: usize, const MD: usize> Router<T, MC, MD> {
  /// Empty instance
  #[inline]
  pub const fn new() -> Self {
    Self { rows: Vector::new() }
  }
}

impl<T, const MC: usize, const MD: usize> Router<T, MC, MD> {
  /// See [`MatcherBuilder`].
  #[inline]
  pub fn builder(&mut self) -> RouterBuilder<'_, T, MC, MD> {
    self.rows.clear();
    RouterBuilder { rows: &mut self.rows }
  }

  /// Tries to find registered path that corresponds to `route`.
  #[inline]
  pub fn find<'this, 'data, 'rslt>(
    &'this self,
    route_str: &'data str,
  ) -> crate::Result<RouterMatch<'rslt, T, MC, MD>>
  where
    'data: 'rslt,
    'this: 'rslt,
  {
    let Self { rows } = self;
    let route_short_str = ShortStrU8::new(route_str)?;
    let route = route_short_str.into_short_slice();
    let route_len = route_short_str.len();
    let curr_route = &mut route_short_str.into_str().as_bytes();
    let mut edges;
    let mut path_rows = ArrayVectorU8::new();

    let Some(first_row) = rows.first() else {
      cold_path();
      return Err(RouterError::FindErrEmpty.into());
    };
    match Self::check_row(curr_route, &mut path_rows, route_len, first_row, 0) {
      CheckSearchRowRslt::CompleteMatch(value) => {
        return Ok(RouterMatch { rmpi: path_rows, route, rows, value });
      }
      CheckSearchRowRslt::IncompleteMatch(local_edges) => edges = local_edges,
      CheckSearchRowRslt::Mismatch => return Err(RouterError::FindErrMismatch.into()),
    }

    loop {
      let [statics @ .., last] = edges.as_slice() else {
        return Err(RouterError::FindErrMismatch.into());
      };
      let curr_ident_first = curr_route.first().copied();
      let last_edge_has_param = last.first_byte.is_none();
      if last_edge_has_param {
        if let Some(edge) = statics.iter().find(|el| el.first_byte == curr_ident_first) {
          match Self::check_edge(*edge, &mut edges, curr_route, &mut path_rows, route_len, rows) {
            CheckSearchRowRslt::CompleteMatch(value) => {
              return Ok(RouterMatch { rmpi: path_rows, route, rows, value });
            }
            CheckSearchRowRslt::IncompleteMatch(_) => continue,
            CheckSearchRowRslt::Mismatch => {}
          }
        }
        match Self::check_edge(*last, &mut edges, curr_route, &mut path_rows, route_len, rows) {
          CheckSearchRowRslt::CompleteMatch(value) => {
            return Ok(RouterMatch { rmpi: path_rows, route, rows, value });
          }
          CheckSearchRowRslt::IncompleteMatch(_) => {}
          CheckSearchRowRslt::Mismatch => cold_path(),
        }
      } else {
        let Some(edge) = edges.iter().find(|el| el.first_byte == curr_ident_first) else {
          return Err(RouterError::FindErrMismatch.into());
        };
        match Self::check_edge(*edge, &mut edges, curr_route, &mut path_rows, route_len, rows) {
          CheckSearchRowRslt::CompleteMatch(value) => {
            return Ok(RouterMatch { rmpi: path_rows, route, rows, value });
          }
          CheckSearchRowRslt::IncompleteMatch(_) => {}
          CheckSearchRowRslt::Mismatch => return Err(RouterError::FindErrMismatch.into()),
        }
      }
    }
  }

  #[inline(always)]
  fn check_edge<'this>(
    edge: Edge,
    edges: &mut &'this ArrayVectorU8<Edge, MC>,
    curr_route: &mut &'this [u8],
    path_rows: &mut ArrayVectorU8<RouterMatchParamIndices, MD>,
    route_len: u8,
    rows: &'this [Row<T, MC>],
  ) -> CheckSearchRowRslt<'this, T, MC> {
    let row_idx = edge.row_target_idx;
    // SAFETY: constructed edges always point to valid row indices
    let row = unsafe { rows.get(usize::from(row_idx)).unwrap_unchecked() };
    match Self::check_row(curr_route, path_rows, route_len, row, row_idx) {
      CheckSearchRowRslt::CompleteMatch(value) => CheckSearchRowRslt::CompleteMatch(value),
      CheckSearchRowRslt::IncompleteMatch(local_edges) => {
        *edges = local_edges;
        CheckSearchRowRslt::IncompleteMatch(local_edges)
      }
      CheckSearchRowRslt::Mismatch => CheckSearchRowRslt::Mismatch,
    }
  }

  #[inline(always)]
  fn check_row<'this>(
    curr_route: &mut &'this [u8],
    path_rows: &mut ArrayVectorU8<RouterMatchParamIndices, MD>,
    route_len: u8,
    row: &'this Row<T, MC>,
    row_idx: u8,
  ) -> CheckSearchRowRslt<'this, T, MC> {
    match row.ty {
      RowTy::Literal => {
        if let Some(rest) = curr_route.strip_prefix(row.route.as_bytes()) {
          *curr_route = rest;
        } else {
          return CheckSearchRowRslt::Mismatch;
        }
      }
      #[expect(
        clippy::as_conversions,
        clippy::cast_possible_truncation,
        reason = "full routes are limited by 255, subroutes are smaller"
      )]
      RowTy::Param => {
        // For some reason `memchr` degrades the performance if `target-cpu=native`.
        let (param, rest) = if let Some(param_end_idx) = bytes_pos1(*curr_route, b'/') {
          // SAFETY: the index has just been checked
          unsafe { curr_route.split_at_checked(param_end_idx).unwrap_unchecked() }
        } else {
          (*curr_route, &[][..])
        };
        let begin_idx = route_len.wrapping_sub(curr_route.len() as u8);
        let end_idx = begin_idx.wrapping_add(param.len() as u8);
        // SAFETY: The Drop implementation of `MatcherBuilder` guarantees that `MD` will never
        //         be lesser than the maximum depth of this instance.
        unsafe {
          path_rows
            .push(RouterMatchParamIndices::new((begin_idx..end_idx).into(), row_idx))
            .unwrap_unchecked();
        }
        *curr_route = rest;
      }
    }
    if let ([], Some(value)) = (curr_route, &row.value) {
      return CheckSearchRowRslt::CompleteMatch(value);
    }
    CheckSearchRowRslt::IncompleteMatch(&row.edges)
  }
}

impl<T> Default for Router<T> {
  #[inline]
  fn default() -> Self {
    Self::new()
  }
}

/// Constructs routes
#[derive(Debug)]
pub struct RouterBuilder<
  'instance,
  T,
  const MC: usize = DEFAULT_MAX_CHILDREN,
  const MD: usize = DEFAULT_MAX_DEPTH,
> {
  rows: &'instance mut Vector<Row<T, MC>>,
}

impl<T, const MC: usize, const MD: usize> RouterBuilder<'_, T, MC, MD> {
  /// Adds a new route and its associated value.
  #[inline]
  pub fn add(&mut self, route: &ShortBoxStrU8, value: T) -> crate::Result<&mut Self> {
    let mut curr_bytes_idx = 0;
    loop {
      let (mut local_route, local_ty) = Self::manage_row_ty(route.as_bytes(), &mut curr_bytes_idx)?;
      let final_row_idx = Self::add_local_route(&mut local_route, local_ty, self.rows)?;
      let should_stop = curr_bytes_idx >= usize::from(route.len());
      if should_stop {
        if let Some(row) = self.rows.get_mut(usize::from(final_row_idx)) {
          if row.value.is_some() {
            return Err(RouterError::AddErrDuplicate.into());
          }
          row.value = Some(value);
        }
        return Ok(self);
      }
    }
  }

  #[inline]
  fn add_local_route(
    local_route: &mut &[u8],
    local_ty: RowTy,
    rows: &mut Vector<Row<T, MC>>,
  ) -> crate::Result<u8> {
    let mut row_idx = 0;

    let mut edges = if let Some(row) = rows.first() {
      let crr = Self::compare_row(local_route, row)?;
      match crr {
        CompareRowRslt::Finished => {
          return Ok(row_idx);
        }
        CompareRowRslt::RowFull(edges) => edges,
        CompareRowRslt::RowPartial { common_prefix_len, is_single } => {
          Self::add_row((local_route, local_ty), common_prefix_len, is_single, &mut row_idx, rows)?;
          return Ok(row_idx);
        }
        // Unreachable because all routes must start with '/'
        CompareRowRslt::Unmatched => return Ok(row_idx),
      }
    } else {
      let row_route = from_utf8_basic(local_route)?.try_into()?;
      rows.push(Row::new(ArrayVectorU8::new(), row_route, local_ty, None))?;
      return Ok(row_idx);
    };

    'outer: loop {
      'inner: for edge in edges {
        let Some(row) = rows.get_mut(usize::from(edge.row_target_idx)) else {
          break 'inner;
        };
        let crr = Self::compare_row(local_route, row)?;
        match crr {
          CompareRowRslt::Finished => {
            return Ok(edge.row_target_idx);
          }
          CompareRowRslt::RowFull(local_edges) => {
            row_idx = edge.row_target_idx;
            edges = local_edges;
            continue 'outer;
          }
          CompareRowRslt::RowPartial { common_prefix_len, is_single } => {
            row_idx = edge.row_target_idx;
            Self::add_row(
              (local_route, local_ty),
              common_prefix_len,
              is_single,
              &mut row_idx,
              rows,
            )?;
            return Ok(row_idx);
          }
          CompareRowRslt::Unmatched => {}
        }
      }
      Self::add_row((local_route, local_ty), 0, true, &mut row_idx, rows)?;
      return Ok(row_idx);
    }
  }

  #[inline]
  fn add_row(
    (route, ty): (&[u8], RowTy),
    common_prefix_len: usize,
    is_single: bool,
    row_idx: &mut u8,
    rows: &mut Vector<Row<T, MC>>,
  ) -> crate::Result<()> {
    let route_str: ShortBoxStrU8 = route.try_into()?;
    let parent_idx = usize::from(*row_idx);
    if is_single {
      let child_idx = u8::try_from(rows.len()).map_err(|_err| RouterError::AddErrOverflow)?;
      if route_str.is_empty() {
        let Some(parent) = rows.get_mut(parent_idx) else {
          return Ok(());
        };
        let child_edges = mem::take(&mut parent.edges);
        let child_route = unique_route(common_prefix_len, &parent.route)?;
        let child_value = parent.value.take();
        parent.edges.push(Edge::new(None, child_idx))?;
        parent.route = common_route(common_prefix_len, &parent.route)?;
        parent.ty = ty;
        rows.push(Row::new(child_edges, child_route, ty, child_value))?;
      } else {
        let Some(parent) = rows.get_mut(parent_idx) else {
          return Ok(());
        };
        parent.edges.push(Edge::new(None, child_idx))?;
        rows.push(Row::new(ArrayVectorU8::new(), route_str, ty, None))?;
        *row_idx = child_idx;
      }
    } else {
      let child0_idx = u8::try_from(rows.len()).map_err(|_err| RouterError::AddErrOverflow)?;
      let child1_idx = child0_idx.checked_add(1).ok_or(RouterError::AddErrOverflow)?;
      let Some(parent) = rows.get_mut(parent_idx) else {
        return Ok(());
      };
      let child0_edges = mem::take(&mut parent.edges);
      let child0_route = unique_route(common_prefix_len, &parent.route)?;
      let child0_value = parent.value.take();
      parent.edges.push(Edge::new(None, child0_idx))?;
      parent.edges.push(Edge::new(None, child1_idx))?;
      parent.route = common_route(common_prefix_len, &parent.route)?;
      rows.push(Row::new(child0_edges, child0_route, RowTy::Literal, child0_value))?;
      rows.push(Row::new(ArrayVectorU8::new(), route_str, ty, None))?;
      *row_idx = child1_idx;
    }

    Ok(())
  }

  #[inline]
  fn common_prefix_len(lhs: &[u8], rhs: &[u8]) -> usize {
    lhs.iter().zip(rhs).take_while(|(b0, b1)| b0 == b1).count()
  }

  #[inline]
  fn compare_row(route: &mut &[u8], row: &Row<T, MC>) -> crate::Result<CompareRowRslt<MC>> {
    if row.ty == RowTy::Param {
      if route.first() == Some(&b'{')
        && let Some(idx) = bytes_pos1(*route, b'}')
        && let Some((lhs, rhs)) = route.split_at_checked(idx.wrapping_add(1))
      {
        if lhs == row.route.as_bytes() {
          *route = rhs;
          if rhs.is_empty() {
            return Ok(CompareRowRslt::Finished);
          }
          return Ok(CompareRowRslt::RowFull(row.edges.clone()));
        }
        return Err(RouterError::AddErrMultipleRouteParameters.into());
      }
      return Ok(CompareRowRslt::Unmatched);
    }

    let row_route = row.route.as_bytes();
    let common_prefix_len = Self::common_prefix_len(route, row_route);
    let (_lhs, rhs) = route.split_at_checked(common_prefix_len).unwrap_or_default();
    let is_row_fulfilled = common_prefix_len == row_route.len();
    if common_prefix_len == 0 {
      Ok(CompareRowRslt::Unmatched)
    } else if is_row_fulfilled {
      *route = rhs;
      if rhs.is_empty() {
        return Ok(CompareRowRslt::Finished);
      }
      Ok(CompareRowRslt::RowFull(row.edges.clone()))
    } else {
      *route = rhs;
      Ok(CompareRowRslt::RowPartial { common_prefix_len, is_single: rhs.is_empty() })
    }
  }

  /// Breaks `/a/{}/b/c-{}` into `/a/`, `{}`, `/b` and `/c-{}`.
  #[inline]
  fn manage_row_ty<'bytes>(
    bytes: &'bytes [u8],
    curr_bytes_idx: &mut usize,
  ) -> crate::Result<(&'bytes [u8], RowTy)> {
    let curr_bytes = bytes.get(*curr_bytes_idx..).unwrap_or_default();
    let Some(([first], rest)) = curr_bytes.split_at_checked(1) else {
      return Err(RouterError::AddErrEmpty.into());
    };
    let mut iter = rest.iter();
    let mut route_end_idx: u8 = 1;
    match first {
      b'{' => {
        for byte in iter.by_ref() {
          match byte {
            b'{' => return Err(RouterError::AddErrMultipleRouteParameters.into()),
            b'}' => {
              let next = iter.next().copied();
              if next.is_some() && next != Some(b'/') {
                return Err(RouterError::AddErrParamSuffix.into());
              }
              route_end_idx = route_end_idx.checked_add(1).ok_or(RouterError::AddErrOverflow)?;
              *curr_bytes_idx = curr_bytes_idx.wrapping_add(route_end_idx.into());
              let route = bytes.get(..*curr_bytes_idx).unwrap_or_default();
              return Ok((route, RowTy::Param));
            }
            other => {
              if !other.is_ascii_graphic() {
                return Err(RouterError::AddErrNonAsciiParam.into());
              }
            }
          }
          route_end_idx = route_end_idx.checked_add(1).ok_or(RouterError::AddErrOverflow)?;
        }
        Err(RouterError::AddErrUnclosedParameterBracket.into())
      }
      b'/' => {
        for byte in iter {
          match byte {
            b'{' => break,
            other => {
              if !other.is_ascii_graphic() {
                return Err(RouterError::AddErrNonAsciiLiteral.into());
              }
            }
          }
          route_end_idx = route_end_idx.checked_add(1).ok_or(RouterError::AddErrOverflow)?;
        }
        *curr_bytes_idx = curr_bytes_idx.wrapping_add(route_end_idx.into());
        let route = bytes.get(..*curr_bytes_idx).unwrap_or_default();
        Ok((route, RowTy::Literal))
      }
      _ => Err(RouterError::AddErrInvalidStart.into()),
    }
  }
}

impl<T, const MC: usize, const MD: usize> Drop for RouterBuilder<'_, T, MC, MD> {
  #[inline]
  fn drop(&mut self) {
    #[inline]
    fn evaluate_max_depth<T, const MC: usize, const MD: usize>(rows: &mut Vector<Row<T, MC>>) {
      let mut depths = alloc::vec![0u8; rows.len()];
      let mut max_depth = 0;
      for (idx, row) in rows.iter().enumerate() {
        let next_depth = {
          let Some(curr_depth) = depths.get(idx) else {
            continue;
          };
          if row.ty == RowTy::Param { curr_depth.saturating_add(1) } else { *curr_depth }
        };
        if next_depth > max_depth {
          max_depth = next_depth;
        }
        for edge in &row.edges {
          let target = usize::from(edge.row_target_idx);
          if target >= depths.len() {
            continue;
          }
          let Some(depth) = depths.get_mut(target) else {
            continue;
          };
          *depth = next_depth;
        }
      }
      if usize::from(max_depth) > MD {
        rows.clear();
      }
    }

    #[inline]
    fn fill_first_bytes<T, const MC: usize>(rows: &mut Vector<Row<T, MC>>) {
      let mut row_idx: usize = 0;
      while row_idx < rows.len() {
        let mut first_bytes = ArrayVectorU8::<_, MC>::new();
        if let Some(row) = rows.get(row_idx) {
          for edge in &row.edges {
            let Some(target_row) = rows.get(usize::from(edge.row_target_idx)) else {
              continue;
            };
            let first = match target_row.ty {
              RowTy::Literal => target_row.route.as_bytes().first().copied(),
              RowTy::Param => None,
            };
            let _rslt = first_bytes.push(first);
          }
        }
        if let Some(row) = rows.get_mut(row_idx) {
          for (edge, first_byte) in row.edges.iter_mut().zip(first_bytes) {
            edge.first_byte = first_byte;
          }
        }
        row_idx = row_idx.wrapping_add(1);
      }
      for row in rows {
        let Some(param_idx) = row.edges.iter().position(|el| el.first_byte.is_none()) else {
          continue;
        };
        let last_idx = row.edges.len().wrapping_sub(1).into();
        row.edges.swap(param_idx, last_idx);
      }
    }

    #[inline]
    fn sort_by_the_number_of_children<T, const MC: usize>(rows: &mut Vector<Row<T, MC>>) {
      let mut weights = alloc::vec![0u32; rows.len()];
      for idx in (0..rows.len()).rev() {
        let Some(row) = rows.get(idx) else {
          continue;
        };
        let mut weight = u32::from(row.value.is_some());
        for edge in &row.edges {
          let weight_node = weights.get(usize::from(edge.row_target_idx)).copied();
          weight = weight.wrapping_add(weight_node.unwrap_or_default());
        }
        if let Some(elem) = weights.get_mut(idx) {
          *elem = weight;
        }
      }
      for row in rows {
        row.edges.sort_unstable_by(|lhs, rhs| {
          let weight_a = weights.get(usize::from(lhs.row_target_idx)).copied().unwrap_or_default();
          let weight_b = weights.get(usize::from(rhs.row_target_idx)).copied().unwrap_or_default();
          weight_b.cmp(&weight_a)
        });
      }
    }

    sort_by_the_number_of_children(self.rows);
    fill_first_bytes(self.rows);
    evaluate_max_depth::<_, _, MD>(self.rows);
  }
}

/// Path constructed from a root route
#[derive(Debug, PartialEq)]
pub struct RouterMatch<
  'any,
  T,
  const MC: usize = DEFAULT_MAX_CHILDREN,
  const MD: usize = DEFAULT_MAX_DEPTH,
> {
  rmpi: ArrayVectorU8<RouterMatchParamIndices, MD>,
  route: ShortSliceU8<'any, u8>,
  rows: &'any Vector<Row<T, MC>>,
  value: &'any T,
}

impl<T, const MC: usize, const MD: usize> RouterMatch<'_, T, MC, MD> {
  /// User data originated from a route
  #[inline]
  pub fn data(&self) -> &T {
    self.value
  }

  /// Gets a parameter according to its index that is related to the entire URI.
  #[inline]
  pub fn param_by_idx(&self, idx: usize) -> Option<RouterMatchParam<'_>> {
    self.params().nth(idx)
  }

  /// Gets a parameter according to its declared name.
  #[inline]
  pub fn param_by_name(&self, name: &[u8]) -> Option<RouterMatchParam<'_>> {
    self.params().find(|el| el.name.as_bytes() == name)
  }

  /// Iterator over all parameters
  #[inline]
  pub fn params(&self) -> impl Iterator<Item = RouterMatchParam<'_>> {
    let Self { rmpi, route, rows, .. } = self;
    rmpi.iter().filter_map(|RouterMatchParamIndices { ident_param_range, row_idx }| {
      let row = rows.get(usize::from(*row_idx))?;
      let name = match row.ty {
        RowTy::Literal => return None,
        RowTy::Param => {
          if let [_, name @ .., _] = row.route.as_bytes() {
            // SAFETY: all routes are graphic ASCII
            unsafe { str::from_utf8_unchecked(name) }
          } else {
            // SAFETY: all parameters are captured with their braces included
            unsafe { unreachable_unchecked() }
          }
        }
      };
      // SAFETY: every route is originated from a string
      let value = unsafe {
        let range = ident_param_range.start.into()..ident_param_range.end.into();
        str::from_utf8_unchecked(route.get(range)?)
      };
      Some(RouterMatchParam::new(name, value))
    })
  }
}

/// Route match parameter
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RouterMatchParam<'any> {
  /// Identifies the parameter and is defined when building the route.
  pub name: &'any str,
  /// Dynamic value associated with the name.
  pub value: &'any str,
}

impl<'any> RouterMatchParam<'any> {
  /// Shortcut
  #[inline]
  pub const fn new(name: &'any str, value: &'any str) -> Self {
    Self { name, value }
  }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum CheckSearchRowRslt<'any, T, const MC: usize = DEFAULT_MAX_CHILDREN> {
  CompleteMatch(&'any T),
  IncompleteMatch(&'any ArrayVectorU8<Edge, MC>),
  Mismatch,
}

#[derive(Clone, Debug, PartialEq)]
enum CompareRowRslt<const MC: usize = DEFAULT_MAX_CHILDREN> {
  Finished,
  RowFull(ArrayVectorU8<Edge, MC>),
  RowPartial { common_prefix_len: usize, is_single: bool },
  Unmatched,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum RowTy {
  Literal,
  Param,
}

/// An edge is also a piece of data in CSR terms.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Edge {
  // If the target node is a parameter, then the first byte will always be 'None'.
  first_byte: Option<u8>,
  row_target_idx: u8,
}

impl Edge {
  #[inline]
  const fn new(first_byte: Option<u8>, row_target_idx: u8) -> Self {
    Self { first_byte, row_target_idx }
  }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct RouterMatchParamIndices {
  ident_param_range: Range<u8>,
  row_idx: u8,
}

impl RouterMatchParamIndices {
  #[inline]
  const fn new(ident_param_range: Range<u8>, row_idx: u8) -> Self {
    Self { ident_param_range, row_idx }
  }
}

/// A row can have at-most one parameter and a row is also a node in CSR terms.
///
/// Intermediate rows refer common prefixes shared by other routes while final rows are unique.
///
/// Leaf rows are always guarantee to have non-null values. Intermediate rows may or may not
/// have values.
#[derive(Clone, Debug, PartialEq)]
struct Row<T, const MC: usize = DEFAULT_MAX_CHILDREN> {
  edges: ArrayVectorU8<Edge, MC>,
  route: ShortBoxStrU8,
  ty: RowTy,
  value: Option<T>,
}

impl<T, const MC: usize> Row<T, MC> {
  #[inline]
  const fn new(
    edges: ArrayVectorU8<Edge, MC>,
    route: ShortBoxStrU8,
    ty: RowTy,
    value: Option<T>,
  ) -> Self {
    Self { edges, route, ty, value }
  }
}

#[inline]
fn common_route(
  common_prefix_len: usize,
  common_route: &ShortBoxStrU8,
) -> crate::Result<ShortBoxStrU8> {
  common_route.get(..common_prefix_len).unwrap_or_default().try_into()
}

#[inline]
fn unique_route(
  common_prefix_len: usize,
  common_route: &ShortBoxStrU8,
) -> crate::Result<ShortBoxStrU8> {
  common_route.get(common_prefix_len..).unwrap_or_default().try_into()
}
