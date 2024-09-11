//! Convenient subset of HTTP parameters. Intended to be only used by HTTP endpoints.

use crate::{
  client_api_framework::network::transport::TransportParams,
  http::{Headers, Method, Mime, StatusCode},
  misc::UriString,
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
        user_agent: None,
      },
      HttpResParams { headers: Headers::new(), status_code: StatusCode::Forbidden },
    )
  }
}

impl TransportParams for HttpParams {
  type ExternalRequestParams = HttpReqParams;
  type ExternalResponseParams = HttpResParams;

  #[inline]
  fn ext_req_params(&self) -> &Self::ExternalRequestParams {
    &self.0
  }

  #[inline]
  fn ext_req_params_mut(&mut self) -> &mut Self::ExternalRequestParams {
    &mut self.0
  }

  #[inline]
  fn ext_res_params(&self) -> &Self::ExternalResponseParams {
    &self.1
  }

  #[inline]
  fn ext_res_params_mut(&mut self) -> &mut Self::ExternalResponseParams {
    &mut self.1
  }

  #[inline]
  fn reset(&mut self) {
    self.0.headers.clear();
    self.0.method = Method::Get;
    self.0.mime = None;
    self.0.uri.truncate_with_initial_len();
    self.0.user_agent = None;
    self.1.headers.clear();
    self.1.status_code = StatusCode::Forbidden;
  }
}

/// Characteristic string that lets servers and network peers identify a client.
#[derive(Clone, Copy, Debug)]
pub enum HttpUserAgent {
  /// Generic Mozilla
  Mozilla,
}

impl HttpUserAgent {
  pub(crate) fn _as_str(self) -> &'static str {
    match self {
      Self::Mozilla => "Mozilla",
    }
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
  /// User agent.
  pub user_agent: Option<HttpUserAgent>,
}

#[doc = generic_trans_res_params_doc!("HTTP")]
#[derive(Debug)]
pub struct HttpResParams {
  /// Http headers.
  pub headers: Headers,
  /// Status code.
  pub status_code: StatusCode,
}
