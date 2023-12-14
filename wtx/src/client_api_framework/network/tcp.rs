use crate::client_api_framework::{misc::UrlString, network::transport::TransportParams};

#[derive(Debug)]
#[doc = generic_trans_params_doc!()]
pub struct TcpParams(TcpReqParams, TcpResParams);

impl TcpParams {
  /// For example, from `127.0.0.1:8090`.
  #[inline]
  pub fn from_url(url: &str) -> crate::Result<Self> {
    Ok(Self(TcpReqParams { url: UrlString::from_url(url.into())? }, TcpResParams))
  }
}

impl TransportParams for TcpParams {
  type ExternalRequestParams = TcpReqParams;
  type ExternalResponseParams = TcpResParams;

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
    self.0.url.retain_with_initial_len();
  }
}

#[derive(Debug)]
#[doc = generic_trans_req_params_doc!("TCP")]
pub struct TcpReqParams {
  /// Used every time a send-like function is called.
  pub url: UrlString,
}

#[derive(Debug)]
#[doc = generic_trans_res_params_doc!("TCP")]
pub struct TcpResParams;
