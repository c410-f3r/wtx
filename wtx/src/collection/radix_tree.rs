use crate::{
  collection::{ArrayVectorU8, ShortStr, ShortStrU8, Vector},
  misc::{Ascii, str_split_once_str, str_split_once1},
};
use core::ops::Range;

/// Radix Tree Error
#[derive(Debug)]
pub enum RadixTreeError {
  /// Conflicting route definitions
  ConflictingRoute,
  /// Duplicate route already exists
  DuplicateRoute,
  /// Invalid parameter syntax
  InvalidParameterSyntax,
  /// Invalid Route Definition
  InvalidRouteDefinition,
  /// Multiple wildcards in the same path
  MultipleWildcardsInPath,
  /// Nested parameters. For example, `{id{nested}}`.
  NestedParameters,
  /// Create a sub node to add more parameters in a path
  NodesCanHaveAtMostOneParameter,
  /// Route does not exist
  UnknownMatchingRoute,
  /// Unclosed parameter bracket
  UnclosedParameterBracket,
  /// Wildcard must be at the end of a path segment
  WildcardNotAtEnd,
}

/// Decomposes a series of prefixed strings into common nodes to allow fast comparisons with
/// other dynamic strings.
///
/// This particular radix tree allows the insertion of wildcards or dynamic parameters, which makes
/// it suitable for URI routers.
///
/// * `hey_{anything*}`: hey_, hey_world, hey_you_you_are_nodding_out
/// * `{phrase}_ready_to_go.avif`: are_you_ready_to_go.avif,  cause i_am_ready_to_go.avif
#[derive(Clone, Debug)]
pub struct RadixTree<T> {
  edges: Vector<Edge>,
  nodes: Vector<Node<T>>,
  root_len: u8,
}

impl<T> RadixTree<T> {
  /// Empty instance
  #[inline]
  pub const fn new() -> Self {
    Self { edges: Vector::new(), nodes: Vector::new(), root_len: 0 }
  }

