use crate::collection::{LinearStorageLen, TryExtend};
use core::{mem, ptr, slice};

/// Calculates the current index of a previous `push_front` element.
///
/// `elem_idx` and `elem_idx_last` should be orchestrated by an index that is always increasing
/// at each `push_front` invocation.
///
/// ```rust
/// use wtx::collection::{Deque, backward_deque_idx};
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
  elem_idx_last.wrapping_sub(elem_idx) % 9_223_372_036_854_775_807
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
    let mut guard = SliceDropGuard { data, begin: 0, len: len.usize() };
    while guard.begin < guard.len {
      // SAFETY: caller ensures `len` is valid, so `data + begin` is in-bounds.
      let src = unsafe { guard.data.add(guard.begin) };
      guard.begin = guard.begin.wrapping_add(1);
      // SAFETY: `src` points to an initialized element per the function's contract.
      let value = unsafe { ptr::read(src) };
      buffer.try_extend([value])?;
    }
    #[expect(
      clippy::mem_forget,
      reason = "there is nothing else to drop but it is possible to avoid one arithmetic operation"
    )]
    mem::forget(guard);
  }
  Ok(())
}

struct SliceDropGuard<T> {
  data: *mut T,
  begin: usize,
  len: usize,
}

impl<T> Drop for SliceDropGuard<T> {
  fn drop(&mut self) {
    let remaining_len = self.len.wrapping_sub(self.begin);
    if remaining_len > 0 {
      // SAFETY: it is up to the caller to provide a valid slice
      let data = unsafe { self.data.add(self.begin) };
      // SAFETY: it is up to the caller to provide a valid slice
      let slice_to_drop = unsafe { slice::from_raw_parts_mut(data, remaining_len) };
      // SAFETY: it is up to the caller to provide a valid slice
      unsafe {
        ptr::drop_in_place(slice_to_drop);
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use crate::{
    collection::{ArrayVectorU8, Vector, misc::drop_elements},
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
  fn drops_all_elements_with_buffer() {
    let dpm = DropSpyManager::new();
    let mut buffer = ArrayVectorU8::<_, 5>::new();
    let mut vec = Vector::from_iter((0..5).map(|_| dpm.spawn())).unwrap();
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
    let mut vec = Vector::from_iter((0..5).map(|_| dpm.spawn())).unwrap();
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
    let mut vec = Vector::from_iter((0..5).map(|_| dpm.spawn())).unwrap();
    unsafe {
      let _rslt = drop_elements(&mut buffer, 5u32, 0, vec.as_mut_ptr());
      vec.set_len(0);
    }
    assert_eq!(buffer.len(), 2);
    assert_eq!(dpm.counter(), 3);
  }
}
