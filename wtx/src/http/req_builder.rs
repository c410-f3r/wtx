use crate::{
  http::{Header, KnownHeaderName, Method, Mime, ReqResBuffer, ReqUri, Request, Response},
  http2::{Http2, Http2Buffer, Http2Data, Http2RecvStatus},
  misc::{LeaseMut, Lock, RefCounter, StreamWriter},
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
  /// Constructor shortcut that has a default `GET` method
  #[inline]
  pub const fn get(rrb: ReqResBuffer) -> Self {
    Self { method: Method::Get, rrb }
  }

  /// Constructor shortcut that has a default `POST` method
  #[inline]
  pub const fn post(rrb: ReqResBuffer) -> Self {
    Self { method: Method::Get, rrb }
  }
}

impl ReqBuilder {
  /// Sends a request with inner parameters.
  #[inline]
  pub async fn send<HD, SW>(
    self,
    client: &mut Http2<HD, true>,
    req_uri: impl Into<ReqUri<'_>>,
  ) -> crate::Result<Response<ReqResBuffer>>
  where
    HD: RefCounter,
    HD::Item: Lock<Resource = Http2Data<Http2Buffer, SW, true>>,
    SW: StreamWriter,
  {
    let mut stream = client.stream().await?;
    if stream.send_req(Request::http2(self.method, &self.rrb), req_uri).await?.is_closed() {
      return Err(crate::Error::ClosedConnection);
    }
    let (hrs, res_rrb) = stream.recv_res(self.rrb).await?;
    let status_code = match hrs {
      Http2RecvStatus::Eos(elem) => elem,
      _ => return Err(crate::Error::ClosedConnection),
    };
    stream.common().clear(false).await?;
    Ok(Response::http2(res_rrb, status_code))
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

  /// Changes the method
  #[inline]
  pub fn method(mut self, method: Method) -> Self {
    self.method = method;
    self
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
