use crate::collections::{
  DynVectorError, SparseLinearStorageLen,
  dyn_vector::dyn_vector_data::DynVectorData,
  linear_storage::{LinearStorage, linear_storage_mut::LinearStorageMut},
  misc::{HeapDeallocator, drop_elements, grow_heap, reserve_new_capacity},
};
use alloc::alloc::Global;
use core::{
  alloc::{Allocator as _, Layout},
  ptr,
};

pub(crate) struct DynVectorInner<L, T, const N: usize>
where
  L: SparseLinearStorageLen,
{
  data: DynVectorData<T, N>,
  cap: L,
  len: L,
}

impl<L, T, const N: usize> DynVectorInner<L, T, N>
where
  L: SparseLinearStorageLen,
{
  #[inline]
  pub(crate) const fn new() -> Self {
    Self { data: DynVectorData::new(), cap: L::ZERO, len: L::ZERO }
  }

  #[inline]
  pub(crate) fn is_heap(&self) -> bool {
    self.cap.has_spare_bit()
  }

  #[inline]
  pub(crate) fn is_stack(&self) -> bool {
    !self.is_heap()
  }

  #[inline]
  fn capacity_upper_bound_stack() -> L {
    if Self::IS_ZST { L::UPPER_BOUND } else { L::from_usize(N).unwrap_or_default() }
  }

  #[inline]
  fn grow(&mut self, additional: L, new_capacity: L) -> crate::Result<()> {
    if Self::IS_ZST {
      self.cap = new_capacity;
      return Ok(());
    }
    let cap = self.capacity();
    let len = self.len();
    let requires_heap = new_capacity > Self::capacity_upper_bound_stack();
    if requires_heap {
      let opt = self.grow_heap(new_capacity);
      if opt.is_some() {
        self.cap = new_capacity;
        self.turn_into_heap();
        Ok(())
      } else {
        Err(
          DynVectorError::ReserveOverflow {
            additional: additional.usize().try_into().unwrap_or(u16::MAX),
            curr: len.usize().try_into().unwrap_or(u32::MAX),
            max: L::UPPER_BOUND.usize().try_into().unwrap_or(u32::MAX),
          }
          .into(),
        )
      }
    } else {
      if self.is_stack() {
        self.cap = new_capacity;
        return Ok(());
      }
      let (heap_ptr, stack_ptr) = self.data.as_ptrs_mut();
      // SAFETY: `heap_pointer` is valid because the instance is on the heap.
      let heap_ptr_nn = unsafe { *heap_ptr };
      // SAFETY: `heap_ptr` points to `cap` allocated elements and the stack has room for
      //         `new_capacity` elements, with `len <= new_capacity`.
      unsafe {
        ptr::copy_nonoverlapping(
          heap_ptr_nn.as_ptr().cast::<T>(),
          stack_ptr.cast::<T>(),
          len.usize(),
        );
      }
      // SAFETY: `heap_ptr` was allocated with `cap` elements of `T`.
      drop(unsafe { HeapDeallocator::new::<T>(&Global, cap.usize(), heap_ptr_nn) });
      self.cap = new_capacity;
      self.turn_into_stack();
      Ok(())
    }
  }

  #[inline]
  fn grow_heap(&mut self, new_capacity: L) -> Option<()> {
    let len = self.len;
    let is_stack = self.is_stack();
    let new_layout = Layout::array::<T>(new_capacity.usize()).ok()?;
    let new_ptr: ptr::NonNull<u8> = if is_stack {
      let new_ptr = Global.allocate(new_layout).ok()?;
      // SAFETY: `new_ptr` points to `new_capacity` allocated elements and the stack holds
      //         `len` initialized elements.
      unsafe {
        ptr::copy_nonoverlapping(
          self.data.as_ptr_mut_stack().cast::<T>(),
          new_ptr.cast::<T>().as_ptr(),
          len.usize(),
        );
      }
      new_ptr.cast()
    } else {
      // SAFETY: `self.data.heap` is valid because the instance is on the heap.
      let ptr = unsafe { *self.data.as_ptr_mut_heap() };
      // SAFETY: `ptr` was allocated with `self.capacity()` elements of `T`.
      unsafe { grow_heap::<_, T>(new_capacity, self.capacity(), ptr)?.cast() }
    };
    self.data.heap = new_ptr;
    Some(())
  }

  #[inline]
  fn turn_into_heap(&mut self) {
    self.cap.set_spare_bit(true);
  }

  #[inline]
  fn turn_into_stack(&mut self) {
    self.cap.set_spare_bit(false);
  }
}