  /// See [`MatcherBuilder`].
  #[inline]
  pub fn builder(&mut self) -> RadixTreeBuilder<'_, T> {
    self.edges.clear();
    self.nodes.clear();
    self.root_len = 0;
    RadixTreeBuilder {
      edges: &mut self.edges,
      nodes: &mut self.nodes,
      root_len: &mut self.root_len,
    }
  }

  /// Tries to find a path that corresponds to `identifier`.
  #[inline]
  pub fn find<'this, 'data, 'rslt>(
    &'this self,
    identifier: &'data str,
  ) -> crate::Result<RadixTreePath<'rslt, T>>
  where
    'data: 'rslt,
    'this: 'rslt,
  {
    let identifier_len: u8 = identifier.len().try_into()?;
    let mut absolute_offset = 0u8;
    let mut curr_edges_range_opt = None;
    let mut curr_identifier = identifier;
    let mut edges_offset_begin = 0u8;
    let mut node_idx = 0u8;
    let mut nodes_params = ArrayVectorU8::new();

    for node in self.nodes.get(0..usize::from(self.root_len)).unwrap_or_default() {
      let len_before = curr_identifier.len();
      match Self::check_search_node(
        absolute_offset,
        &mut curr_edges_range_opt,
        &mut curr_identifier,
        &mut edges_offset_begin,
        identifier_len,
        node,
        node_idx,
        &mut nodes_params,
      ) {
        Some(false) => {
          node_idx = node_idx.wrapping_add(1);
          continue;
        }
        Some(true) => return Ok(RadixTreePath { identifier, nodes: &self.nodes, nodes_params }),
        None => {
          let consumed = len_before.wrapping_sub(curr_identifier.len()).try_into()?;
          absolute_offset = absolute_offset.wrapping_add(consumed);
          break;
        }
      }
    }
    loop {
      let Some(curr_edges_range) = curr_edges_range_opt.take() else {
        return Err(RadixTreeError::UnknownMatchingRoute.into());
      };
      let range = usize::from(curr_edges_range.start)..usize::from(curr_edges_range.end);
      for edge in self.edges.get(range).unwrap_or_default() {
        let Some(node) = self.nodes.get(usize::from(edge.node_target_idx)) else {
          continue;
        };
        let len_before = curr_identifier.len();
        match Self::check_search_node(
          absolute_offset,
          &mut curr_edges_range_opt,
          &mut curr_identifier,
          &mut edges_offset_begin,
          identifier_len,
          node,
          edge.node_target_idx,
          &mut nodes_params,
        ) {
          Some(false) => continue,
          Some(true) => return Ok(RadixTreePath { identifier, nodes: &self.nodes, nodes_params }),
          None => {
            let consumed = len_before.wrapping_sub(curr_identifier.len()).try_into()?;
            absolute_offset = absolute_offset.wrapping_add(consumed);
            break;
          }
        }
      }
    }
  }

  #[inline]
  fn check_search_node(
    absolute_offset: u8,
    curr_edges_range_opt: &mut Option<Range<u8>>,
    curr_identifier: &mut &str,
    edges_offset_begin: &mut u8,
    identifier_len: u8,
    node: &Node<T>,
    node_idx: u8,
    nodes_params: &mut ArrayVectorU8<NodeParam, 8>,
  ) -> Option<bool> {
    let comparing_name = match node.ty {
      NodeTy::Literal => &*node.identifier,
      NodeTy::Param { begin_idx, .. } | NodeTy::Wildcard { begin_idx, .. } => {
        node.identifier.get(..usize::from(begin_idx)).unwrap_or_default()
      }
    };
    let Some((lhs, rhs)) = curr_identifier.split_at_checked(comparing_name.len()) else {
      return Some(false);
    };
    if lhs != comparing_name {
      return Some(false);
    }
    let param_range = match node.ty {
      NodeTy::Literal => {
        *curr_identifier = rhs;
        0..0
      }
      NodeTy::Param { after, end_idx, .. } => {
        let tail = node.identifier.get(usize::from(end_idx)..).unwrap_or_default();
        let (param, after) = if tail.is_empty() {
          if let Some(byte) = after
            && let Some((param, _)) = str_split_once1(rhs, byte)
          {
            (param, rhs.get(param.len()..).unwrap_or_default())
          } else {
            (rhs, "")
          }
        } else {
          let Some((param, after)) = str_split_once_str(rhs, tail) else {
            return Some(false);
          };
          (param, after)
        };
        *curr_identifier = after;
        let comparing_name_len = comparing_name.len().try_into().unwrap_or_default();
        let param_start = absolute_offset.wrapping_add(comparing_name_len);
        let param_end = param_start.wrapping_add(param.len().try_into().unwrap_or_default());
        param_start..param_end
      }
      NodeTy::Wildcard { begin_idx, .. } => {
        *curr_identifier = "";
        absolute_offset.wrapping_add(begin_idx)..identifier_len
      }
    };
    let _rslt = nodes_params.push(NodeParam::new(node_idx, param_range));
    if curr_identifier.is_empty() {
      Some(true)
    } else {
      let edges_range = *edges_offset_begin..node.edges_offset;
      *curr_edges_range_opt = Some(edges_range);
      *edges_offset_begin = node.edges_offset;
      None
    }
  }
}

impl<T> Default for RadixTree<T> {
  #[inline]
  fn default() -> Self {
    Self::new()
  }
}

/// Constructs nodes
#[derive(Debug)]
pub struct RadixTreeBuilder<'instance, T> {
  edges: &'instance mut Vector<Edge>,
  nodes: &'instance mut Vector<Node<T>>,
  root_len: &'instance mut u8,
}

