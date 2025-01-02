use crate::{
  http::cookie::{SameSite, FMT1},
  misc::{BytesFmt, Lease},
};
use chrono::{DateTime, Utc};
use core::{
  fmt::{Display, Formatter},
  time::Duration,
};

#[derive(Debug)]
pub(crate) struct CookieGeneric<T, V> {
  pub(crate) domain: T,
  pub(crate) expires: Option<DateTime<Utc>>,
  pub(crate) http_only: bool,
  pub(crate) max_age: Option<Duration>,
  pub(crate) name: T,
  pub(crate) path: T,
  pub(crate) same_site: Option<SameSite>,
  pub(crate) secure: bool,
  pub(crate) value: V,
}

impl<T, V> Display for CookieGeneric<T, V>
where
  T: Lease<[u8]>,
  V: Lease<[u8]>,
{
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    f.write_fmt(format_args!("{}={}", BytesFmt(self.name.lease()), BytesFmt(self.value.lease())))?;
    if !self.domain.lease().is_empty() {
      f.write_fmt(format_args!("; Domain={}", BytesFmt(self.domain.lease())))?;
    }
    if let Some(elem) = self.expires {
      f.write_fmt(format_args!("; Expires={}", elem.format(FMT1)))?;
    }
    if self.http_only {
      f.write_str("; HttpOnly")?;
    }
    if let Some(elem) = self.max_age {
      f.write_fmt(format_args!("; Max-Age={}", elem.as_secs()))?;
    }
    if !self.path.lease().is_empty() {
      f.write_fmt(format_args!("; Path={}", BytesFmt(self.path.lease())))?;
    }
    if let Some(elem) = self.same_site {
      f.write_fmt(format_args!("; SameSite={elem}"))?;
      if matches!(elem, SameSite::None) && !self.secure {
        f.write_str("; Secure")?;
      }
    }
    if self.secure {
      f.write_str("; Secure")?;
    }
    Ok(())
  }
}
