use crate::{
  collection::Vector,
  http::{
    Header, Headers, HttpError, KnownHeaderName, Method, ReqResBuffer, Request, Response,
    StatusCode,
    server_framework::{ConnAux, Middleware, ServerFrameworkError},
  },
  misc::{Intersperse, str_split1},
};
use alloc::string::String;
use core::{ops::ControlFlow, str};
use hashbrown::HashSet;

type AllowHeaders = (bool, Vector<String>);
type AllowMethods = (bool, Vector<Method>);
type AllowOrigins = (bool, Vector<String>);
type ExposeHeaders = (bool, Vector<String>);

/// Used internally to manage the origins of CORS responses.
#[derive(Debug)]
pub enum OriginResponse {
  /// An internal origin is passed to the response
  AllowedFromInternalList(usize),
  /// No origin in response
  None,
  /// Response origin is "*"
  Wildcard,
}

/// Cross-origin resource sharing
#[derive(Debug)]
pub struct CorsMiddleware {
  allow_credentials: bool,
  // Many local options, many request/response options.
  allow_headers: AllowHeaders,
  // Many local options, many request/response options.
  allow_methods: AllowMethods,
  // Many local options, single request/response option.
  allow_origins: AllowOrigins,
  // Many local options, many request/response options.
  expose_headers: ExposeHeaders,
  max_age: Option<u32>,
}

impl ConnAux for CorsMiddleware {
  type Init = Self;

  #[inline]
  fn conn_aux(init: Self::Init) -> crate::Result<Self> {
    Ok(init)
  }
}

impl CorsMiddleware {
  /// New empty instance
  #[inline]
  pub const fn new() -> Self {
    Self {
      allow_credentials: false,
      allow_headers: (false, Vector::new()),
      allow_methods: (false, Vector::new()),
      allow_origins: (false, Vector::new()),
      expose_headers: (false, Vector::new()),
      max_age: None,
    }
  }

  /// * Credentials are allowed
  /// * All request headers allowed (wildcard).
  /// * All methods are allowed.
  /// * All origins are allowed (wildcard).
  /// * All headers are exposed (wildcard).
  /// * No caching
  #[inline]
  pub fn permissive() -> crate::Result<Self> {
    Ok(Self {
      allow_credentials: true,
      allow_headers: (true, Vector::new()),
      allow_methods: (false, Vector::from_iterator(Method::ALL.into_iter())?),
      allow_origins: (true, Vector::new()),
      expose_headers: (true, Vector::new()),
      max_age: None,
    })
  }

  /// <https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Access-Control-Allow-Credentials>
  #[inline]
  #[must_use]
  pub const fn allow_credentials(mut self) -> Self {
    self.allow_credentials = true;
    self
  }

  /// <https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Access-Control-Allow-Headers>
  ///
  /// Wildcard is only allowed in requests without credentials.
  #[inline]
  #[must_use]
  pub fn allow_headers(
    mut self,
    is_wildcard: bool,
    specifics: impl IntoIterator<Item = String>,
  ) -> Self {
    if is_wildcard {
      self.allow_headers.0 = true;
    } else {
      self.allow_headers.0 = false;
      self.allow_headers.1.clear();
      let iter = specifics.into_iter();
      let _rslt = self.allow_headers.1.extend_from_iter(iter);
    }
    self
  }

  /// <https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Access-Control-Allow-Methods>
  ///
  /// Wildcard is only allowed in requests without credentials.
  #[inline]
  #[must_use]
  pub fn allow_methods(
    mut self,
    is_wildcard: bool,
    specifics: impl IntoIterator<Item = Method>,
  ) -> Self {
    if is_wildcard {
      self.allow_methods.0 = true;
    } else {
      self.allow_methods.0 = false;
      self.allow_methods.1.clear();
      let iter = specifics.into_iter();
      let _rslt = self.allow_methods.1.extend_from_iter(iter);
    }
    self
  }

  /// <https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Access-Control-Allow-Origin>
  ///
  /// Wildcard is only allowed in requests without credentials.
  #[inline]
  #[must_use]
  pub fn allow_origins(
    mut self,
    is_wildcard: bool,
    specifics: impl IntoIterator<Item = String>,
  ) -> Self {
    if is_wildcard {
      self.allow_origins.0 = true;
    } else {
      self.allow_origins.0 = false;
      self.allow_origins.1.clear();
      let iter = specifics.into_iter();
      let _rslt = self.allow_origins.1.extend_from_iter(iter);
    }
    self
  }

  /// <https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Access-Control-Expose-Headers>
  ///
  /// Wildcard is only allowed in requests without credentials.
  #[inline]
  #[must_use]
  pub fn expose_headers(
    mut self,
    is_wildcard: bool,
    specifics: impl IntoIterator<Item = String>,
  ) -> Self {
    if is_wildcard {
      self.expose_headers.0 = true;
    } else {
      self.expose_headers.0 = false;
      self.expose_headers.1.clear();
      let iter = specifics.into_iter();
      let _rslt = self.expose_headers.1.extend_from_iter(iter);
    }
    self
  }

