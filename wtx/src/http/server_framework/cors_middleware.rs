use crate::{
  http::{
    server_framework::{ConnAux, ResMiddleware},
    Header, KnownHeaderName, Method, ReqResBuffer, ReqResDataMut, Response,
  },
  misc::ArrayVector,
};

/// Cross-origin resource sharing
#[derive(Debug)]
pub struct CorsMiddleware {
  allow_credentials: bool,
  allow_headers: ArrayVector<&'static str, 8>,
  allow_methods: (bool, ArrayVector<Method, 9>),
  allow_origin: Option<&'static str>,
  expose_headers: ArrayVector<&'static str, 8>,
  max_age: Option<u32>,
}

impl CorsMiddleware {
  /// New empty instance
  #[inline]
  pub const fn new() -> Self {
    Self {
      allow_credentials: false,
      allow_headers: ArrayVector::new(),
      allow_methods: (false, ArrayVector::new()),
      allow_origin: None,
      expose_headers: ArrayVector::new(),
      max_age: None,
    }
  }

  /// * All request headers allowed.
  /// * All methods are allowed.
  /// * All origins are allowed.
  /// * All headers are exposed.
  #[inline]
  #[must_use]
  pub fn permissive() -> Self {
    Self::new().allow_headers(["*"]).allow_methods(true, []).allow_origin("*").expose_headers(["*"])
  }

  /// Like [`Self::permissive`] with the additional feature of allowing credentials.
  #[inline]
  #[must_use]
  pub fn unrestricted() -> Self {
    Self::new().allow_credentials().allow_headers(["*"]).allow_methods(true, []).allow_origin("*")
  }

  /// <https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Access-Control-Allow-Credentials>
  #[inline]
  #[must_use]
  pub const fn allow_credentials(mut self) -> Self {
    self.allow_credentials = true;
    self
  }

  /// <https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Access-Control-Allow-Headers>
  #[inline]
  #[must_use]
  pub fn allow_headers(mut self, elem: impl IntoIterator<Item = &'static str>) -> Self {
    let _rslt = self.allow_headers.extend_from_iter(elem);
    self
  }

  /// <https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Access-Control-Allow-Methods>
  #[inline]
  #[must_use]
  pub fn allow_methods(
    mut self,
    is_all: bool,
    specifics: impl IntoIterator<Item = Method>,
  ) -> Self {
    if is_all {
      self.allow_methods.0 = true;
    } else {
      self.allow_methods.0 = false;
      self.allow_methods.1.clear();
      let _rslt = self.allow_methods.1.extend_from_iter(specifics);
    }
    self
  }

  /// <https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Access-Control-Allow-Origin>
  #[inline]
  #[must_use]
  pub const fn allow_origin(mut self, elem: &'static str) -> Self {
    self.allow_origin = Some(elem);
    self
  }

  /// <https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Access-Control-Expose-Headers>
  #[inline]
  #[must_use]
  pub fn expose_headers(mut self, elem: impl IntoIterator<Item = &'static str>) -> Self {
    let _rslt = self.expose_headers.extend_from_iter(elem);
    self
  }

  /// <https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Access-Control-Max-Age>
  #[inline]
  #[must_use]
  pub fn max_age(mut self, elem: u32) -> Self {
    self.max_age = Some(elem);
    self
  }
}

impl ConnAux for CorsMiddleware {
  type Init = CorsMiddleware;

  #[inline]
  fn conn_aux(init: Self::Init) -> crate::Result<Self> {
    Ok(init)
  }
}

impl<CA, E, RA> ResMiddleware<CA, E, RA> for CorsMiddleware
where
  E: From<crate::Error>,
{
  #[inline]
  async fn apply_res_middleware(
    &self,
    _: &mut CA,
    _: &mut RA,
    res: Response<&mut ReqResBuffer>,
  ) -> Result<(), E> {
    let Self {
      allow_credentials,
      allow_headers,
      allow_methods,
      allow_origin,
      expose_headers,
      max_age,
    } = self;
    if *allow_credentials {
      res.rrd.headers_mut().push_from_iter(Header::from_name_and_value(
        KnownHeaderName::AccessControlAllowCredentials.into(),
        ["true".as_bytes()],
      ))?;
    }
    if !allow_headers.is_empty() {
      res.rrd.headers_mut().push_from_iter(Header::from_name_and_value(
        KnownHeaderName::AccessControlAllowHeaders.into(),
        allow_headers.iter().map(|el| el.as_bytes()),
      ))?;
    }
    if allow_methods.0 {
      res.rrd.headers_mut().push_from_iter(Header::from_name_and_value(
        KnownHeaderName::AccessControlAllowMethods.into(),
        ["*".as_bytes()],
      ))?;
    } else if !allow_methods.1.is_empty() {
      res.rrd.headers_mut().push_from_iter(Header::from_name_and_value(
        KnownHeaderName::AccessControlAllowMethods.into(),
        allow_methods.1.iter().map(|el| el.strings().custom[0].as_bytes()),
      ))?;
    } else {
    }
    if let Some(elem) = allow_origin {
      res.rrd.headers_mut().push_from_iter(Header::from_name_and_value(
        KnownHeaderName::AccessControlAllowOrigin.into(),
        [elem.as_bytes()],
      ))?;
    }
    if !expose_headers.is_empty() {
      res.rrd.headers_mut().push_from_iter(Header::from_name_and_value(
        KnownHeaderName::AccessControlAllowHeaders.into(),
        expose_headers.iter().map(|el| el.as_bytes()),
      ))?;
    }
    if let Some(elem) = max_age {
      res.rrd.headers_mut().push_from_fmt(Header::from_name_and_value(
        KnownHeaderName::AccessControlMaxAge.into(),
        format_args!("{elem}"),
      ))?;
    }
    Ok(())
  }
}

impl Default for CorsMiddleware {
  #[inline]
  fn default() -> Self {
    Self::new()
  }
}
