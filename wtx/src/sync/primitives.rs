#![allow(clippy::disallowed_types, reason = "This is the only allowed place")]

macro_rules! create_atomic_primitive {
  ($name:ident, $ty:ty) => {
    /// An integer type which can be safely shared between threads.
    #[derive(Debug)]
    pub struct $name(
      #[cfg(feature = "portable-atomic")] portable_atomic::$name,
      #[cfg(all(feature = "loom", not(any(feature = "portable-atomic"))))]
      (std::sync::OnceLock<loom::sync::atomic::$name>, $ty),
      #[cfg(not(any(feature = "loom", feature = "portable-atomic")))] core::sync::atomic::$name,
    );

    impl $name {
      /// New instance with the given data.
      #[inline]
      pub const fn new(data: $ty) -> Self {
        Self(
          #[cfg(feature = "portable-atomic")]
          portable_atomic::$name::new(data),
          #[cfg(all(feature = "loom", not(any(feature = "portable-atomic"))))]
          (std::sync::OnceLock::new(), data),
          #[cfg(not(any(feature = "loom", feature = "portable-atomic")))]
          core::sync::atomic::$name::new(data),
        )
      }
    }

    impl core::ops::Deref for $name {
      #[cfg(feature = "portable-atomic")]
      type Target = portable_atomic::$name;
      #[cfg(all(feature = "loom", not(any(feature = "portable-atomic"))))]
      type Target = loom::sync::atomic::$name;
      #[cfg(not(any(feature = "loom", feature = "portable-atomic")))]
      type Target = core::sync::atomic::$name;

      #[inline]
      fn deref(&self) -> &Self::Target {
        #[cfg(all(feature = "loom", not(any(feature = "portable-atomic"))))]
        {
          self.0.0.get_or_init(|| loom::sync::atomic::$name::new(self.0.1))
        }
        #[cfg(not(all(feature = "loom", not(any(feature = "portable-atomic")))))]
        {
          &self.0
        }
      }
    }

    impl core::ops::DerefMut for $name {
      #[inline]
      fn deref_mut(&mut self) -> &mut Self::Target {
        // FIXME(STABLE): Use `get_mut_or_init`
        #[cfg(all(feature = "loom", not(any(feature = "portable-atomic"))))]
        {
          if self.0.0.get_mut().is_some() {
            self.0.0.get_mut().unwrap()
          } else {
            let _ = self.0.0.set(loom::sync::atomic::$name::new(self.0.1));
            self.0.0.get_mut().unwrap()
          }
        }
        #[cfg(not(all(feature = "loom", not(any(feature = "portable-atomic")))))]
        {
          &mut self.0
        }
      }
    }
  };
}

create_atomic_primitive!(AtomicBool, bool);
create_atomic_primitive!(AtomicU32, u32);
create_atomic_primitive!(AtomicU64, u64);
create_atomic_primitive!(AtomicUsize, usize);
