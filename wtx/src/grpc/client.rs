use crate::{
  data_transformation::{
    dnsn::{Deserialize, Serialize},
    format::{ProtobufRequest, ProtobufResponse},
  },
  grpc::serialize,
  http::{
    client_framework::ClientFramework, Header, Headers, KnownHeaderName, Method, ReqResBuffer,
    ReqUri, Response,
  },
  http2::{Http2, Http2Buffer, Http2Data},
  misc::{Lock, RefCounter, StreamWriter},
  pool::{ResourceManager, SimplePoolResource},
};

const WTX_USER_AGENT: &str = concat!("wtx/", env!("CARGO_PKG_VERSION"));

/// Performs requests to gRPC servers.
#[derive(Debug)]
pub struct Client<DRSR, RL, RM> {
  cf: ClientFramework<RL, RM>,
  drsr: DRSR,
}

impl<DRSR, HD, RL, RM, SW> Client<DRSR, RL, RM>
where
  HD: RefCounter + 'static,
  HD::Item: Lock<Resource = Http2Data<Http2Buffer<ReqResBuffer>, ReqResBuffer, SW, true>>,
  RL: Lock<Resource = SimplePoolResource<RM::Resource>>,
  RM: ResourceManager<
    CreateAux = str,
    Error = crate::Error,
    RecycleAux = str,
    Resource = Http2<HD, true>,
  >,
  SW: StreamWriter,
  for<'any> RL: 'any,
  for<'any> RM: 'any,
{
  /// Constructor
  #[inline]
  pub fn new(cf: ClientFramework<RL, RM>, drsr: DRSR) -> Self {
    Self { cf, drsr }
  }

  /// Deserialize From Response Bytes
  #[inline]
  pub fn des_from_res_bytes<'de, T>(&mut self, bytes: &'de [u8]) -> crate::Result<T>
  where
    ProtobufResponse<T>: Deserialize<'de, DRSR>,
  {
    let elem = if let [_, _, _, _, _, elem @ ..] = bytes { elem } else { &[] };
    Ok(ProtobufResponse::from_bytes(elem, &mut self.drsr)?.data)
  }

  /// Send Unary Request
  ///
  /// Builds a valid unary gRPC request and awaits for a raw response.
  ///
  /// It is necessary to call [`Self::des_from_res_bytes`] to create the corresponding decoded element.
  #[inline]
  pub async fn send_unary_req<T>(
    &mut self,
    (package, service, method): (&str, &str, &str),
    data: T,
    mut rrb: ReqResBuffer,
  ) -> crate::Result<Response<ReqResBuffer>>
  where
    ProtobufRequest<T>: Serialize<DRSR>,
  {
    rrb.data.clear();
    rrb.headers.clear();
    rrb.uri.truncate_with_initial_len();
    rrb.uri.push_path(format_args!("/{package}.{service}/{method}"))?;
    serialize(&mut rrb.data, ProtobufRequest { data }, &mut self.drsr)?;
    Self::push_headers(&mut rrb.headers)?;
    let res = self.cf.send(Method::Post, rrb, ReqUri::Data).await?;
    Ok(Response::http2(res.rrd, res.status_code))
  }

  fn push_headers(headers: &mut Headers) -> crate::Result<()> {
    let array = [
      Header {
        is_sensitive: false,
        is_trailer: false,
        name: KnownHeaderName::ContentType.into(),
        value: b"application/grpc",
      },
      Header {
        is_sensitive: false,
        is_trailer: false,
        name: KnownHeaderName::Te.into(),
        value: b"trailers",
      },
      Header {
        is_sensitive: false,
        is_trailer: false,
        name: KnownHeaderName::UserAgent.into(),
        value: WTX_USER_AGENT.as_bytes(),
      },
    ];
    headers.set_max_bytes(64);
    for header in array {
      headers.push_front(header, &[])?;
    }
    Ok(())
  }
}
