use crate::collections::{LinearStorageLen, TryExtend};
use alloc::alloc::Global;
use core::{
  alloc::{Allocator, Layout},
  mem::{DropGuard, MaybeUninit},
  ptr::{self, NonNull},
  slice,
};

/// Calculates the current index of a previous `push_front` element.
///
/// `elem_idx` and `elem_idx_last` should be orchestrated by an index that is always increasing
/// at each `push_front` invocation.
///
/// ```rust
/// use wtx::collections::{Deque, backward_deque_idx};
///
/// let mut deque = Deque::new();
///
/// let idx_of_first_elem = 0;
/// deque.push_front(1).unwrap();
/// assert_eq!(deque.get(backward_deque_idx(idx_of_first_elem, idx_of_first_elem)), Some(&1));
///
/// let idx_of_second_elem = 1;
/// deque.push_front(2).unwrap();
/// assert_eq!(deque.get(backward_deque_idx(idx_of_first_elem, idx_of_second_elem)), Some(&1));
/// assert_eq!(deque.get(backward_deque_idx(idx_of_second_elem, idx_of_second_elem)), Some(&2));
///
/// let idx_of_third_elem = 2;
/// deque.push_front(3).unwrap();
/// assert_eq!(deque.get(backward_deque_idx(idx_of_first_elem, idx_of_third_elem)), Some(&1));
/// assert_eq!(deque.get(backward_deque_idx(idx_of_second_elem, idx_of_third_elem)), Some(&2));
/// assert_eq!(deque.get(backward_deque_idx(idx_of_third_elem, idx_of_third_elem)), Some(&3));
/// ```
#[inline]
pub const fn backward_deque_idx(elem_idx: usize, elem_idx_last: usize) -> usize {
  elem_idx_last.wrapping_sub(elem_idx) % (usize::MAX >> 1)
}

/// Concatenates all slices into a single contiguous chunk of memory.
///
/// Returns `None` if `N` differs from the sum of all lengths.
#[expect(clippy::indexing_slicing, reason = "there are no out-of-bound reads or writes")]
#[inline]
pub const fn concat_slices<T, const N: usize>(slices: &[&[T]]) -> Option<[T; N]>
where
  T: Copy,
{
  let mut concatenated: [MaybeUninit<T>; N] = [MaybeUninit::uninit(); N];
  let mut idx_outer = 0;
  let mut slice_offset: usize = 0;
  while idx_outer < slices.len() {
    let slice = slices[idx_outer];
    let slice_len = slice.len();
    if slice_len > N.wrapping_sub(slice_offset) {
      return None;
    }
    // SAFETY: the above check prevents an out-of-bounds `slice_offset`
    let dst = unsafe { concatenated.as_mut_ptr().add(slice_offset).cast::<T>() };
    // SAFETY: source and destination point to `slice.len()` elements
    unsafe {
      ptr::copy_nonoverlapping(slice.as_ptr(), dst, slice_len);
    }
    slice_offset = slice_offset.wrapping_add(slice_len);
    idx_outer = idx_outer.wrapping_add(1);
  }
  if slice_offset != N {
    return None;
  }
  // SAFETY: All elements have been initialized
  Some(unsafe { ptr::read(concatenated.as_ptr().cast()) })
}

pub(crate) struct HeapDeallocator<'alloc, A: Allocator> {
  align: usize,
  allocator: &'alloc A,
  len_bytes: usize,
  pointer: NonNull<u8>,
}

impl<'alloc, A> HeapDeallocator<'alloc, A>
where
  A: Allocator,
{
  #[inline]
  pub(crate) const unsafe fn new<E>(
    allocator: &'alloc A,
    capacity: usize,
    pointer: NonNull<u8>,
  ) -> Self {
    Self {
      align: align_of::<E>(),
      allocator,
      len_bytes: capacity.wrapping_mul(size_of::<E>()),
      pointer,
    }
  }
}

impl<A> Drop for HeapDeallocator<'_, A>
where
  A: Allocator,
{
  #[inline]
  fn drop(&mut self) {
    if self.len_bytes == 0 {
      return;
    }
    // SAFETY: Up to the caller
    let layout = unsafe { Layout::from_size_align_unchecked(self.len_bytes, self.align) };
    // SAFETY: Up to the caller
    unsafe {
      self.allocator.deallocate(self.pointer, layout);
    }
  }
}

pub(crate) fn is_char_boundary(idx: usize, slice: &[u8]) -> bool {
  if idx == 0 {
    return true;
  }
  if idx >= slice.len() {
    idx == slice.len()
  } else {
    let byte = slice.get(idx).copied().unwrap_or_default();
    !(128..192).contains(&byte)
  }
}

