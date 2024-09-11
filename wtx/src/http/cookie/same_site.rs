use core::fmt::{Display, Formatter};

/// Controls whether or not a cookie is sent with cross-site requests
#[derive(Clone, Copy, Debug)]
pub enum SameSite {
  /// Means that the is sent when a user is navigating to the origin site from an external site.
  Lax,
  /// Means that the browser sends the cookie with both cross-site and same-site requests.
  None,
  /// Means that the browser sends the cookie only for same-site requests
  Strict,
}

impl Display for SameSite {
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    match *self {
      Self::Lax => write!(f, "Lax"),
      Self::None => write!(f, "None"),
      Self::Strict => write!(f, "Strict"),
    }
  }
}
