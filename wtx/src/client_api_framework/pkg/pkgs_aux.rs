use crate::{
  collection::Vector,
  de::{
    Id,
    protocol::{JsonRpcEncoder, VerbatimEncoder},
  },
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
  /// The number of build requests
  pub built_requests: Id,
  /// Used by practically all transports to serialize or receive data in any desired operation.
  ///
  /// Some transports require a pre-filled buffer so it is important to not modify indiscriminately.
  pub bytes_buffer: Vector<u8>,
  /// Deserializer/Serializer instance
  pub drsr: DRSR,
  /// Useful in cases where the data is already encoded in the buffers.
  pub encode_data: bool,
  /// /// If the current request/response should be logged.
  pub log_data: bool,
  /// External request and response parameters.
  pub tp: TP,
}

impl<A, DRSR, TP> PkgsAux<A, DRSR, TP> {
  /// Creates an instance with the minimum amount of relevant parameters.
  #[inline]
  pub const fn from_minimum(api: A, drsr: DRSR, tp: TP) -> Self {
    Self {
      api,
      bytes_buffer: Vector::new(),
      drsr,
      encode_data: true,
      log_data: false,
      tp,
      built_requests: 0,
    }
  }

  /// New instance
  #[inline]
  pub const fn new(
    api: A,
    built_requests: u64,
    bytes_buffer: Vector<u8>,
    drsr: DRSR,
    encode_data: bool,
    log_data: bool,
    tp: TP,
  ) -> Self {
    Self { api, built_requests, bytes_buffer, drsr, encode_data, log_data, tp }
  }

  /// Should be used after a new request construction
  pub const fn increase_requests_num(&mut self) {
    self.built_requests = self.built_requests.wrapping_add(1);
  }

  /// Constructs [JsonRpcEncoder] and also increases the number of requests.
  #[inline]
  pub const fn json_rpc_request<P>(
    &mut self,
    method: &'static str,
    params: P,
  ) -> JsonRpcEncoder<P> {
    self.increase_requests_num();
    JsonRpcEncoder { id: self.built_requests, method, params }
  }

  /// Constructs [VerbatimEncoder] and also increases the number of requests.
  #[inline]
  pub const fn verbatim_request<D>(&mut self, data: D) -> VerbatimEncoder<D> {
    self.increase_requests_num();
    VerbatimEncoder { data }
  }
}
