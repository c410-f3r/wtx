// https://fetch.spec.whatwg.org/#http-cors-protocol

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

type AllowHeaders = (bool, Vector<String>);
type AllowMethods = (bool, Vector<Method>);
type AllowOrigins = (bool, Vector<String>);
type ExposeHeaders = (bool, Vector<String>);

/// Used internally to manage the origins of CORS responses.
#[derive(Debug)]
pub enum OriginResponse {
  /// An internal allowed origin is passed to the response
  Match(usize),
  /// Received origin is not in the internal list of allowed origins
  Mismatch,
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

  /// * All request headers allowed (wildcard).
  /// * All methods are allowed (wildcard).
  /// * All origins are allowed (wildcard).
  /// * All headers are exposed (wildcard).
  /// * No caching
  #[inline]
  pub const fn permissive() -> Self {
    Self {
      allow_credentials: false,
      allow_headers: (true, Vector::new()),
      allow_methods: (true, Vector::new()),
      allow_origins: (true, Vector::new()),
      expose_headers: (true, Vector::new()),
      max_age: None,
    }
  }

  /// <https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Access-Control-Allow-Credentials>
  #[inline]
  pub fn allow_credentials(mut self, allow_credentials: bool) -> crate::Result<Self> {
    self.allow_credentials = allow_credentials;
    self.manage_local_params()?;
    Ok(self)
  }

  /// <https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Access-Control-Allow-Headers>
  ///
  /// Wildcard is only allowed in requests without credentials.
  #[inline]
  pub fn allow_headers(
    mut self,
    is_wildcard: bool,
    specifics: impl IntoIterator<Item = String>,
  ) -> crate::Result<Self> {
    if is_wildcard {
      self.allow_headers.0 = true;
      self.allow_headers.1.clear();
    } else {
      self.allow_headers.0 = false;
      self.allow_headers.1.clear();
      let iter = specifics.into_iter();
      self.allow_headers.1.extend_from_iter(iter)?;
    }
    self.manage_local_params()?;
    Ok(self)
  }

  /// <https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Access-Control-Allow-Methods>
  ///
  /// Wildcard is only allowed in requests without credentials.
  #[inline]
  pub fn allow_methods(
    mut self,
    is_wildcard: bool,
    specifics: impl IntoIterator<Item = Method>,
  ) -> crate::Result<Self> {
    if is_wildcard {
      self.allow_methods.0 = true;
      self.allow_methods.1.clear();
    } else {
      self.allow_methods.0 = false;
      self.allow_methods.1.clear();
      let iter = specifics.into_iter();
      self.allow_methods.1.extend_from_iter(iter)?;
    }
    self.manage_local_params()?;
    Ok(self)
  }

  /// <https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Access-Control-Allow-Origin>
  ///
  /// Wildcard is only allowed in requests without credentials.
  #[inline]
  pub fn allow_origins(
    mut self,
    is_wildcard: bool,
    specifics: impl IntoIterator<Item = String>,
  ) -> crate::Result<Self> {
    if is_wildcard {
      self.allow_origins.0 = true;
      self.allow_origins.1.clear();
    } else {
      self.allow_origins.0 = false;
      self.allow_origins.1.clear();
      let iter = specifics.into_iter();
      self.allow_origins.1.extend_from_iter(iter)?;
    }
    self.manage_local_params()?;
    Ok(self)
  }

  /// <https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Access-Control-Expose-Headers>
  ///
  /// Wildcard is only allowed in requests without credentials.
  #[inline]
  pub fn expose_headers(
    mut self,
    is_wildcard: bool,
    specifics: impl IntoIterator<Item = String>,
  ) -> crate::Result<Self> {
    if is_wildcard {
      self.expose_headers.0 = true;
      self.expose_headers.1.clear();
    } else {
      self.expose_headers.0 = false;
      self.expose_headers.1.clear();
      let iter = specifics.into_iter();
      self.expose_headers.1.extend_from_iter(iter)?;
    }
    self.manage_local_params()?;
    Ok(self)
  }

  /// <https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Access-Control-Max-Age>
  #[inline]
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

  fn apply_allow_origin(
    apply_vary: bool,
    origin: &str,
    headers: &mut Headers,
  ) -> crate::Result<()> {
    headers.push_from_iter(Header::from_name_and_value(
      KnownHeaderName::AccessControlAllowOrigin.into(),
      [origin],
    ))?;
    if apply_vary {
      Self::apply_vary(headers)?;
    }
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

  fn apply_normal_response(
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
    match origin_response {
      OriginResponse::Match(idx) => {
        Self::apply_allow_credentials(*allow_credentials, headers)?;
        Self::apply_allow_origin(
          true,
          self.allow_origins.1.get(*idx).map(|el| el.as_str()).unwrap_or_default(),
          headers,
        )?;
        Self::apply_expose_headers(expose_headers, headers)?;
      }
      OriginResponse::Mismatch => {
        Self::apply_vary(headers)?;
      }
      OriginResponse::None => {}
      OriginResponse::Wildcard => {
        Self::apply_allow_credentials(*allow_credentials, headers)?;
        Self::apply_allow_origin(false, "*", headers)?;
        Self::apply_expose_headers(expose_headers, headers)?;
      }
    }
    Ok(())
  }

  fn apply_preflight_response(
    &self,
    apply_vary: bool,
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
    Self::apply_allow_origin(apply_vary, evaluated_allow_origin, headers)?;
    Self::apply_max_age(*max_age, headers)?;
    Ok(())
  }

  fn apply_vary(headers: &mut Headers) -> crate::Result<()> {
    headers
      .push_from_iter(Header::from_name_and_value(KnownHeaderName::Vary.into(), ["Origin"]))?;
    Ok(())
  }

  fn extract_origin<'any>(
    opt: Option<Header<'any, &'any str>>,
  ) -> crate::Result<Header<'any, &'any str>> {
    Ok(opt.ok_or(HttpError::MissingHeader(KnownHeaderName::Origin))?)
  }