impl<L, T, const N: usize> LinearStorage<T> for DynVectorInner<L, T, N>
where
  L: SparseLinearStorageLen,
{
  type Len = L;
  type Slice = [T];

  #[inline]
  fn as_ptr(&self) -> *const T {
    let (heap_ptr, stack_ptr) = self.data.as_ptrs();
    if self.is_heap() {
      // SAFETY: `heap_ptr` is valid because the instance is on the heap.
      unsafe { (*heap_ptr).as_ptr().cast() }
    } else {
      stack_ptr.cast()
    }
  }

  #[inline]
  fn capacity(&self) -> Self::Len {
    self.cap & L::UPPER_BOUND
  }

  #[inline]
  fn len(&self) -> Self::Len {
    self.len
  }
}

impl<L, T, const N: usize> LinearStorageMut<T> for DynVectorInner<L, T, N>
where
  L: SparseLinearStorageLen,
{
  #[inline]
  fn as_ptr_mut(&mut self) -> *mut T {
    if self.is_heap() {
      // SAFETY: `self.data.heap` is valid because the instance is on the heap.
      unsafe { (*self.data.as_ptr_mut_heap()).as_ptr().cast() }
    } else {
      self.data.as_ptr_mut_stack().cast()
    }
  }

  #[inline]
  fn reserve(&mut self, additional: Self::Len) -> crate::Result<()> {
    let cap = self.capacity();
    let len = self.len;
    let new_capacity = reserve_new_capacity(additional, cap, len).ok_or_else(|| {
      DynVectorError::ReserveOverflow {
        additional: additional.usize().try_into().unwrap_or(u16::MAX),
        curr: len.usize().try_into().unwrap_or(u32::MAX),
        max: L::UPPER_BOUND.usize().try_into().unwrap_or(u32::MAX),
      }
    })?;
    if new_capacity <= cap {
      return Ok(());
    }
    self.grow(additional, new_capacity)
  }

  #[inline]
  fn reserve_exact(&mut self, additional: Self::Len) -> crate::Result<()> {
    let cap = self.capacity();
    let len = self.len;
    let required = len.checked_add(additional).ok_or_else(|| DynVectorError::ReserveOverflow {
      additional: additional.usize().try_into().unwrap_or(u16::MAX),
      curr: len.usize().try_into().unwrap_or(u32::MAX),
      max: L::UPPER_BOUND.usize().try_into().unwrap_or(u32::MAX),
    })?;
    if required <= cap {
      return Ok(());
    }
    self.grow(additional, required)
  }

  #[inline]
  unsafe fn set_len(&mut self, new_len: Self::Len) {
    self.len = new_len;
  }
}

impl<L, T, const N: usize> Default for DynVectorInner<L, T, N>
where
  L: SparseLinearStorageLen,
{
  #[inline]
  fn default() -> Self {
    Self::new()
  }
}

impl<L, T, const N: usize> Drop for DynVectorInner<L, T, N>
where
  L: SparseLinearStorageLen,
{
  #[inline]
  fn drop(&mut self) {
    if Self::NEEDS_DROP {
      // SAFETY: `self.len` elements are initialized.
      let _rslt = unsafe { drop_elements(&mut (), self.len, L::ZERO, self.as_ptr_mut()) };
    }
    if self.is_heap() {
      let cap = self.capacity();
      // SAFETY: `self.data.heap` is valid because the instance is on the heap.
      let heap_ptr = unsafe { *self.data.as_ptr_mut_heap() };
      // SAFETY: `heap_ptr` was allocated with `cap` elements of `T`.
      drop(unsafe { HeapDeallocator::new::<T>(&Global, cap.usize(), heap_ptr) });
    }
  }
}

// SAFETY: there is no immutable method internally operating mutable modifications
unsafe impl<L, T, const N: usize> Send for DynVectorInner<L, T, N>
where
  L: SparseLinearStorageLen,
  T: Send,
{
}

// SAFETY: there is no immutable method internally operating mutable modifications
unsafe impl<L, T, const N: usize> Sync for DynVectorInner<L, T, N>
where
  L: SparseLinearStorageLen,
  T: Sync,
{
}
