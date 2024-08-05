use crate::{
  http::{Client, Header, KnownHeaderName, Method, ReqResBuffer, ReqResData, ReqUri, Response},
  http2::{Http2, Http2Buffer, Http2Data},
  misc::{LeaseMut, Lock, RefCounter, Stream},
  pool::{ResourceManager, SimplePoolResource},
};

/// Request builder
///
/// Provides shortcuts to manipulate requests through a fluent interface.
///
/// It is also possible to work directly with fields.
#[derive(Debug)]
pub struct ReqBuilder<RRB> {
  /// Method
  pub method: Method,
  /// Buffer
  pub rrb: RRB,
}

impl<RRB> ReqBuilder<RRB>
where
  RRB: LeaseMut<ReqResBuffer>,
{
  /// Constructor shortcut
  #[inline]
  pub const fn new(method: Method, rrb: RRB) -> Self {
    Self { method, rrb }
  }
}

impl<RRB> ReqBuilder<RRB>
where
  RRB: LeaseMut<ReqResBuffer>,
{
  /// Sends a request with inner parameters.
  #[inline]
  pub async fn send<HD, RL, RM, S>(
    self,
    client: &Client<RL, RM>,
    req_uri: impl Into<ReqUri<'_>>,
  ) -> crate::Result<Response<RRB>>
  where
    HD: RefCounter + 'static,
    HD::Item: Lock<Resource = Http2Data<Http2Buffer<RRB>, RRB, S, true>>,
    RL: Lock<Resource = SimplePoolResource<RM::Resource>>,
    RM: ResourceManager<
      CreateAux = str,
      Error = crate::Error,
      RecycleAux = str,
      Resource = Http2<HD, true>,
    >,
    RRB: LeaseMut<ReqResBuffer> + ReqResData,
    S: Stream,
    for<'any> RL: 'any,
    for<'any> RM: 'any,
  {
    client.send(self.method, self.rrb, req_uri).await
  }

  /// Characteristic string that lets servers and network peers identify the application.
  #[inline]
  pub fn user_agent(mut self, value: &[u8]) -> crate::Result<Self> {
    self.rrb.lease_mut().headers_mut().push_front(
      Header {
        is_sensitive: false,
        is_trailer: false,
        name: KnownHeaderName::UserAgent.into(),
        value,
      },
      &[],
    )?;
    Ok(self)
  }
}

impl<RRB> ReqBuilder<RRB>
where
  RRB: Default,
{
  /// Performs a heap allocation to create a buffer suitable for `GET` requests.
  #[inline]
  pub fn get() -> Self {
    Self { method: Method::Get, rrb: RRB::default() }
  }
}
