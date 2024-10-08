use core::{borrow::Borrow, hash::Hash};
use hashbrown::HashMap;
use std::collections::VecDeque;

#[derive(Debug)]
pub struct IndexMap<K, V> {
  elements: HashMap<K, V>,
  keys: VecDeque<K>,
}

impl<K, V> IndexMap<K, V>
where
  K: Clone + Copy + Eq + Hash,
{
  #[inline]
  pub fn new() -> Self {
    Self { elements: HashMap::new(), keys: VecDeque::new() }
  }

  #[inline]
  pub fn clear(&mut self) {
    self.elements.clear();
    self.keys.clear();
  }

  #[inline]
  pub fn contains_key<Q>(&mut self, key: &Q) -> bool
  where
    K: Borrow<Q>,
    Q: Eq + Hash + ?Sized,
  {
    self.elements.contains_key(key)
  }

  #[inline]
  pub fn front_mut(&mut self) -> Option<&mut V> {
    let key = self.keys.front()?;
    let value = self.elements.get_mut(key)?;
    Some(value)
  }

  #[inline]
  pub fn get<Q>(&self, key: &Q) -> Option<&V>
  where
    K: Borrow<Q>,
    Q: Eq + Hash + ?Sized,
  {
    self.elements.get(key.borrow())
  }

  #[inline]
  pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
    self.elements.iter()
  }

  #[inline]
  pub fn push_back(&mut self, key: K, value: V) -> Option<V> {
    let prev_value = self.elements.insert(key.clone(), value);
    if prev_value.is_none() {
      self.keys.push_back(key);
    }
    prev_value
  }

  #[inline]
  pub fn pop_front(&mut self) -> Option<(K, V)> {
    let key = self.keys.pop_front()?;
    let value = self.elements.remove(&key)?;
    Some((key, value))
  }
}
