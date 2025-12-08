//! Convenient subset of HTTP parameters. Intended to be only used by HTTP endpoints.

use crate::{
  client_api_framework::network::transport::TransportParams,
  collection::Vector,
  http::{Headers, Method, Mime, ReqResBuffer, StatusCode},
  misc::{Lease, LeaseMut, UriString},
};
use alloc::string::String;

#[derive(Debug)]
#[doc = generic_trans_params_doc!()]
pub struct HttpParams(HttpReqParams, HttpResParams);

impl HttpParams {
  /// For example, from `http://localhost`.
  #[inline]
  pub fn from_uri(uri: String) -> Self {
    Self(
      HttpReqParams {
        host: true,
        method: Method::Get,
        mime: None,
        rrb: ReqResBuffer {
          body: Vector::new(),
          headers: Headers::new(),
          uri: UriString::new(uri),
        },
        user_agent_custom: None,
        user_agent_default: true,
      },
      HttpResParams { status_code: StatusCode::Forbidden },
    )
  }
}

impl Lease<HttpParams> for HttpParams {
  #[inline]
  fn lease(&self) -> &HttpParams {
    self
  }
}

impl LeaseMut<HttpParams> for HttpParams {
  #[inline]
  fn lease_mut(&mut self) -> &mut HttpParams {
    self
  }
}

impl TransportParams for HttpParams {
  type ExternalRequestParams = HttpReqParams;
  type ExternalResponseParams = HttpResParams;

  #[inline]
  fn ext_params(&self) -> (&Self::ExternalRequestParams, &Self::ExternalResponseParams) {
    (&self.0, &self.1)
  }

  #[inline]
  fn ext_params_mut(
    &mut self,
  ) -> (&mut Self::ExternalRequestParams, &mut Self::ExternalResponseParams) {
    (&mut self.0, &mut self.1)
  }

  #[inline]
  fn reset(&mut self) {
    self.0.reset();
    self.1.reset();
  }
}

#[derive(Debug)]
#[doc = generic_trans_req_params_doc!("HTTP")]
pub struct HttpReqParams {
  /// If the host should be included in the headers.
  ///
  /// Defaults to `true`.
  pub host: bool,
  /// See [`Method`].
  pub method: Method,
  /// See [`Mime`].
  pub mime: Option<Mime>,
  /// See [`ReqResBuffer`].
  pub rrb: ReqResBuffer,
  /// Custom user agent that will be included in the headers
  ///
  /// If `user_agent_default` is `true`, then this field becomes a NO-OP.
  pub user_agent_custom: Option<&'static str>,
  /// System's user agent that will be included in the headers. For example, "wtx/0.0.1".
  ///
  /// Defaults to `true`.
  pub user_agent_default: bool,
}

impl HttpReqParams {
  /// Sets the inner parameters with their default values.
  #[inline]
  pub fn reset(&mut self) {
    let Self { host, method, mime, rrb, user_agent_custom, user_agent_default } = self;
    *host = true;
    *method = Method::Get;
    *mime = None;
    *user_agent_custom = None;
    *user_agent_default = true;
    let ReqResBuffer { body, headers, uri } = rrb;
    body.clear();
    headers.clear();
    uri.truncate_with_initial_len();
  }
}

#[doc = generic_trans_res_params_doc!("HTTP")]
#[derive(Debug)]
pub struct HttpResParams {
  /// Status code.
  pub status_code: StatusCode,
}

impl HttpResParams {
  /// Sets the inner parameters with their default values.
  #[inline]
  pub const fn reset(&mut self) {
    self.status_code = StatusCode::InternalServerError;
  }
}
