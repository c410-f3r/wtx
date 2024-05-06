use crate::misc::Lease;
use alloc::vec::Vec;
use core::{
  fmt::{Debug, Formatter},
  hint::unreachable_unchecked,
  ops::{Deref, DerefMut},
  ptr,
};

/// A wrapper around the std's vector with some additional methods to manipulate copyable data.
#[derive(Default, Eq, PartialEq)]
pub struct Vector<D> {
  data: Vec<D>,
}

impl<D> Vector<D>
where
  D: Copy,
{
  /// Constructs a new, empty instance.
  #[inline]
  pub const fn new() -> Self {
    Self { data: Vec::new() }
  }

  /// Constructs a new, empty instance with at least the specified capacity.
  #[inline]
  pub fn with_capacity(cap: usize) -> Self {
    let data = Vec::with_capacity(cap);
    // SAFETY: There is enough capacity
    unsafe {
      if data.len().unchecked_add(cap) > data.capacity() {
        unreachable_unchecked();
      }
    }
    Self { data }
  }

  /// Returns an unsafe mutable pointer to the vector's buffer, or a dangling
  /// raw pointer valid for zero sized reads if the vector didn't allocate.
  #[inline]
  pub fn as_mut_ptr(&mut self) -> *mut D {
    self.data.as_mut_ptr()
  }

  /// Returns a raw pointer to the vector's buffer, or a dangling raw pointer
  /// valid for zero sized reads if the vector didn't allocate.
  #[inline]
  pub fn as_ptr(&self) -> *const D {
    self.data.as_ptr()
  }

  /// Returns the total number of elements the vector can hold without reallocating.
  #[inline]
  pub fn capacity(&self) -> usize {
    self.data.capacity()
  }

  /// Clears the vector, removing all values.
  #[inline]
  pub fn clear(&mut self) {
    self.data.clear();
  }

  /// Iterates over the slice `other`, copies each element, and then appends
  /// it to this vector. The `other` slice is traversed in-order.
  #[inline]
  pub fn extend_from_slice(&mut self, other: &[D]) -> crate::Result<()> {
    let len = self.len();
    let other_len = other.len();
    // SAFETY: There is enough capacity
    unsafe {
      let new_len = len.unchecked_add(other_len);
      if new_len > self.data.capacity() {
        return Err(crate::Error::CapacityOverflow);
      }
      ptr::copy_nonoverlapping(other.as_ptr(), self.data.as_mut_ptr().add(len), other_len);
      self.set_len(new_len);
    }
    Ok(())
  }

  /// Generalization of [Self::extend_from_slice].
  #[allow(
    // False-positive
    clippy::missing_panics_doc
  )]
  #[inline]
  pub fn extend_from_slices<U, const N: usize>(&mut self, others: &[U; N]) -> crate::Result<()>
  where
    U: Lease<[D]>,
  {
    const {
      if N > 8 {
        panic!("It is not possible to extend more than 8 slices");
      }
    }
    let mut len: usize = 0;
    for other in others {
      // SAFETY: 8 slices is feasible by contract
      unsafe {
        len = len.unchecked_add(other.lease().len());
      }
    }
    self.reserve(len);
    for other in others {
      self.extend_from_slice(other.lease())?;
    }
    Ok(())
  }

  /// Appends an element to the back of the collection.
  ///
  /// # Panics
  ///
  /// If there is no available capacity.
  #[inline]
  pub fn push(&mut self, value: D) -> crate::Result<()> {
    let len = self.data.len();
    if len >= self.data.capacity() {
      return Err(crate::Error::CapacityOverflow);
    }
    // SAFETY: There is enough capacity
    unsafe {
      ptr::write(self.data.as_mut_ptr().add(len), value);
      self.set_len(len.unchecked_add(1));
    }
    Ok(())
  }

  /// Reserves capacity for at least `additional` more elements to be inserted
  /// in the given instance. The collection may reserve more space to
  /// speculatively avoid frequent reallocations. After calling `reserve`,
  /// capacity will be greater than or equal to `self.len() + additional`.
  /// Does nothing if capacity is already sufficient.
  ///
  /// # Panics
  ///
  /// Panics if the new capacity exceeds `isize::MAX` _bytes_.
  #[inline]
  pub fn reserve(&mut self, additional: usize) {
    self.data.reserve(additional);
    // SAFETY: There is enough capacity
    unsafe {
      if self.data.len().unchecked_add(additional) > self.data.capacity() {
        unreachable_unchecked();
      }
    }
  }

  /// Shortens the vector, keeping the first len elements and dropping the rest.
  #[inline]
  pub fn truncate(&mut self, len: usize) {
    self.data.truncate(len);
  }

  /// Forces the length of the vector to `new_len`.
  ///
  /// # Safety
  ///
  /// - `new_len` must be less than or equal to the capacity.
  /// - The elements at `prev_len..new_len` must be initialized.
  #[inline]
  pub(crate) unsafe fn set_len(&mut self, new_len: usize) {
    // Safety: up to the caller
    unsafe {
      self.data.set_len(new_len);
    }
  }
}

impl<D> Debug for Vector<D>
where
  D: Debug,
{
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), core::fmt::Error> {
    self.data.fmt(f)
  }
}

impl<D> Deref for Vector<D>
where
  D: Copy,
{
  type Target = [D];

  #[inline]
  fn deref(&self) -> &Self::Target {
    self.data.as_slice()
  }
}

impl<D> DerefMut for Vector<D>
where
  D: Copy,
{
  #[inline]
  fn deref_mut(&mut self) -> &mut Self::Target {
    self.data.as_mut_slice()
  }
}

impl<D> Lease<[D]> for Vector<D> {
  #[inline]
  fn lease(&self) -> &[D] {
    self.data.as_slice()
  }
}

