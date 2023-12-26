use crate::{client_api_framework::network::transport::TransportParams, misc::UriString};

#[derive(Debug)]
#[doc = generic_trans_params_doc!()]
pub struct UdpParams(UdpReqParams, UdpResParams);

impl UdpParams {
  /// For example, from `127.0.0.1:8090`.
  #[inline]
  pub fn from_uri(url: &str) -> Self {
    Self(UdpReqParams { url: UriString::new(url.into()) }, UdpResParams)
  }
}

impl TransportParams for UdpParams {
  type ExternalRequestParams = UdpReqParams;
  type ExternalResponseParams = UdpResParams;

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
#[doc = generic_trans_req_params_doc!("UDP")]
pub struct UdpReqParams {
  /// Used every time a send-like function is called.
  pub url: UriString,
}

#[derive(Debug)]
#[doc = generic_trans_res_params_doc!("UDP")]
pub struct UdpResParams;