impl<'instance, T> RadixTreeBuilder<'instance, T> {
  /// Adds a new node and its associated value to the tree.
  #[inline]
  pub fn node(&mut self, mut identifier: &'static str, value: T) -> crate::Result<&mut Self> {
    let mut curr_edges_range_opt = None;
    let mut is_root = true;
    let mut nodes_idx = 0u8;
    let mut parent_node_idx = 0;
    let mut edges_offset_begin = 0;

    for node in self.nodes.get(0..usize::from(*self.root_len)).unwrap_or_default() {
      match Self::check_insert_node(
        &mut curr_edges_range_opt,
        &mut edges_offset_begin,
        &mut is_root,
        node,
        &mut identifier,
      )? {
        CheckInsertNodeRslt::Impossible | CheckInsertNodeRslt::Unmatched => {
          nodes_idx = nodes_idx.wrapping_add(1);
          continue;
        }
        CheckInsertNodeRslt::MatchedAll => {
          parent_node_idx = nodes_idx;
          break;
        }
        CheckInsertNodeRslt::MatchedPart { common_prefix_len, is_single } => {
          let node_identifier = ShortStrU8::new(identifier)?;
          self.add_common_node(common_prefix_len, is_single, node_identifier, nodes_idx, value)?;
          return Ok(self);
        }
      }
    }

    if is_root {
      self.add_root_node(ShortStrU8::new(identifier)?, value)?;
      return Ok(self);
    }

    loop {
      let Some(curr_edges_range) = curr_edges_range_opt.take() else {
        return Ok(self);
      };
      let range = usize::from(curr_edges_range.start)..usize::from(curr_edges_range.end);
      let mut has_match = false;
      for edge in self.edges.get(range).unwrap_or_default() {
        let Some(node) = self.nodes.get(usize::from(edge.node_target_idx)) else {
          continue;
        };
        edges_offset_begin = if let Some(idx) = edge.node_target_idx.checked_sub(1) {
          self.nodes.get(usize::from(idx)).map_or(0, |el| el.edges_offset)
        } else {
          0
        };
        match Self::check_insert_node(
          &mut curr_edges_range_opt,
          &mut edges_offset_begin,
          &mut is_root,
          node,
          &mut identifier,
        )? {
          CheckInsertNodeRslt::Impossible | CheckInsertNodeRslt::Unmatched => continue,
          CheckInsertNodeRslt::MatchedAll => {
            parent_node_idx = edge.node_target_idx;
            has_match = true;
            break;
          }
          CheckInsertNodeRslt::MatchedPart { common_prefix_len, is_single } => {
            self.add_common_node(
              common_prefix_len,
              is_single,
              ShortStrU8::new(identifier)?,
              edge.node_target_idx,
              value,
            )?;
            return Ok(self);
          }
        }
      }
      if !has_match {
        self.add_common_node(0, true, ShortStrU8::new(identifier)?, parent_node_idx, value)?;
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
    node_identifier: ShortStrU8<'static>,
    mut node_insert_idx0: u8,
    node_value: T,
  ) -> crate::Result<()> {
    node_insert_idx0 = node_insert_idx0.wrapping_add(1);
    let is_in_same_depth = common_prefix_len > 0;
    let offset = if is_single { 1 } else { 2 };
    self.adjust_edges(node_insert_idx0.into(), offset);
    let (parent, rest) = if is_single {
      self.nodes.insert(node_insert_idx0.into(), Node::from_identifier(node_identifier))?;
      let split_opt = self.nodes.split_at_mut_checked(node_insert_idx0.into());
      let Some(([.., parent], [child, rest @ ..])) = split_opt else {
        return Ok(());
      };
      Self::modify_child(child, common_prefix_len, node_identifier, parent)?;
      child.edges_offset = parent.edges_offset.wrapping_add(offset.into());
      child.value = Some(node_value);
      self.edges.insert(parent.edges_offset.into(), Edge::new(node_insert_idx0))?;
      (parent, rest)
    } else {
      let node_insert_idx1 = node_insert_idx0.wrapping_add(1);
      self.nodes.insert(node_insert_idx0.into(), Node::from_identifier(node_identifier))?;
      self.nodes.insert(node_insert_idx1.into(), Node::from_identifier(node_identifier))?;
      let split_opt = self.nodes.split_at_mut_checked(node_insert_idx0.into());
      let Some((lhs, [child0, child1, rest @ ..])) = split_opt else {
        return Ok(());
      };
      let (parent1_edges_offset, parent0) = match lhs {
        [.., parent1, parent0] => (parent1.edges_offset, parent0),
        [.., parent0] => (0, parent0),
        _ => return Ok(()),
      };
      Self::modify_child(child0, common_prefix_len, node_identifier, parent0)?;
      Self::modify_child(child1, common_prefix_len, parent0.identifier, parent0)?;
      child0.edges_offset = parent1_edges_offset.wrapping_add(offset);
      child0.value = Some(node_value);
      child1.edges_offset = parent0.edges_offset.wrapping_add(offset);
      child1.value = parent0.value.take();
      self.edges.insert(parent1_edges_offset.into(), Edge::new(node_insert_idx1))?;
      self.edges.insert(parent1_edges_offset.into(), Edge::new(node_insert_idx0))?;
      parent0.edges_offset = parent1_edges_offset;
      (parent0, rest)
    };
    parent.edges_offset = parent.edges_offset.wrapping_add(offset);
    if is_in_same_depth {
      parent.identifier = Self::common_path(common_prefix_len, parent.identifier);
    }
    for node in rest {
      node.edges_offset = node.edges_offset.wrapping_add(offset);
    }
    Ok(())
  }

  #[inline]
  fn add_root_node(
    &mut self,
    node_identifier: ShortStrU8<'static>,
    node_value: T,
  ) -> crate::Result<()> {
    let ty = Self::find_node_ty(node_identifier)?;
    self.adjust_edges(*self.root_len, 1);
    let edges_offset = if let Some(idx) = self.root_len.checked_sub(1) {
      self.nodes.get(usize::from(idx)).map_or(0, |el| el.edges_offset)
    } else {
      0
    };
    self.nodes.insert(
      usize::from(*self.root_len),
      Node::new(edges_offset, node_identifier, ty, Some(node_value)),
    )?;
    *self.root_len = self.root_len.wrapping_add(1);
    Ok(())
  }

  fn adjust_edges(&mut self, node_idx: u8, offset: u8) {
    for edge in &mut *self.edges {
      if edge.node_target_idx >= node_idx {
        edge.node_target_idx = edge.node_target_idx.wrapping_add(offset);
      }
    }
  }

  #[inline]
  fn check_insert_node(
    curr_edges_range_opt: &mut Option<Range<u8>>,
    edges_offset_begin: &mut u8,
    is_root: &mut bool,
    node: &Node<T>,
    node_identifier: &mut &str,
  ) -> crate::Result<CheckInsertNodeRslt> {
    let edges_range = *edges_offset_begin..node.edges_offset;
    *edges_offset_begin = node.edges_offset;
    let comparing_identifier = match node.ty {
      NodeTy::Literal => &*node.identifier,
      NodeTy::Param { begin_idx, .. } | NodeTy::Wildcard { begin_idx, .. } => {
        node.identifier.get(..usize::from(begin_idx)).unwrap_or_default()
      }
    };
    let Some((lhs, rhs)) = node_identifier.split_at_checked(comparing_identifier.len()) else {
      let common_prefix_len = Self::common_prefix_len(node_identifier, comparing_identifier);
      if common_prefix_len > 0 && node_identifier.contains('{') {
        return Err(RadixTreeError::InvalidRouteDefinition.into());
      }
      return Ok(CheckInsertNodeRslt::Impossible);
    };
    let common_prefix_len = Self::common_prefix_len(lhs, comparing_identifier);
    if common_prefix_len == 0 {
      return Ok(CheckInsertNodeRslt::Unmatched);
    } else if common_prefix_len != lhs.len() {
      return Ok(CheckInsertNodeRslt::MatchedPart {
        common_prefix_len,
        is_single: common_prefix_len == comparing_identifier.len(),
      });
    }
    *is_root = false;
    match node.ty {
      NodeTy::Literal => {
        *node_identifier = rhs;
      }
      NodeTy::Param { end_idx, .. } => {
        let tail = node.identifier.get(usize::from(end_idx)..).unwrap_or_default();
        if tail.is_empty() {
          let common_prefix_len = Self::common_prefix_len(&*node.identifier, *node_identifier);
          if common_prefix_len == node.identifier.len() {
            let remainder = &node_identifier.get(common_prefix_len..).unwrap_or_default();
            *curr_edges_range_opt = Some(edges_range);
            *node_identifier = remainder;
            return Ok(CheckInsertNodeRslt::MatchedAll);
          }
          return Err(RadixTreeError::ConflictingRoute.into());
        } else {
          let Some((_, after)) = str_split_once_str(rhs, tail) else {
            let common_prefix_len = Self::common_prefix_len(&*node.identifier, *node_identifier);
            return Ok(CheckInsertNodeRslt::MatchedPart {
              common_prefix_len,
              is_single: common_prefix_len == node.identifier.len(),
            });
          };
          *node_identifier = after;
        }
      }
      NodeTy::Wildcard { .. } => return Err(RadixTreeError::MultipleWildcardsInPath.into()),
    };
    if node_identifier.is_empty() {
      return Err(RadixTreeError::DuplicateRoute.into());
    }
    *curr_edges_range_opt = Some(edges_range);
    Ok(CheckInsertNodeRslt::MatchedAll)
  }

  #[inline]
  fn common_path(common_prefix_len: usize, identifier: ShortStrU8<'_>) -> ShortStrU8<'_> {
    ShortStrU8::from(identifier.into_str().get(..common_prefix_len).unwrap_or_default())
  }

  #[inline]
  fn common_prefix_len(lhs: &str, rhs: &str) -> usize {
    lhs.as_bytes().iter().zip(rhs.as_bytes()).take_while(|(a, b)| a == b).count()
  }

  #[inline]
  fn find_node_ty(identifier: ShortStrU8<'static>) -> crate::Result<NodeTy> {
    let str = identifier.into_str();
    let Some((lhs, rhs)) = str_split_once1(str, Ascii::OPENING_BRACE) else {
      return Ok(NodeTy::Literal);
    };
    let begin_idx: u8 = lhs.len().try_into()?;
    let mut end_idx = begin_idx;
    let mut iter = rhs.as_bytes().iter();
    for elem in iter.by_ref() {
      match elem {
        b'*' => {
          let (Some(b'}'), None) = (iter.next(), iter.next()) else {
            return Err(RadixTreeError::WildcardNotAtEnd.into());
          };
          let name_begin_idx = begin_idx.wrapping_add(1);
          let name_end_idx = end_idx.wrapping_add(1);
          let name = str.get(name_begin_idx.into()..name_end_idx.into()).unwrap_or_default();
          return Ok(NodeTy::Wildcard { begin_idx, name: name.into() });
        }
        b'{' => {
          return Err(RadixTreeError::NestedParameters.into());
        }
        b'}' => {
          if iter.any(|el| *el == b'{') {
            return Err(RadixTreeError::NodesCanHaveAtMostOneParameter.into());
          }
          let name_begin_idx = begin_idx.wrapping_add(1);
          let name_end_idx = end_idx.wrapping_add(1);
          let name = str.get(name_begin_idx.into()..name_end_idx.into()).unwrap_or_default();
          end_idx = end_idx.wrapping_add(2);
          let does_not_have_suffix = usize::from(end_idx) == name.len();
          let after = does_not_have_suffix.then_some(Ascii::NULL);
          return Ok(NodeTy::Param { after, begin_idx, end_idx, name: name.into() });
        }
        _ => {
          end_idx = end_idx.wrapping_add(1);
        }
      }
    }
    Err(RadixTreeError::UnclosedParameterBracket.into())
  }

  #[inline]
  fn modify_child(
    child: &mut Node<T>,
    common_prefix_len: usize,
    node_identifier: ShortStr<'static, u8>,
    parent: &mut Node<T>,
  ) -> crate::Result<()> {
    child.identifier = Self::unique_path(common_prefix_len, node_identifier);
    child.ty = Self::find_node_ty(child.identifier)?;
    if let NodeTy::Param { after, .. } = &mut parent.ty
      && after.is_none()
    {
      *after = Some(child.identifier.as_bytes().first().copied().unwrap_or_default().try_into()?);
    }
    Ok(())
  }

  #[inline]
  fn unique_path(common_prefix_len: usize, identifier: ShortStrU8<'_>) -> ShortStrU8<'_> {
    ShortStrU8::from(identifier.into_str().get(common_prefix_len..).unwrap_or_default())
  }
}

#[derive(Debug, PartialEq)]
pub struct RadixTreePath<'any, T> {
  identifier: &'any str,
  nodes: &'any Vector<Node<T>>,
  nodes_params: ArrayVectorU8<NodeParam, 8>,
}

impl<'any, T> RadixTreePath<'any, T> {
  #[inline]
  pub fn data(&self) -> &T {
    // SAFETY: Non-empty routes always have at least one path
    let last_node_idx = unsafe { self.nodes_params.last().unwrap_unchecked().node_idx };
    // SAFETY: The searcher always ensure that indices point to valid instances.
    let node = unsafe { self.nodes.get(usize::from(last_node_idx)).unwrap_unchecked() };
    // SAFETY: Leaf nodes always have non-null values
    unsafe { node.value.as_ref().unwrap_unchecked() }
  }

  /// Gets a parameter according to its index that is related to the entire URI.
  #[inline]
  pub fn param_by_idx(&self, idx: usize) -> Option<RadixTreePathParam<'_>> {
    self.params().nth(idx)
  }

  /// Gets a parameter according to its declared name.
  #[inline]
  pub fn param_by_name(&self, name: &[u8]) -> Option<RadixTreePathParam<'_>> {
    self.params().find(|p| p.name.as_bytes() == name)
  }

  /// Iterator over all parameters
  #[inline]
  pub fn params(&self) -> impl Iterator<Item = RadixTreePathParam<'_>> {
    let Self { identifier, nodes, nodes_params } = self;
    nodes_params.iter().filter_map(|NodeParam { node_idx, param_range }| {
      let node = nodes.get(usize::from(*node_idx))?;
      let name = match node.ty {
        NodeTy::Literal => return None,
        NodeTy::Param { name, .. } => name,
        NodeTy::Wildcard { name, .. } => name,
      };
      let value = identifier.get(param_range.start.into()..param_range.end.into())?;
      Some(RadixTreePathParam::new(name.into_str(), value))
    })
  }
}

