//! Convenient subset of HTTP parameters. Intended to be only used by HTTP endpoints.

use crate::{
  client_api_framework::network::transport::TransportParams,
  http::{Method, Mime, StatusCode},
  misc::UriString,
};
use alloc::{string::String, vec::Vec};
use core::fmt::{Arguments, Write};

#[derive(Debug)]
#[doc = generic_trans_params_doc!()]
pub struct HttpParams(HttpReqParams, HttpResParams);

impl HttpParams {
  /// For example, from `http://localhost`.
  #[inline]
  pub fn from_uri(url: &str) -> Self {
    Self(
      HttpReqParams {
        headers: HttpHeaders::default(),
        method: Method::Get,
        mime: None,
        uri: UriString::new(url.into()),
        user_agent: None,
      },
      HttpResParams { headers: HttpHeaders::default(), status_code: StatusCode::Forbidden },
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
  pub headers: HttpHeaders,
  /// Http method.
  pub method: Method,
  /// MIME type.
  pub mime: Option<Mime>,
  /// URL.
  pub uri: UriString,
  /// User agent.
  pub user_agent: Option<HttpUserAgent>,
}

#[doc = generic_trans_res_params_doc!("HTTP")]
#[derive(Debug)]
pub struct HttpResParams {
  /// Http headers.
  pub headers: HttpHeaders,
  /// Status code.
  pub status_code: StatusCode,
}

/// List of pairs sent and received on every request.
#[derive(Debug, Default)]
pub struct HttpHeaders {
  buffer: String,
  indcs: Vec<(usize, usize)>,
}

impl HttpHeaders {
  /// Clears the internal buffer "erasing" all previously inserted elements.
  #[inline]
  pub fn clear(&mut self) {
    self.buffer.clear();
    self.indcs.clear();
  }

  /// Retrieves all stored elements.
  #[inline]
  pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
    self.indcs.iter().scan(0, |idx_tracker, &(key_idx, value_idx)| {
      let key_str = self.buffer.get(*idx_tracker..key_idx)?;
      let value_str = self.buffer.get(key_idx..value_idx)?;
      *idx_tracker = value_idx;
      Some((key_str, value_str))
    })
  }

  /// Pushes a new pair of `key` and `value` at the end of the internal buffer.
  #[inline]
  pub fn push_str(&mut self, key: &str, value: &str) -> crate::Result<()> {
    self.push_fmt(format_args!("{key}"), format_args!("{value}"))?;
    Ok(())
  }

  /// Similar to [`Self::push_str`] but expects an `Arguments` instead.
  #[inline]
  pub fn push_fmt(&mut self, key: Arguments<'_>, value: Arguments<'_>) -> crate::Result<()> {
    let curr_len = self.buffer.len();

    let before_key_len = self.buffer.len();
    self.buffer.write_fmt(key)?;
    let key_idx = curr_len.wrapping_add(self.buffer.len().wrapping_sub(before_key_len));

    let before_value_len = self.buffer.len();
    self.buffer.write_fmt(value)?;
    let value_idx = key_idx.wrapping_add(self.buffer.len().wrapping_sub(before_value_len));

    self.indcs.push((key_idx, value_idx));
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use crate::client_api_framework::network::HttpHeaders;
  use alloc::{vec, vec::Vec};

  #[test]
  fn headers_has_correct_values() {
    let mut headers = HttpHeaders::default();
    headers.push_str("1", "2").unwrap();
    assert_eq!(headers.iter().collect::<Vec<_>>(), vec![("1", "2")]);
    headers.push_str("3", "4").unwrap();
    assert_eq!(headers.iter().collect::<Vec<_>>(), vec![("1", "2"), ("3", "4")]);
    headers.clear();
    assert_eq!(headers.iter().collect::<Vec<_>>(), vec![]);
  }
}
