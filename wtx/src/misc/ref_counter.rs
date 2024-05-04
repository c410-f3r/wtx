use alloc::{rc::Rc, sync::Arc};
use core::ops::Deref;

/// Reference Counter
///
/// Stores the number of references, pointers, or handles to a resource, such as an object, a
/// block of memory, disk space and others.
pub trait RefCounter: Clone + Deref<Target = Self::Item> {
  /// Item behind this counter.
  type Item;

  /// Generic way to build a reference counter.
  fn new(elem: Self::Item) -> Self;
}

impl<T> RefCounter for Arc<T> {
  type Item = T;

  #[inline]
  fn new(elem: Self::Item) -> Self {
    Arc::new(elem)
  }
}

impl<T> RefCounter for Rc<T> {
  type Item = T;

  #[inline]
  fn new(elem: Self::Item) -> Self {
    Rc::new(elem)
  }
}