/// Route match parameter
#[derive(Debug, PartialEq)]
pub struct RadixTreePathParam<'any> {
  /// Identifies the parameter and is defined when building the route.
  pub name: &'static str,
  /// Dynamic value associated with the name.
  pub value: &'any str,
}

impl<'any> RadixTreePathParam<'any> {
  /// Shortcut
  #[inline]
  pub const fn new(name: &'static str, value: &'any str) -> Self {
    Self { name, value }
  }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum CheckInsertNodeRslt {
  Impossible,
  // Path was not fully consumed. A fully consumed path is actually an error.
  MatchedAll,
  MatchedPart { common_prefix_len: usize, is_single: bool },
  Unmatched,
}

/// Parameters and wildcards are always suffixes.
#[derive(Clone, Copy, Debug, PartialEq)]
enum NodeTy {
  Literal,
  // If `after` is null, then the parameter has a suffix
  Param { after: Option<Ascii>, begin_idx: u8, end_idx: u8, name: ShortStrU8<'static> },
  Wildcard { begin_idx: u8, name: ShortStrU8<'static> },
}

/// An edge is also a piece of data in CSR terms.
#[derive(Clone, Debug, PartialEq)]
struct Edge {
  node_target_idx: u8,
}

impl Edge {
  #[inline]
  const fn new(node_target_idx: u8) -> Self {
    Self { node_target_idx }
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
  edges_offset: u8,
  identifier: ShortStrU8<'static>,
  ty: NodeTy,
  value: Option<T>,
}

impl<T> Node<T> {
  #[inline]
  const fn from_identifier(identifier: ShortStrU8<'static>) -> Self {
    Self { edges_offset: 0, identifier, ty: NodeTy::Literal, value: None }
  }

  #[inline]
  const fn new(
    edges_offset: u8,
    identifier: ShortStrU8<'static>,
    ty: NodeTy,
    value: Option<T>,
  ) -> Self {
    Self { edges_offset, identifier, ty, value }
  }
}

#[derive(Debug, PartialEq)]
struct NodeParam {
  node_idx: u8,
  param_range: Range<u8>,
}

impl NodeParam {
  #[inline]
  const fn new(node_idx: u8, param_range: Range<u8>) -> Self {
    Self { node_idx, param_range }
  }
}

#[cfg(test)]
mod tests {
  use crate::collection::{
    RadixTreePathParam,
    radix_tree::{Edge, Node, NodeTy, RadixTree},
  };

  #[test]
  fn add_static_route_after_param_route_and_vice_versa() {
    {
      let mut radix_tree = RadixTree::new();
      let mut builder = radix_tree.builder();
      let _ = builder.node("/user/{id}", 1).unwrap();
      assert!(builder.node("/user/static", 2).is_err());
    }
    {
      let mut radix_tree = RadixTree::new();
      let mut builder = radix_tree.builder();
      let _ = builder.node("/user/static", 1).unwrap();
      assert!(builder.node("/user/{id}", 2).is_err());
    }
  }

  #[test]
  fn deep_traversal_after_splits() {
    let mut radix_tree = RadixTree::new();
    {
      let mut builder = radix_tree.builder();
      builder.node("/api/users", 1).unwrap();
      builder.node("/api/posts", 2).unwrap();
      builder.node("/api/comments", 3).unwrap();
    }
    assert_eq!(radix_tree.find("/api/users").unwrap().data(), &1);
    assert_eq!(radix_tree.find("/api/posts").unwrap().data(), &2);
    assert_eq!(radix_tree.find("/api/comments").unwrap().data(), &3);
    assert!(radix_tree.find("/api").is_err());
    assert!(radix_tree.find("/api/user").is_err());
  }

  #[test]
  fn find_on_intermediate_node_without_value() {
    let mut radix_tree = RadixTree::new();
    {
      let mut builder = radix_tree.builder();
      let _ = builder.node("/foo/bar", 1).unwrap();
      let _ = builder.node("/foo/baz", 2).unwrap();
    }
    assert!(radix_tree.find("/foo/").is_err());
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
  // | /ddd     | <- Row 0
  // | /aaa/     | <- Row 1
  //  ----------
  //   ccc       <- Row 2
  //   bbb       <- Row 3
  // ```
  //
  // `/bbb` and `/ccc` are not in root, as such, they are not searchable by the `find` method.
  #[test]
  fn literal() {
    let mut radix_tree = RadixTree::new();
    {
      let mut builder = radix_tree.builder();
      let _ = builder.node("/aaa/bbb", 1).unwrap();
      let _ = builder.node("/aaa/ccc", 2).unwrap();
      let _ = builder.node("/ddd", 3).unwrap();
    }
    assert_eq!(&radix_tree.edges, &[Edge::new(2), Edge::new(3)]);
    assert_eq!(
      &radix_tree.nodes,
      &[
        Node::new(2, "/aaa/".into(), NodeTy::Literal, None),
        Node::new(2, "/ddd".into(), NodeTy::Literal, Some(3)),
        Node::new(2, "ccc".into(), NodeTy::Literal, Some(2)),
        Node::new(2, "bbb".into(), NodeTy::Literal, Some(1)),
      ]
    );
    assert_eq!(radix_tree.root_len, 2);
    assert_eq!(radix_tree.find("/aaa/bbb").unwrap().data(), &1);
    assert_eq!(radix_tree.find("/aaa/ccc").unwrap().data(), &2);
    assert_eq!(radix_tree.find("/ddd").unwrap().data(), &3);
    assert!(radix_tree.find("").is_err());
    assert!(radix_tree.find("/aaa").is_err());
    assert!(radix_tree.find("/aaa/bb").is_err());
    assert!(radix_tree.find("/aaa/bbbb").is_err());
    assert!(radix_tree.find("/eee").is_err());
  }

  #[test]
  fn multiple_root_nodes_edge_tracking() {
    let mut radix_tree = RadixTree::new();
    {
      let mut builder = radix_tree.builder();
      builder.node("/foo", 1).unwrap();
      builder.node("/bar", 2).unwrap();
      builder.node("/baz", 3).unwrap();
    }
    assert_eq!(
      &radix_tree.nodes,
      &[
        Node::new(2, "/".into(), NodeTy::Literal, None),
        Node::new(4, "ba".into(), NodeTy::Literal, None),
        Node::new(4, "z".into(), NodeTy::Literal, Some(3)),
        Node::new(4, "r".into(), NodeTy::Literal, Some(2)),
        Node::new(4, "foo".into(), NodeTy::Literal, Some(1)),
      ]
    );
    assert_eq!(radix_tree.edges, &[Edge::new(1), Edge::new(4), Edge::new(2), Edge::new(3)]);
    assert_eq!(radix_tree.root_len, 1);
    assert_eq!(radix_tree.find("/foo").unwrap().data(), &1);
    assert_eq!(radix_tree.find("/bar").unwrap().data(), &2);
    assert_eq!(radix_tree.find("/baz").unwrap().data(), &3);
  }

  #[test]
  fn parameter_after_literal() {
    let mut radix_tree = RadixTree::new();
    {
      let mut builder = radix_tree.builder();
      let _ = builder.node("/aaaa", 1).unwrap();
      let _ = builder.node("/aaaa/{bbb}", 2).unwrap();
    }
    assert_eq!(radix_tree.find("/aaaa/rrr").unwrap().data(), &2);
  }

  #[test]
  fn parameter_in_split_node_is_correctly_extracted() {
    let mut radix_tree = RadixTree::new();
    {
      let mut builder = radix_tree.builder();
      let _ = builder.node("/api/v1/user", 1).unwrap();
      let _ = builder.node("/api/{version}/data", 2).unwrap();
    }
    let path = radix_tree.find("/api/v2/data").unwrap();
    assert_eq!(path.data(), &2);
    assert_eq!(path.param_by_name(b"version").unwrap(), RadixTreePathParam::new("version", "v2"));
  }

  #[test]
  fn parameter_node_split() {
    let mut radix_tree = RadixTree::new();
    {
      let mut builder = radix_tree.builder();
      builder.node("/user/{id}/profile", 1).unwrap();
      builder.node("/user/{id}/settings", 2).unwrap();
    }
    assert_eq!(radix_tree.find("/user/123/profile").unwrap().data(), &1);
    assert_eq!(radix_tree.find("/user/456/settings").unwrap().data(), &2);
  }

  #[test]
  fn parameters() {
    let mut radix_tree = RadixTree::new();
    {
      let mut builder = radix_tree.builder();
      let _ = builder.node("/aaa/{}/bbb", 1).unwrap();
    }
    let path = radix_tree.find("/aaa/123/bbb").unwrap();
    assert_eq!(path.data(), &1);
    assert_eq!(path.param_by_idx(0).unwrap(), RadixTreePathParam::new("", "123"));
    assert_eq!(path.param_by_idx(1), None);
  }

  #[test]
  fn route_with_param_suffix_should_not_match_without_it() {
    let mut radix_tree = RadixTree::new();
    {
      let mut builder = radix_tree.builder();
      let _ = builder.node("/user/{id}/profile", 1).unwrap();
    }
    assert!(radix_tree.find("/user/123").is_err());
  }

  #[test]
  fn several_parameters() {
    let mut radix_tree = RadixTree::new();
    {
      let mut builder = radix_tree.builder();
      let _ = builder.node("/aa", 1).unwrap();
      let _ = builder.node("/aa/{}", 2).unwrap();
      let _ = builder.node("/bb/{}", 3).unwrap();
      let _ = builder.node("/bb/{}/cc/{}", 4).unwrap();
    }
    let _ = radix_tree.find("/aa").unwrap();
    let _ = radix_tree.find("/aa/111").unwrap();
    let _ = radix_tree.find("/bb/222").unwrap();
    let _ = radix_tree.find("/bb/333/cc/444").unwrap();
  }

  #[test]
  fn split_node_single_wrong_identifier() {
    {
      let mut radix_tree = RadixTree::new();
      {
        let mut builder = radix_tree.builder();
        builder.node("/ab", 1).unwrap();
        builder.node("/abc", 2).unwrap();
      }
      assert_eq!(radix_tree.find("/ab").unwrap().data(), &1);
      assert_eq!(radix_tree.find("/abc").unwrap().data(), &2);
    }

    {
      let mut radix_tree = RadixTree::new();
      {
        let mut builder = radix_tree.builder();
        builder.node("/abc", 2).unwrap();
        builder.node("/ab", 1).unwrap();
      }
      assert_eq!(radix_tree.find("/ab").unwrap().data(), &1);
      assert_eq!(radix_tree.find("/abc").unwrap().data(), &2);
    }
  }

  #[test]
  fn two_parameters_in_a_path() {
    let mut radix_tree = RadixTree::new();
    {
      let mut builder = radix_tree.builder();
      let _ = builder.node("/aa/{}", 1).unwrap();
      let _ = builder.node("/aa/{}/bb/{}", 2).unwrap();
    }
    let path = radix_tree.find("/aa/111/bb/222").unwrap();
    assert_eq!(path.data(), &2);
    assert_eq!(path.param_by_idx(0).unwrap(), RadixTreePathParam::new("", "111"));
    assert_eq!(path.param_by_idx(1).unwrap(), RadixTreePathParam::new("", "222"));
  }

  #[test]
  fn two_parameters_in_a_string() {
    let mut radix_tree = RadixTree::new();
    let mut builder = radix_tree.builder();
    assert!(builder.node("/aa/{}/bb/{}", 1).is_err());
  }

  #[test]
  fn wildcards() {
    let mut radix_tree = RadixTree::new();
    {
      let mut builder = radix_tree.builder();
      let _ = builder.node("/aaa/{all*}", 1).unwrap();
    }
    let path = radix_tree.find("/aaa/123/bbb").unwrap();
    assert_eq!(path.data(), &1);
    assert_eq!(path.param_by_idx(0).unwrap(), RadixTreePathParam::new("all", "123/bbb"));
    assert_eq!(path.param_by_name(b"all").unwrap(), RadixTreePathParam::new("all", "123/bbb"));
    assert_eq!(path.param_by_idx(1), None);
    assert!(radix_tree.find("/bbb").is_err());
  }
}
