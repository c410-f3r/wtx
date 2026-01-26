macro_rules! _create_cipher_suite_wrapper {
  ($(($name:ident, $ty:ty, $value:expr))*) => {
    $(
      /// A wrapper used to generalize third-party dependencies. Doesn't contain any actual code.
      pub struct $name(pub(crate) $ty);

      impl $name {
        /// New instance
        #[inline]
        pub const fn new() -> Self {
          Self($value)
        }
      }
    )*
  };
}
