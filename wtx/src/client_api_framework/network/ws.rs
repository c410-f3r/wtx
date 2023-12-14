use crate::client_api_framework::network::transport::TransportParams;

/// How the WebSocket request should be issued.
#[derive(Clone, Copy, Debug)]
pub enum WsReqParamsTy {
  /// As opaque bytes.
  Bytes,
  /// As a string.
  String,
}

#[derive(Debug)]
#[doc = generic_trans_params_doc!()]
pub struct WsParams(WsReqParams, WsResParams);

impl TransportParams for WsParams {
  type ExternalRequestParams = WsReqParams;
  type ExternalResponseParams = WsResParams;

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
    self.0.ty = WsReqParamsTy::String;
  }
}

impl Default for WsParams {
  #[inline]
  fn default() -> Self {
    Self(WsReqParams { ty: WsReqParamsTy::String }, WsResParams)
  }
}

#[derive(Debug)]
#[doc = generic_trans_req_params_doc!("WebSocket")]
pub struct WsReqParams {
  /// Type
  pub ty: WsReqParamsTy,
}

#[derive(Debug)]
#[doc = generic_trans_res_params_doc!("WebSocket")]
pub struct WsResParams;
