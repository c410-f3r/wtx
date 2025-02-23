//! Convenient subset of HTTP parameters. Intended to be only used by HTTP endpoints.

use crate::{
  client_api_framework::network::transport::TransportParams,
  http::{Headers, Method, Mime, StatusCode},
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
        headers: Headers::new(),
        method: Method::Get,
        mime: None,
        uri: UriString::new(uri),
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
  /// Http headers.
  pub headers: Headers,
  /// Http method.
  pub method: Method,
  /// MIME type.
  pub mime: Option<Mime>,
  /// URI.
  pub uri: UriString,
}

impl HttpReqParams {
  /// Sets the inner parameters with their default values.
  #[inline]
  pub fn reset(&mut self) {
    self.headers.clear();
    self.method = Method::Get;
    self.mime = None;
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
  pub fn reset(&mut self) {
    self.status_code = StatusCode::InternalServerError;
  }
}
