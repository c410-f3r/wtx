use crate::{
  codec::{
    Decode, DecodeWrapper, Encode, GenericCodec,
    protocol::{VerbatimDecoder, VerbatimEncoder},
  },
  collection::Clear,
  grpc::serialize,
  http::{
    Header, Headers, HttpClient, KnownHeaderName, MsgBuffer, MsgBufferString, MsgDataMut,
    ReqBuilder, Response, WTX_USER_AGENT,
  },
  misc::Lease,
};

/// Performs requests to gRPC servers.
#[derive(Debug)]
pub struct GrpcClient<C, DRSR> {
  client: C,
  drsr: DRSR,
}

impl<C, DRSR> GrpcClient<C, DRSR>
where
  C: HttpClient,
{
  /// Constructor
  #[inline]
  pub const fn new(client: C, drsr: DRSR) -> Self {
    Self { client, drsr }
  }

  /// Deserialize From Response Bytes
  #[inline]
  pub fn des_from_res_bytes<'de, T>(&mut self, bytes: &'de [u8]) -> crate::Result<T>
  where
    VerbatimDecoder<T>: for<'drsr> Decode<'de, GenericCodec<&'drsr mut DRSR, &'drsr mut DRSR>>,
  {
    let elem = if let [_, _, _, _, _, elem @ ..] = bytes { elem } else { &[] };
    Ok(VerbatimDecoder::decode(&mut DecodeWrapper::new(elem, &mut self.drsr))?.data)
  }

  /// Send Unary Request
  ///
  /// Builds a valid unary gRPC request and awaits for a raw response.
  ///
  /// It is necessary to call [`Self::des_from_res_bytes`] to create the corresponding decoded element.
  #[inline]
  pub async fn send_unary_req<S, T>(
    &mut self,
    data: T,
    mut msg_buffer: MsgBuffer<S>,
  ) -> crate::Result<Response<MsgBufferString>>
  where
    S: Clear + Lease<str>,
    VerbatimEncoder<T>: for<'drsr> Encode<GenericCodec<&'drsr mut DRSR, &'drsr mut DRSR>>,
  {
    msg_buffer.clear_body_and_headers();
    serialize(&mut msg_buffer.body, VerbatimEncoder { data }, &mut self.drsr)?;
    Self::push_headers(&mut msg_buffer.headers)?;
    let rb = ReqBuilder::post(msg_buffer);
    let req_id = self.client.send_req(rb.into_request()).await?;
    let res = self.client.recv_res(req_id).await?;
    Ok(Response::new(res.msg_data, res.status_code))
  }

  #[inline]
  fn push_headers(headers: &mut Headers) -> crate::Result<()> {
    headers.push_from_iter_many([
      Header::from_name_and_value(
        KnownHeaderName::ContentType.into(),
        ["application/grpc"].into_iter(),
      ),
      Header::from_name_and_value(KnownHeaderName::Te.into(), ["trailers"].into_iter()),
      Header::from_name_and_value(KnownHeaderName::UserAgent.into(), [WTX_USER_AGENT].into_iter()),
    ])?;
    Ok(())
  }
}
