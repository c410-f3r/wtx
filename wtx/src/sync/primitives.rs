#![allow(clippy::disallowed_types, reason = "This is the only allowed place")]

macro_rules! create_atomic_primitive {
  ($local_ty:ident, $name:ident, $ty:ty) => {
    #[cfg(feature = "portable-atomic")]
    type $local_ty = portable_atomic::$name;
    #[cfg(all(feature = "loom", not(feature = "portable-atomic")))]
    type $local_ty = loom::sync::atomic::$name;
    #[cfg(all(not(feature = "portable-atomic"), not(feature = "loom")))]
    type $local_ty = core::sync::atomic::$name;

    /// An integer type which can be safely shared between threads.
    #[derive(Debug)]
    pub struct $name($local_ty);

    impl $name {
      /// New instance with the given data.
      #[cfg(all(feature = "loom", not(feature = "portable-atomic")))]
      #[inline]
      pub fn new(data: $ty) -> Self {
        Self($local_ty::new(data))
      }
      /// New instance with the given data.
      #[cfg(any(not(feature = "loom"), feature = "portable-atomic"))]
      #[inline]
      pub const fn new(data: $ty) -> Self {
        Self($local_ty::new(data))
      }
    }

    impl core::ops::Deref for $name {
      type Target = $local_ty;

      #[inline]
      fn deref(&self) -> &Self::Target {
        &self.0
      }
    }

    impl core::ops::DerefMut for $name {
      #[inline]
      fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
      }
    }
  };
}

create_atomic_primitive!(LocalAtomicBool, AtomicBool, bool);
create_atomic_primitive!(LocalAtomicU32, AtomicU32, u32);
create_atomic_primitive!(LocalAtomicU64, AtomicU64, u64);
create_atomic_primitive!(LocalAtomicUsize, AtomicUsize, usize);
