use crate::{
  data_transformation::{
    dnsn::{De, DecodeWrapper},
    format::{VerbatimRequest, VerbatimResponse},
  },
  grpc::{GrpcStatusCode, serialize},
  misc::{Decode, Encode, Vector},
};

/// Responsible for managing internal structures that interact with gRPC.
#[derive(Debug)]
pub struct GrpcManager<DRSR> {
  drsr: DRSR,
  status_code: GrpcStatusCode,
}

impl<DRSR> GrpcManager<DRSR> {
  /// From Deserializer/Serializer
  ///
  /// Instance has an initial [`GrpcStatusCode::Ok`] that can be modified in endpoints.
  #[inline]
  pub fn from_drsr(drsr: DRSR) -> Self {
    Self { drsr, status_code: GrpcStatusCode::Ok }
  }

  /// Deserialize From Request Bytes.
  #[inline]
  pub fn des_from_req_bytes<'de, T>(&mut self, bytes: &mut &'de [u8]) -> crate::Result<T>
  where
    VerbatimRequest<T>: Decode<'de, De<DRSR>>,
  {
    let elem = if let [_, _, _, _, _, elem @ ..] = bytes { elem } else { &[] };
    Ok(VerbatimRequest::decode(&mut self.drsr, &mut DecodeWrapper::new(elem))?.data)
  }

  /// Serialize to Response Bytes
  #[inline]
  pub fn ser_to_res_bytes<T>(&mut self, bytes: &mut Vector<u8>, data: T) -> crate::Result<()>
  where
    VerbatimResponse<T>: Encode<De<DRSR>>,
  {
    serialize(bytes, VerbatimResponse { data }, &mut self.drsr)
  }

  /// Allows the modification of a gRPC response status.
  #[inline]
  pub fn status_code_mut(&mut self) -> &mut GrpcStatusCode {
    &mut self.status_code
  }
}

#[cfg(feature = "grpc-server")]
impl<DRSR> crate::http::server_framework::StreamAux for GrpcManager<DRSR>
where
  DRSR: Default,
{
  type Init = DRSR;

  #[inline]
  fn stream_aux(init: Self::Init) -> crate::Result<Self> {
    Ok(GrpcManager::from_drsr(init))
  }
}
