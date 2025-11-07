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
  pub(crate) built_requests: Id,
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
    bytes_buffer: Vector<u8>,
    drsr: DRSR,
    log_body: bool,
    send_bytes_buffer: bool,
    tp: TP,
  ) -> Self {
    Self {
      api,
      bytes_buffer,
      drsr,
      log_body: (log_body, false),
      send_bytes_buffer,
      tp,
      built_requests: 0,
    }
  }

  /// The number of constructed requests that is not necessarily equal the number of sent requests.
  ///
  /// Wraps when a hard-to-happen overflow occurs
  #[inline]
  pub const fn built_requests(&self) -> Id {
    self.built_requests
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

  /// Logs sending or receiving bytes.
  #[inline]
  pub const fn log_body(&mut self) {
    self.log_body.0 = true;
  }

  /// Whether to show the contents of a request/response.
  #[inline]
  pub const fn set_log_body(&mut self, elem: bool) {
    self.log_body.0 = elem;
  }

  /// Constructs [VerbatimEncoder] and also increases the number of requests.
  #[inline]
  pub const fn verbatim_request<D>(&mut self, data: D) -> VerbatimEncoder<D> {
    self.increase_requests_num();
    VerbatimEncoder { data }
  }

  const fn increase_requests_num(&mut self) {
    self.built_requests = self.built_requests.wrapping_add(1);
  }
}
