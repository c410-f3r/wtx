use crate::{
  de::{
    Decode, Encode,
    format::{De, DecodeWrapper},
    protocol::{VerbatimDecoder, VerbatimEncoder},
  },
  grpc::serialize,
  http::{
    Header, Headers, HttpClient, KnownHeaderName, Method, ReqResBuffer, Response, WTX_USER_AGENT,
  },
  http2::{Http2, Http2Buffer, Http2Data},
  misc::{LeaseMut, SingleTypeStorage, UriRef},
  stream::StreamWriter,
  sync::{Lock, RefCounter},
};

/// Performs requests to gRPC servers.
#[derive(Debug)]
pub struct GrpcClient<C, DRSR> {
  client: C,
  drsr: DRSR,
}

impl<C, DRSR, HD, SW> GrpcClient<C, DRSR>
where
  C: LeaseMut<Http2<HD, true>> + SingleTypeStorage<Item = HD>,
  HD: RefCounter,
  HD::Item: Lock<Resource = Http2Data<Http2Buffer, SW, true>>,
  SW: StreamWriter,
{
  /// Constructor
  #[inline]
  pub const fn new(client: C, drsr: DRSR) -> Self {
    Self { client, drsr }
  }

  /// Deserialize From Response Bytes
  #[inline]
  pub fn des_from_res_bytes<'de, T>(&mut self, bytes: &mut &'de [u8]) -> crate::Result<T>
  where
    VerbatimDecoder<T>: Decode<'de, De<DRSR>>,
  {
    let elem = if let [_, _, _, _, _, elem @ ..] = bytes { elem } else { &[] };
    Ok(VerbatimDecoder::decode(&mut self.drsr, &mut DecodeWrapper::new(elem))?.data)
  }

  /// Send Unary Request
  ///
  /// Builds a valid unary gRPC request and awaits for a raw response.
  ///
  /// It is necessary to call [`Self::des_from_res_bytes`] to create the corresponding decoded element.
  #[inline]
  pub async fn send_unary_req<T>(
    &mut self,
    data: T,
    mut rrb: ReqResBuffer,
    uri: &UriRef<'_>,
  ) -> crate::Result<Response<ReqResBuffer>>
  where
    VerbatimEncoder<T>: Encode<De<DRSR>>,
  {
    rrb.clear();
    serialize(&mut rrb.body, VerbatimEncoder { data }, &mut self.drsr)?;
    Self::push_headers(&mut rrb.headers)?;
    let res = self.client.lease_mut().send_recv_single(Method::Post, rrb, uri).await?;
    Ok(Response::http2(res.rrd, res.status_code))
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
