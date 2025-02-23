use crate::{
  data_transformation::{
    Id,
    format::{JsonRpcRequest, VerbatimRequest},
  },
  misc::Vector,
};

/// # Packages Auxiliary
///
/// Responsible for assisting the creation and management of packages and their requests.
///
/// # Types
///
/// * `API`: Application Programming Interface
/// * `DRSR`: DeserializeR/SerializeR
/// * `TP`: Transport Parameters
#[derive(Debug)]
pub struct PkgsAux<A, DRSR, TP> {
  /// API instance.
  pub api: A,
  /// Used by practically all transports to serialize or receive data in any desired operation.
  ///
  /// Some transports require a pre-filled buffer so it is important to not modify indiscriminately.
  pub byte_buffer: Vector<u8>,
  /// Deserializer/Serializer instance
  pub drsr: DRSR,
  /// External request and response parameters.
  pub tp: TP,
  pub(crate) built_requests: Id,
}

impl<A, DRSR, TP> PkgsAux<A, DRSR, TP> {
  /// Creates an instance with the minimum amount of mandatory parameters.
  #[inline]
  pub fn from_minimum(api: A, drsr: DRSR, tp: TP) -> Self {
    Self { api, byte_buffer: Vector::new(), drsr, tp, built_requests: 0 }
  }

  /// New instance
  #[inline]
  pub fn new(api: A, byte_buffer: Vector<u8>, drsr: DRSR, tp: TP) -> Self {
    Self { api, byte_buffer, drsr, tp, built_requests: 0 }
  }

  /// The number of constructed requests that is not necessarily equal the number of sent requests.
  ///
  /// Wraps when a hard-to-happen overflow occurs
  #[inline]
  pub fn built_requests(&self) -> Id {
    self.built_requests
  }

  /// Constructs [JsonRpcRequest] and also increases the number of requests.
  #[inline]
  pub fn json_rpc_request<P>(&mut self, method: &'static str, params: P) -> JsonRpcRequest<P> {
    self.increase_requests_num();
    JsonRpcRequest { id: self.built_requests, method, params }
  }

  /// Constructs [VerbatimRequest] and also increases the number of requests.
  #[inline]
  pub fn verbatim_request<D>(&mut self, data: D) -> VerbatimRequest<D> {
    self.increase_requests_num();
    VerbatimRequest { data }
  }

  fn increase_requests_num(&mut self) {
    self.built_requests = self.built_requests.wrapping_add(1);
  }
}