  /// <https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Access-Control-Max-Age>
  #[inline]
  #[must_use]
  pub const fn max_age(mut self, elem: u32) -> Self {
    self.max_age = Some(elem);
    self
  }

  fn allowed_origin<'this>(&'this self, origin: &str) -> Option<(&'this str, usize)> {
    self
      .allow_origins
      .1
      .iter()
      .enumerate()
      .find_map(|(idx, el)| (el == origin).then_some((el.as_str(), idx)))
  }

  fn apply_allow_credentials(allow_credentials: bool, headers: &mut Headers) -> crate::Result<()> {
    if allow_credentials {
      headers.push_from_iter(Header::from_name_and_value(
        KnownHeaderName::AccessControlAllowCredentials.into(),
        ["true"],
      ))?;
    }
    Ok(())
  }

  fn apply_allow_headers(allow_headers: &str, headers: &mut Headers) -> crate::Result<()> {
    if !allow_headers.is_empty() {
      headers.push_from_iter(Header::from_name_and_value(
        KnownHeaderName::AccessControlAllowHeaders.into(),
        [allow_headers],
      ))?;
    }
    Ok(())
  }

  fn apply_allow_methods(
    (is_wildcard, specifics): &AllowMethods,
    headers: &mut Headers,
  ) -> crate::Result<()> {
    if *is_wildcard {
      headers.push_from_iter(Header::from_name_and_value(
        KnownHeaderName::AccessControlAllowMethods.into(),
        ["*"],
      ))?;
      return Ok(());
    }
    if !specifics.is_empty() {
      headers.push_from_iter(Header::from_name_and_value(
        KnownHeaderName::AccessControlAllowMethods.into(),
        Intersperse::new(
          specifics.iter().map(|el| {
            let [name, _] = el.strings().custom;
            name
          }),
          || ",",
        ),
      ))?;
      return Ok(());
    }
    Ok(())
  }

  fn apply_allow_origin(origin: &str, headers: &mut Headers) -> crate::Result<()> {
    headers.push_from_iter(Header::from_name_and_value(
      KnownHeaderName::AccessControlAllowOrigin.into(),
      [origin],
    ))?;
    Ok(())
  }

  fn apply_expose_headers(
    (is_wildcard, specifics): &ExposeHeaders,
    headers: &mut Headers,
  ) -> crate::Result<()> {
    if *is_wildcard {
      headers.push_from_iter(Header::from_name_and_value(
        KnownHeaderName::AccessControlExposeHeaders.into(),
        ["*"],
      ))?;
      return Ok(());
    }
    if !specifics.is_empty() {
      headers.push_from_iter(Header::from_name_and_value(
        KnownHeaderName::AccessControlExposeHeaders.into(),
        Intersperse::new(specifics.iter().map(|el| el.as_str()), || ","),
      ))?;
      return Ok(());
    }
    Ok(())
  }

  fn apply_max_age(max_age: Option<u32>, headers: &mut Headers) -> crate::Result<()> {
    if let Some(elem) = max_age {
      headers.push_from_fmt(Header::from_name_and_value(
        KnownHeaderName::AccessControlMaxAge.into(),
        format_args!("{elem}"),
      ))?;
    }
    Ok(())
  }

  async fn apply_normal_response(
    &self,
    origin_response: &OriginResponse,
    headers: &mut Headers,
  ) -> crate::Result<()> {
    let Self {
      allow_credentials,
      allow_headers: _,
      allow_methods: _,
      allow_origins: _,
      expose_headers,
      max_age: _,
    } = self;
    Self::apply_allow_credentials(*allow_credentials, headers)?;
    match origin_response {
      OriginResponse::AllowedFromInternalList(idx) => {
        Self::apply_allow_origin(
          self.allow_origins.1.get(*idx).map(|el| el.as_str()).unwrap_or_default(),
          headers,
        )?;
      }
      OriginResponse::None => {}
      OriginResponse::Wildcard => {
        Self::apply_allow_origin("*", headers)?;
      }
    }
    Self::apply_expose_headers(expose_headers, headers)?;
    Ok(())
  }

  async fn apply_preflight_response(
    &self,
    evaluated_allow_headers: &str,
    evaluated_allow_origin: &str,
    headers: &mut Headers,
  ) -> crate::Result<()> {
    let Self {
      allow_credentials,
      allow_headers: _,
      allow_methods,
      allow_origins: _,
      expose_headers: _,
      max_age,
    } = self;
    Self::apply_allow_credentials(*allow_credentials, headers)?;
    Self::apply_allow_headers(evaluated_allow_headers, headers)?;
    Self::apply_allow_methods(allow_methods, headers)?;
    Self::apply_allow_origin(evaluated_allow_origin, headers)?;
    Self::apply_max_age(*max_age, headers)?;
    Ok(())
  }

  fn extract_origin<'any>(
    opt: Option<Header<'any, &'any str>>,
  ) -> crate::Result<Header<'any, &'any str>> {
    Ok(opt.ok_or(HttpError::MissingHeader(KnownHeaderName::Origin))?)
  }

  fn manage_preflight_headers(
    &self,
    acrh: Header<'_, &str>,
    body: &mut Vector<u8>,
  ) -> crate::Result<()> {
    if self.allow_headers.0 {
      body.extend_from_copyable_slice(acrh.value.as_bytes())?;
      return Ok(());
    }
    let mut uniques = HashSet::new();
    for sub_header in str_split1(acrh.value, b',') {
      let _ = uniques.insert(sub_header.trim_ascii());
    }
    let mut matched_headers: usize = 0;
    for allow_header in self.allow_headers.1.iter() {
      if uniques.contains(allow_header.as_str()) {
        matched_headers = matched_headers.wrapping_add(1);
      }
    }
    if matched_headers != uniques.len() {
      return Err(crate::Error::from(ServerFrameworkError::ForbiddenCorsHeader));
    }
    let mut iter = uniques.iter();
    if let Some(elem) = iter.next() {
      body.extend_from_copyable_slice(elem.as_bytes())?;
    }
    for elem in iter {
      let _ = body.extend_from_copyable_slices([",".as_bytes(), elem.as_bytes()])?;
    }
    Ok(())
  }

  fn manage_preflight_methods(&self, acrm: Header<'_, &str>) -> crate::Result<()> {
    if self.allow_methods.0 {
      return Ok(());
    }
    if !self.allow_methods.1.iter().any(|method| {
      let strings = method.strings();
      let [a, b] = strings.custom;
      strings.ident == acrm.value || a == acrm.value || b == acrm.value
    }) {
      return Err(crate::Error::from(ServerFrameworkError::ForbiddenCorsMethod));
    }
    Ok(())
  }

  fn manage_preflight_origin(
    &self,
    body: &mut Vector<u8>,
    origin: Header<'_, &str>,
  ) -> crate::Result<()> {
    let actual_origin = if self.allow_origins.0 {
      origin.value
    } else if let Some(allowed_origin) = self.allowed_origin(origin.value) {
      allowed_origin.0
    } else {
      return Err(crate::Error::from(ServerFrameworkError::ForbiddenCorsOrigin));
    };
    body.extend_from_copyable_slice(actual_origin.as_bytes())?;
    Ok(())
  }
}

