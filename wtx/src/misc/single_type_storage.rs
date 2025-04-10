use crate::misc::{ArrayVector, Vector};
use alloc::rc::Rc;
use core::cell::RefCell;

/// Internal trait not intended for public usage
pub trait SingleTypeStorage {
  /// Internal method not intended for public usage
  type Item;
}

impl SingleTypeStorage for () {
  type Item = ();
}

impl<T> SingleTypeStorage for &T
where
  T: SingleTypeStorage,
{
  type Item = T::Item;
}

impl<T> SingleTypeStorage for &mut T
where
  T: SingleTypeStorage,
{
  type Item = T::Item;
}

impl<T, const N: usize> SingleTypeStorage for [T; N] {
  type Item = T;
}

impl<T> SingleTypeStorage for &'_ [T] {
  type Item = T;
}

impl<T> SingleTypeStorage for &'_ mut [T] {
  type Item = T;
}

#[cfg(feature = "sync")]
impl<T> SingleTypeStorage for crate::sync::Arc<T> {
  type Item = T;
}

impl<T> SingleTypeStorage for Option<T>
where
  T: SingleTypeStorage,
{
  type Item = T::Item;
}

impl<T> SingleTypeStorage for RefCell<T> {
  type Item = T;
}

impl<T> SingleTypeStorage for Rc<T> {
  type Item = T;
}

impl<T> SingleTypeStorage for Vector<T> {
  type Item = T;
}

impl<T, const N: usize> SingleTypeStorage for ArrayVector<T, N> {
  type Item = T;
}

#[cfg(feature = "tokio")]
impl<T> SingleTypeStorage for tokio::sync::Mutex<T> {
  type Item = T;
}
