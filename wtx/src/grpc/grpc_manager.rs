use crate::{
  codec::{
    Decode, Encode, GenericCodec, GenericDecodeWrapper,
    protocol::{VerbatimDecoder, VerbatimEncoder},
  },
  collection::Vector,
  grpc::{GrpcStatusCode, serialize},
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
  pub const fn from_drsr(drsr: DRSR) -> Self {
    Self { drsr, status_code: GrpcStatusCode::Ok }
  }

  /// Deserialize From Request Bytes.
  #[inline]
  pub fn des_from_req_bytes<'de, T>(&mut self, bytes: &mut &'de [u8]) -> crate::Result<T>
  where
    VerbatimEncoder<T>: for<'drsr> Decode<'de, GenericCodec<&'drsr mut DRSR, &'drsr mut DRSR>>,
  {
    let elem = if let [_, _, _, _, _, elem @ ..] = bytes { elem } else { &[] };
    Ok(VerbatimEncoder::decode(&mut GenericDecodeWrapper::new(elem, &mut self.drsr))?.data)
  }

  /// Serialize to Response Bytes
  #[inline]
  pub fn ser_to_res_bytes<T>(&mut self, bytes: &mut Vector<u8>, data: T) -> crate::Result<()>
  where
    VerbatimDecoder<T>: for<'drsr> Encode<GenericCodec<&'drsr mut DRSR, &'drsr mut DRSR>>,
  {
    serialize(bytes, VerbatimDecoder { data }, &mut self.drsr)
  }

  /// Allows the modification of a gRPC response status.
  #[inline]
  pub const fn status_code_mut(&mut self) -> &mut GrpcStatusCode {
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
