use crate::{
  calendar::{Datetime, Utc},
  collections::{ArrayStringU8, ArrayVectorU8, Clear},
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

type NameTy = ArrayStringU8<15>;

/// A piece of persistent data send from Client to Server.
#[derive(Debug)]
pub struct CookieGeneric<V> {
  // TODO: Use DynVector
  pub(crate) values: ArrayVectorU8<(NameTy, V), 3>,
}

impl<V> Display for CookieGeneric<V>
where
  V: Lease<str>,
{
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    let mut iter = self.values.iter();
    if let Some(elem) = iter.next() {
      f.write_fmt(format_args!("{}={}", elem.0, elem.1.lease()))?;
    }
    for elem in iter {
      f.write_fmt(format_args!("; {}={}", elem.0, elem.1.lease()))?;
    }
    Ok(())
  }
}

/// A piece of persistent data send from Server to Client.
#[derive(Debug)]
pub struct SetCookieGeneric<T, V> {
  pub(crate) domain: T,
  pub(crate) expires: Option<Datetime<Utc>>,
  pub(crate) http_only: bool,
  pub(crate) max_age: Option<Duration>,
  pub(crate) name: NameTy,
  pub(crate) path: T,
  pub(crate) same_site: Option<SameSite>,
  pub(crate) secure: bool,
  pub(crate) value: V,
}

impl<T, V> SetCookieGeneric<T, V> {
  /// Appends a cookie headers that removes itself
  #[inline]
  pub fn delete(&mut self, headers: &mut Headers) -> crate::Result<()>
  where
    T: Lease<str>,
    V: Clear,
  {
    let prev_expires = self.expires;
    let prev_max_age = self.max_age;
    self.expires = Some(Datetime::EPOCH);
    self.max_age = Some(Duration::ZERO);
    self.value.clear();
    let rslt = headers.push_from_fmt(Header::from_name_and_value(
      KnownHeaderName::SetCookie.into(),
      format_args!("{}", self.map_mut(move |el| el, |_| "")),
    ));
    self.expires = prev_expires;
    self.max_age = prev_max_age;
    rslt
  }

  /// Maps all generic types
  #[inline]
  pub fn map_mut<'this, NT, NV>(
    &'this mut self,
    mut data: impl FnMut(&'this mut T) -> NT,
    value: impl FnOnce(&'this mut V) -> NV,
  ) -> SetCookieGeneric<NT, NV> {
    SetCookieGeneric {
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

impl<T, V> Display for SetCookieGeneric<T, V>
where
  T: Lease<str>,
  V: Lease<str>,
{
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    f.write_fmt(format_args!("{}={}", self.name, self.value.lease()))?;
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
