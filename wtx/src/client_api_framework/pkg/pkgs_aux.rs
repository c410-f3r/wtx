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
  /// The second element is a back-up of the first element. Such a structure is used
  /// by transports.
  ///
  /// See [Self::log_body]
  pub log_body: (bool, bool),
  /// In cases where the data is already available in the `bytes_buffer` field.
  pub send_bytes_buffer: bool,
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
      log_body: (false, false),
      send_bytes_buffer: false,
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
    log_body: bool,
    send_bytes_buffer: bool,
    tp: TP,
  ) -> Self {
    Self {
      api,
      built_requests,
      bytes_buffer,
      drsr,
      log_body: (log_body, false),
      send_bytes_buffer,
      tp,
    }
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

  /// Temporally logs sending or receiving bytes.
  #[inline]
  pub const fn log_body(&mut self) {
    self.log_body.0 = true;
  }

  /// If the current request/response should be logged
  #[inline]
  pub const fn should_log_body(&self) -> bool {
    self.log_body.0
  }

  /// Constructs [VerbatimEncoder] and also increases the number of requests.
  #[inline]
  pub const fn verbatim_request<D>(&mut self, data: D) -> VerbatimEncoder<D> {
    self.increase_requests_num();
    VerbatimEncoder { data }
  }
}
