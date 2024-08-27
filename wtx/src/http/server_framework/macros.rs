/// Shortcut that avoids having to explicit import types related to paths.
#[macro_export]
macro_rules! paths {
  (
    $( ( $name:expr, $value:expr $(,)? ) ),+ $(,)?
  ) => {
    ( $( $crate::http::server_framework::PathParams::new($name, $value), )+ )
  };
}
