/// Shortcut that avoids having to explicit import types related to paths.
#[macro_export]
macro_rules! paths {
  (
    $( ( $name:expr, $value:expr $(,)? ) ),+ $(,)?
  ) => {
    $crate::http::server_framework::Paths::new(
      ( $( $crate::http::server_framework::Path::new($name, $value), )+ )
    )
  };
}