#[cfg(feature = "_bench")]
#[cfg(test)]
mod bench {
  use crate::misc::vector::Vector;
  use alloc::vec::Vec;

  #[rustfmt::skip]
  macro_rules! extend_from_slice_batch {
    ($instance_cb:expr) => {
      $instance_cb(&[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]);
      $instance_cb(&[16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31]);
      $instance_cb(&[32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47]);
      $instance_cb(&[48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63]);
      $instance_cb(&[64, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79]);
      $instance_cb(&[80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 91, 92, 93, 94, 95]);
      $instance_cb(&[96, 97, 98, 99, 100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111]);
      $instance_cb(&[112, 113, 114, 115, 116, 117, 118, 119, 120, 121, 122, 123, 124, 125, 126, 127]);
      $instance_cb(&[128, 129, 130, 131, 132, 133, 134, 135, 136, 137, 138, 139, 140, 141, 142, 143]);
      $instance_cb(&[144, 145, 146, 147, 148, 149, 150, 151, 152, 153, 154, 155, 156, 157, 158, 159]);
      $instance_cb(&[160, 161, 162, 163, 164, 165, 166, 167, 168, 169, 170, 171, 172, 173, 174, 175]);
      $instance_cb(&[176, 177, 178, 179, 180, 181, 182, 183, 184, 185, 186, 187, 188, 189, 190, 191]);
      $instance_cb(&[192, 193, 194, 195, 196, 197, 198, 199, 200, 201, 202, 203, 204, 205, 206, 207]);
      $instance_cb(&[208, 209, 210, 211, 212, 213, 214, 215, 216, 217, 218, 219, 220, 221, 222, 223]);
      $instance_cb(&[224, 225, 226, 227, 228, 229, 230, 231, 232, 233, 234, 235, 236, 237, 238, 239]);
      $instance_cb(&[240, 241, 242, 243, 244, 245, 246, 247, 248, 249, 250, 251, 252, 253, 254, 255]);
    };
  }

  #[rustfmt::skip]
  macro_rules! push_batch {
    ($instance_cb:expr) => {
      $instance_cb(0); $instance_cb(1); $instance_cb(2); $instance_cb(3); $instance_cb(4); $instance_cb(5); $instance_cb(6); $instance_cb(7);
      $instance_cb(8); $instance_cb(9); $instance_cb(10); $instance_cb(11); $instance_cb(12); $instance_cb(13); $instance_cb(14); $instance_cb(15);
      $instance_cb(16); $instance_cb(17); $instance_cb(18); $instance_cb(19); $instance_cb(20); $instance_cb(21); $instance_cb(22); $instance_cb(23);
      $instance_cb(24); $instance_cb(25); $instance_cb(26); $instance_cb(27); $instance_cb(28); $instance_cb(29); $instance_cb(30); $instance_cb(31);
      $instance_cb(32); $instance_cb(33); $instance_cb(34); $instance_cb(35); $instance_cb(36); $instance_cb(37); $instance_cb(38); $instance_cb(39);
      $instance_cb(40); $instance_cb(41); $instance_cb(42); $instance_cb(43); $instance_cb(44); $instance_cb(45); $instance_cb(46); $instance_cb(47);
      $instance_cb(48); $instance_cb(49); $instance_cb(50); $instance_cb(51); $instance_cb(52); $instance_cb(53); $instance_cb(54); $instance_cb(55);
      $instance_cb(56); $instance_cb(57); $instance_cb(58); $instance_cb(59); $instance_cb(60); $instance_cb(61); $instance_cb(62); $instance_cb(63);
    };
  }

  #[bench]
  fn extend_from_slice_local(b: &mut test::Bencher) {
    let mut vec = Vector::default();
    b.iter(|| {
      vec.reserve(256 * 4);
      extend_from_slice_batch!(|elem| {
        vec.extend_from_slice(elem).unwrap();
        vec.extend_from_slice(elem).unwrap();
        vec.extend_from_slice(elem).unwrap();
        vec.extend_from_slice(elem).unwrap();
      });
    });
  }

  #[bench]
  fn extend_from_slice_std(b: &mut test::Bencher) {
    let mut vec = Vec::default();
    b.iter(|| {
      vec.reserve(256 * 4);
      extend_from_slice_batch!(|elem| {
        vec.extend_from_slice(elem);
        vec.extend_from_slice(elem);
        vec.extend_from_slice(elem);
        vec.extend_from_slice(elem);
      });
    });
  }

  #[bench]
  fn push_local(b: &mut test::Bencher) {
    let mut vec = Vector::default();
    b.iter(|| {
      vec.reserve(64 * 8);
      push_batch!(|elem| vec.push(elem).unwrap());
      push_batch!(|elem| vec.push(elem).unwrap());
      push_batch!(|elem| vec.push(elem).unwrap());
      push_batch!(|elem| vec.push(elem).unwrap());
      push_batch!(|elem| vec.push(elem).unwrap());
      push_batch!(|elem| vec.push(elem).unwrap());
      push_batch!(|elem| vec.push(elem).unwrap());
      push_batch!(|elem| vec.push(elem).unwrap());
    });
  }

  #[bench]
  fn push_std(b: &mut test::Bencher) {
    let mut vec = Vec::default();
    b.iter(|| {
      vec.reserve(64 * 8);
      push_batch!(|elem| vec.push(elem));
      push_batch!(|elem| vec.push(elem));
      push_batch!(|elem| vec.push(elem));
      push_batch!(|elem| vec.push(elem));
      push_batch!(|elem| vec.push(elem));
      push_batch!(|elem| vec.push(elem));
      push_batch!(|elem| vec.push(elem));
      push_batch!(|elem| vec.push(elem));
    });
  }
}
