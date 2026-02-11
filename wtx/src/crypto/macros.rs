macro_rules! _create_wrapper {
  ($(($name:ident $(, $ty:ty)?)),* $(,)?) => {
    $(
      /// A wrapper used to generalize third-party dependencies.
      #[derive(Debug)]
      pub struct $name {
        $(pub(crate) value: $ty)?
      }

      impl $name {
        /// New instance
        #[inline]
        pub const fn new($(value: $ty)?) -> Self {
          Self {
            $(value: {
              fn _expander(_: $ty) {}
              value
            })?
          }
        }
      }
    )*
  };
}