/// This methods becomes infallible if `buffer` is `&mut ()`.
pub(crate) unsafe fn drop_elements<B, L, T>(
  buffer: &mut B,
  len: L,
  offset: L,
  ptr: *mut T,
) -> crate::Result<()>
where
  B: TryExtend<[T; 1]>,
  L: LinearStorageLen,
{
  // SAFETY: caller ensures `ptr` is valid and `offset` is in-bounds.
  let data = unsafe { ptr.add(offset.usize()) };
  if B::IS_UNIT {
    // SAFETY: caller ensures that `len` describes a valid, initialized slice region.
    let elements = unsafe { slice::from_raw_parts_mut(data, len.usize()) };
    // SAFETY: per contract, slice points to initialized data.
    unsafe {
      ptr::drop_in_place(elements);
    }
  } else {
    let mut guard = DropGuard::new((data, len.usize()), |(local_data, local_len)| {
      if local_len > 0 {
        // SAFETY: it is up to the caller to provide a valid slice
        let slice_to_drop = unsafe { slice::from_raw_parts_mut(local_data, local_len) };
        // SAFETY: it is up to the caller to provide a valid slice
        unsafe {
          ptr::drop_in_place(slice_to_drop);
        }
      }
    });
    while guard.1 > 0 {
      let src = guard.0;
      // SAFETY: `src` points to an initialized element per the function's contract.
      guard.0 = unsafe { src.add(1) };
      guard.1 = guard.1.wrapping_sub(1);
      // SAFETY: `src` points to an initialized element per the function's contract.
      let value = unsafe { ptr::read(src) };
      buffer.try_extend([value])?;
    }
    let _ = DropGuard::dismiss(guard);
  }
  Ok(())
}

#[inline]
pub(crate) unsafe fn grow_heap<L, T>(
  new_capacity: L,
  old_capacity: L,
  ptr: NonNull<u8>,
) -> Option<NonNull<[u8]>>
where
  L: LinearStorageLen,
{
  if new_capacity > L::UPPER_BOUND {
    return None;
  }
  let new_layout = Layout::array::<T>(new_capacity.usize()).ok()?;
  let old_layout = Layout::array::<T>(old_capacity.usize()).ok()?;
  if old_capacity < new_capacity {
    // SAFETY: Up to the caller to provide valid parameters
    unsafe { Global.grow(ptr, old_layout, new_layout).ok() }
  } else {
    // SAFETY: Up to the caller to provide valid parameters
    unsafe { Global.shrink(ptr, old_layout, new_layout).ok() }
  }
}

#[inline]
pub(crate) fn reserve_new_capacity<L>(additional: L, cap: L, len: L) -> Option<L>
where
  L: LinearStorageLen,
{
  let required = len.checked_add(additional)?;
  if cap >= required {
    return Some(cap);
  }
  let double_capacity = cap.saturating_mul(L::TWO);
  let target = required.max(double_capacity);
  if target > L::UPPER_BOUND {
    return None;
  }
  Some(target.checked_next_power_of_two().unwrap_or(target))
}

#[cfg(test)]
mod tests {
  use crate::{
    collections::{ArrayVectorU8, VectorUsize, misc::drop_elements},
    sync::{Arc, AtomicUsize},
  };
  use core::sync::atomic::Ordering;

  #[derive(Debug)]
  struct DropSpyManager {
    counter: Arc<AtomicUsize>,
  }

  impl DropSpyManager {
    fn new() -> Self {
      Self { counter: Arc::new(AtomicUsize::new(0)) }
    }

    fn counter(&self) -> usize {
      self.counter.load(Ordering::SeqCst)
    }

    fn spawn(&self) -> DropSpySpawn {
      DropSpySpawn { counter: self.counter.clone() }
    }
  }

  #[derive(Debug)]
  struct DropSpySpawn {
    counter: Arc<AtomicUsize>,
  }

  impl Drop for DropSpySpawn {
    fn drop(&mut self) {
      let _ = self.counter.fetch_add(1, Ordering::SeqCst);
    }
  }

  #[test]
  fn concatenation() {
    assert_eq!(concat_slices!(b"a", b"b", b"c"), *b"abc");
    assert_eq!(concat_strings!("a", "b", "c"), "abc");
  }

  #[test]
  fn drops_all_elements_with_buffer() {
    let dpm = DropSpyManager::new();
    let mut buffer = ArrayVectorU8::<_, 5>::new();
    let mut vec = VectorUsize::from_iterator((0..5).map(|_| dpm.spawn())).unwrap();
    unsafe {
      drop_elements(&mut buffer, 5u32, 0, vec.as_mut_ptr()).unwrap();
      vec.set_len(0);
    }
    assert_eq!(buffer.len(), 5);
    assert_eq!(dpm.counter(), 0);
  }

  #[test]
  fn drops_all_elements_without_buffer() {
    let dpm = DropSpyManager::new();
    let mut vec = VectorUsize::from_iterator((0..5).map(|_| dpm.spawn())).unwrap();
    unsafe {
      drop_elements(&mut (), 5u32, 0, vec.as_mut_ptr()).unwrap();
      vec.set_len(0);
    }
    assert_eq!(dpm.counter(), 5);
  }

  #[test]
  fn drops_some_elements() {
    let dpm = DropSpyManager::new();
    let mut buffer = ArrayVectorU8::<_, 2>::new();
    let mut vec = VectorUsize::from_iterator((0..5).map(|_| dpm.spawn())).unwrap();
    unsafe {
      let _rslt = drop_elements(&mut buffer, 5u32, 0, vec.as_mut_ptr());
      vec.set_len(0);
    }
    assert_eq!(buffer.len(), 2);
    assert_eq!(dpm.counter(), 3);
  }
}