impl<CA, E, SA> Middleware<CA, E, SA> for CorsMiddleware
where
  E: From<crate::Error>,
{
  type Aux = OriginResponse;

  #[inline]
  fn aux(&self) -> Self::Aux {
    OriginResponse::None
  }

  #[inline]
  async fn req(
    &self,
    _: &mut CA,
    mw_aux: &mut Self::Aux,
    req: &mut Request<ReqResBuffer>,
    _: &mut SA,
  ) -> Result<ControlFlow<StatusCode, ()>, E> {
    let origin_opt = if req.method == Method::Options {
      let [acrh_opt, acrm_opt, origin_opt] = req.rrd.headers.get_by_names([
        KnownHeaderName::AccessControlRequestHeaders.into(),
        KnownHeaderName::AccessControlRequestMethod.into(),
        KnownHeaderName::Origin.into(),
      ]);
      if let (Some(acrh), Some(acrm)) = (acrh_opt, acrm_opt) {
        req.rrd.body.clear();
        self.manage_preflight_headers(acrh, &mut req.rrd.body)?;
        self.manage_preflight_methods(acrm)?;
        let idx = req.rrd.body.len();
        self.manage_preflight_origin(&mut req.rrd.body, Self::extract_origin(origin_opt)?)?;
        let (headers_bytes, origin_bytes) = req.rrd.body.split_at_checked(idx).unwrap_or_default();
        req.rrd.headers.clear();
        self
          .apply_preflight_response(
            // SAFETY: every single element of `req.rrd.body` was previously inserted with UTF-8
            unsafe { str::from_utf8_unchecked(headers_bytes) },
            // SAFETY: every single element of `req.rrd.body` was previously inserted with UTF-8
            unsafe { str::from_utf8_unchecked(origin_bytes) },
            &mut req.rrd.headers,
          )
          .await?;
        req.rrd.body.clear();
        return Ok(ControlFlow::Break(StatusCode::Ok));
      } else {
        origin_opt
      }
    } else {
      req.rrd.headers.get_by_name(KnownHeaderName::Origin.into())
    };
    if self.allow_origins.0 {
      *mw_aux = OriginResponse::Wildcard
    } else if let Some(origin) = self.allowed_origin(Self::extract_origin(origin_opt)?.value) {
      *mw_aux = OriginResponse::AllowedFromInternalList(origin.1)
    }
    Ok(ControlFlow::Continue(()))
  }

  #[inline]
  async fn res(
    &self,
    _: &mut CA,
    mw_aux: &mut Self::Aux,
    res: Response<&mut ReqResBuffer>,
    _: &mut SA,
  ) -> Result<ControlFlow<StatusCode, ()>, E> {
    self.apply_normal_response(mw_aux, &mut res.rrd.headers).await?;
    Ok(ControlFlow::Continue(()))
  }
}

impl Default for CorsMiddleware {
  #[inline]
  fn default() -> Self {
    Self::new()
  }
}
