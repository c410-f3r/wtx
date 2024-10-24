use alloc::collections::VecDeque;
use core::{borrow::Borrow, hash::Hash};
use hashbrown::HashMap;

#[derive(Debug)]
pub(crate) struct IndexMap<K, V> {
  cursor: usize,
  elements: HashMap<K, V>,
  keys: VecDeque<K>,
}

impl<K, V> IndexMap<K, V>
where
  K: Clone + Copy + Eq + Hash,
{
  #[inline]
  pub(crate) fn new() -> Self {
    Self { cursor: 0, elements: HashMap::new(), keys: VecDeque::new() }
  }

  #[inline]
  pub(crate) fn clear(&mut self) {
    self.cursor = 0;
    self.elements.clear();
    self.keys.clear();
  }

  #[inline]
  pub(crate) fn decrease_cursor(&mut self) {
    self.cursor = self.cursor.wrapping_sub(1);
  }

  #[inline]
  pub(crate) fn front_mut(&mut self) -> Option<&mut V> {
    if self.cursor >= self.elements.len() {
      return None;
    }
    let key = self.keys.front()?;
    let value = self.elements.get_mut(key)?;
    Some(value)
  }

  #[inline]
  pub(crate) fn increase_cursor(&mut self) {
    self.cursor = self.cursor.wrapping_add(1);
  }

  #[inline]
  pub(crate) fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
    self.elements.iter()
  }

  #[inline]
  pub(crate) fn push_back(&mut self, key: K, value: V) -> Option<V> {
    let prev_value = self.elements.insert(key.clone(), value);
    if prev_value.is_none() {
      self.keys.push_back(key);
    }
    prev_value
  }

  #[inline]
  pub(crate) fn remove<Q>(&mut self, key: &Q) -> Option<V>
  where
    K: Borrow<Q>,
    Q: Eq + Hash + ?Sized,
  {
    let value = self.elements.remove(key)?;
    if self.elements.is_empty() {
      self.keys.clear();
    }
    Some(value)
  }
}
