use alloc::{rc::Rc, sync::Arc};
use core::ops::Deref;

/// Reference Counter
///
/// Stores the number of references, pointers, or handles to a resource, such as an object, a
/// block of memory, disk space and others.
pub trait RefCounter<T>: Clone + Deref<Target = T>
where
  T: ?Sized,
{
  /// Generic way to build a reference counter.
  fn new(elem: T) -> Self;
}

impl<T> RefCounter<T> for Arc<T> {
  #[inline]
  fn new(elem: T) -> Self {
    Arc::new(elem)
  }
}

impl<T> RefCounter<T> for Rc<T> {
  #[inline]
  fn new(elem: T) -> Self {
    Rc::new(elem)
  }
}
