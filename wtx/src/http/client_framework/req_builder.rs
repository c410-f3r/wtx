use crate::{
  http::{
    client_framework::ClientFramework, Header, KnownHeaderName, Method, Mime, ReqResBuffer, ReqUri,
    Response,
  },
  http2::{Http2, Http2Buffer, Http2Data},
  misc::{LeaseMut, Lock, RefCounter, StreamWriter},
  pool::{ResourceManager, SimplePoolResource},
};

/// Request builder
///
/// Provides shortcuts to manipulate requests through a fluent interface.
///
/// It is also possible to work directly with fields.
#[derive(Debug)]
pub struct ReqBuilder {
  /// Method
  pub method: Method,
  /// Buffer
  pub rrb: ReqResBuffer,
}

impl ReqBuilder {
  /// Constructor shortcut
  #[inline]
  pub const fn new(method: Method, rrb: ReqResBuffer) -> Self {
    Self { method, rrb }
  }
}

impl ReqBuilder {
  /// A instance suitable for `GET` requests.
  #[inline]
  pub fn get(rrb: ReqResBuffer) -> Self {
    Self { method: Method::Get, rrb }
  }

  /// Sends a request with inner parameters.
  #[inline]
  pub async fn send<HD, RL, RM, SW>(
    self,
    client: &ClientFramework<RL, RM>,
    req_uri: impl Into<ReqUri<'_>>,
  ) -> crate::Result<Response<ReqResBuffer>>
  where
    HD: RefCounter + 'static,
    HD::Item: Lock<Resource = Http2Data<Http2Buffer, SW, true>>,
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
    client.send(self.method, self.rrb, req_uri).await
  }

  /// Media type of the resource.
  #[inline]
  pub fn content_type(mut self, mime: Mime) -> crate::Result<Self> {
    self.rrb.lease_mut().headers.push_from_iter(Header::from_name_and_value(
      KnownHeaderName::ContentType.into(),
      [mime.as_str().as_bytes()],
    ))?;
    Ok(self)
  }

  /// Characteristic string that lets servers and network peers identify the application.
  #[inline]
  pub fn user_agent(mut self, value: &[u8]) -> crate::Result<Self> {
    self
      .rrb
      .lease_mut()
      .headers
      .push_from_iter(Header::from_name_and_value(KnownHeaderName::UserAgent.into(), [value]))?;
    Ok(self)
  }
}
