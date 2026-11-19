use core::{
  mem::{ManuallyDrop, MaybeUninit},
  ptr::NonNull,
};

#[repr(C)]
pub(crate) union DynVectorData<T, const N: usize> {
  pub(crate) stack: ManuallyDrop<[MaybeUninit<T>; N]>,
  pub(crate) heap: NonNull<u8>,
}

impl<T, const N: usize> DynVectorData<T, N> {
  #[inline]
  pub(crate) const fn new() -> Self {
    Self { stack: ManuallyDrop::new([const { MaybeUninit::uninit() }; N]) }
  }

  #[inline]
  pub(crate) const fn as_ptr_mut_heap(&mut self) -> *mut NonNull<u8> {
    &raw mut self.heap
  }

  #[inline]
  pub(crate) const fn as_ptr_mut_stack(&mut self) -> *mut ManuallyDrop<[MaybeUninit<T>; N]> {
    &raw mut self.stack
  }

  #[inline]
  pub(crate) const fn as_ptrs(
    &self,
  ) -> (*const NonNull<u8>, *const ManuallyDrop<[MaybeUninit<T>; N]>) {
    (&raw const self.heap, &raw const self.stack)
  }

  #[inline]
  pub(crate) const fn as_ptrs_mut(
    &mut self,
  ) -> (*mut NonNull<u8>, *mut ManuallyDrop<[MaybeUninit<T>; N]>) {
    (&raw mut self.heap, &raw mut self.stack)
  }
}
