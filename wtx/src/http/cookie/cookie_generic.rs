use crate::{
  calendar::{DateTime, Utc},
  collection::{ArrayStringU8, Clear},
  http::{
    Header, Headers, KnownHeaderName,
    cookie::{FMT1, SameSite},
  },
  misc::Lease,
};
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
  pub(crate) name: ArrayStringU8<15>,
  pub(crate) path: T,
  pub(crate) same_site: Option<SameSite>,
  pub(crate) secure: bool,
  pub(crate) value: V,
}

impl<T, V> CookieGeneric<T, V> {
  pub(crate) fn delete(&mut self, headers: &mut Headers) -> crate::Result<()>
  where
    T: Lease<str>,
    V: Clear,
  {
    let prev_expires = self.expires;
    let prev_max_age = self.max_age;
    self.expires = Some(DateTime::EPOCH);
    self.max_age = None;
    self.value.clear();
    let rslt = headers.push_from_fmt(Header::from_name_and_value(
      KnownHeaderName::SetCookie.into(),
      format_args!("{}", self.map_mut(move |el| el, |_| "")),
    ));
    self.expires = prev_expires;
    self.max_age = prev_max_age;
    rslt
  }

  pub(crate) fn map_mut<'this, NT, NV>(
    &'this mut self,
    mut data: impl FnMut(&'this mut T) -> NT,
    value: impl FnOnce(&'this mut V) -> NV,
  ) -> CookieGeneric<NT, NV> {
    CookieGeneric {
      domain: data(&mut self.domain),
      expires: self.expires,
      http_only: self.http_only,
      max_age: self.max_age,
      name: self.name,
      path: data(&mut self.path),
      same_site: self.same_site,
      secure: self.secure,
      value: value(&mut self.value),
    }
  }
}

impl<T, V> Display for CookieGeneric<T, V>
where
  T: Lease<str>,
  V: Lease<str>,
{
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    f.write_fmt(format_args!("{}={}", &self.name, self.value.lease()))?;
    if !self.domain.lease().is_empty() {
      f.write_fmt(format_args!("; Domain={}", self.domain.lease()))?;
    }
    if let Some(elem) = self.expires {
      f.write_fmt(format_args!(
        "; Expires={}",
        elem.to_string::<32>(FMT1.iter().copied()).map_err(|_err| core::fmt::Error)?
      ))?;
    }
    if self.http_only {
      f.write_str("; HttpOnly")?;
    }
    if let Some(elem) = self.max_age {
      f.write_fmt(format_args!("; Max-Age={}", elem.as_secs()))?;
    }
    if !self.path.lease().is_empty() {
      f.write_fmt(format_args!("; Path={}", self.path.lease()))?;
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