  fn manage_local_params(&self) -> crate::Result<()> {
    let Self {
      allow_credentials, allow_headers, allow_methods, allow_origins, expose_headers, ..
    } = self;
    let has_wildcard = allow_headers.0 || allow_methods.0 || allow_origins.0 || expose_headers.0;
    if has_wildcard && *allow_credentials {
      return Err(crate::Error::from(ServerFrameworkError::ForbiddenLocalCorsParameters));
    }
    Ok(())
  }

  // Writes unique sub headers into `body`.
  fn manage_preflight_headers(
    &self,
    acrh: Header<'_, &str>,
    body: &mut Vector<u8>,
  ) -> crate::Result<()> {
    if self.allow_headers.0 {
      body.extend_from_copyable_slice(acrh.value.as_bytes())?;
      return Ok(());
    }
    let mut iter = str_split1(acrh.value, b',').map(str::trim);
    if let Some(elem) = iter.next() {
      self.push_preflight_headers(body, (&[], elem))?;
    }
    for elem in iter {
      self.push_preflight_headers(body, (b",", elem))?;
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
  ) -> crate::Result<bool> {
    let mut apply_vary = false;
    let actual_origin = if self.allow_origins.0 {
      "*"
    } else if let Some(allowed_origin) = self.allowed_origin(origin.value) {
      apply_vary = true;
      allowed_origin.0
    } else {
      return Err(crate::Error::from(ServerFrameworkError::ForbiddenCorsOrigin));
    };
    body.extend_from_copyable_slice(actual_origin.as_bytes())?;
    Ok(apply_vary)
  }

  fn push_preflight_headers(
    &self,
    body: &mut Vector<u8>,
    (prefix, elem): (&[u8], &str),
  ) -> crate::Result<()> {
    if elem.is_empty() {
      return Ok(());
    }
    if !self.allow_headers.1.iter().any(|el| el == elem) {
      return Err(crate::Error::from(ServerFrameworkError::ForbiddenCorsHeader));
    }
    let _ = body.extend_from_copyable_slices([prefix, elem.as_bytes()])?;
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

  // Requests without ORIGIN headers are ignored.
  #[inline]
  async fn req(
    &self,
    _: &mut CA,
    mw_aux: &mut Self::Aux,
    req: &mut Request<ReqResBuffer>,
    _: &mut SA,
  ) -> Result<ControlFlow<StatusCode, ()>, E> {
    let origin_header_opt = if req.method == Method::Options {
      let [acrh_opt, acrm_opt, origin_opt] = req.rrd.headers.get_by_names([
        KnownHeaderName::AccessControlRequestHeaders.into(),
        KnownHeaderName::AccessControlRequestMethod.into(),
        KnownHeaderName::Origin.into(),
      ]);
      if let Some(acrm) = acrm_opt {
        req.rrd.body.clear();
        if let Some(acrh) = acrh_opt {
          self.manage_preflight_headers(acrh, &mut req.rrd.body)?;
        }
        self.manage_preflight_methods(acrm)?;
        let idx = req.rrd.body.len();
        let origin = Self::extract_origin(origin_opt)?;
        let apply_vary = self.manage_preflight_origin(&mut req.rrd.body, origin)?;
        let (headers_bytes, origin_bytes) = req.rrd.body.split_at_checked(idx).unwrap_or_default();
        req.rrd.headers.clear();
        self.apply_preflight_response(
          apply_vary,
          // SAFETY: every single element of `req.rrd.body` was previously inserted with UTF-8
          unsafe { str::from_utf8_unchecked(headers_bytes) },
          // SAFETY: every single element of `req.rrd.body` was previously inserted with UTF-8
          unsafe { str::from_utf8_unchecked(origin_bytes) },
          &mut req.rrd.headers,
        )?;
        req.rrd.body.clear();
        return Ok(ControlFlow::Break(StatusCode::Ok));
      } else {
        origin_opt
      }
    } else {
      req.rrd.headers.get_by_name(KnownHeaderName::Origin.into())
    };
    if let Some(origin_header) = origin_header_opt {
      if self.allow_origins.0 {
        *mw_aux = OriginResponse::Wildcard
      } else if let Some(allowed_origin) = self.allowed_origin(origin_header.value) {
        *mw_aux = OriginResponse::Match(allowed_origin.1)
      } else {
        *mw_aux = OriginResponse::Mismatch
      }
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
    self.apply_normal_response(mw_aux, &mut res.rrd.headers)?;
    Ok(ControlFlow::Continue(()))
  }
}

impl Default for CorsMiddleware {
  #[inline]
  fn default() -> Self {
    Self::new()
  }
}
