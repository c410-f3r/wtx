use crate::{
  data_transformation::{
    dnsn::{Deserialize, Serialize},
    format::{ProtobufRequest, ProtobufResponse},
  },
  grpc::serialize,
  misc::Vector,
};

/// Responsible for the deserialization/serialization of `gRPC` server requests/responses.
#[derive(Debug)]
pub struct ServerData<DRSR> {
  drsr: DRSR,
}

impl<DRSR> ServerData<DRSR> {
  /// Constructor
  #[inline]
  pub fn new(drsr: DRSR) -> Self {
    Self { drsr }
  }

  /// Deserialize From Request Bytes.
  #[inline]
  pub fn des_from_req_bytes<'de, T>(&mut self, bytes: &'de [u8]) -> crate::Result<T>
  where
    ProtobufRequest<T>: Deserialize<'de, DRSR>,
  {
    let elem = if let [_, _, _, _, _, elem @ ..] = bytes { elem } else { &[] };
    Ok(ProtobufRequest::from_bytes(elem, &mut self.drsr)?.data)
  }

  /// Serialize to Response Bytes
  #[inline]
  pub fn ser_to_res_bytes<T>(&mut self, bytes: &mut Vector<u8>, data: T) -> crate::Result<()>
  where
    ProtobufResponse<T>: Serialize<DRSR>,
  {
    serialize(bytes, ProtobufResponse { data }, &mut self.drsr)
  }
}
