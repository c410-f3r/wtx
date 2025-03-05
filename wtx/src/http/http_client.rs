use crate::{
  http::{Method, ReqResBuffer, ReqResData, Response},
  misc::{Lease, UriRef},
};

/// Generic HTTP client
pub trait HttpClient {
  /// Stream received in a request
  type Stream;

  /// Receives a response
  fn recv_res(
    &mut self,
    rrb: ReqResBuffer,
    stream: Self::Stream,
  ) -> impl Future<Output = crate::Result<Response<ReqResBuffer>>>;

  /// Sends a request a [`ReqResData`] and receives a response using [`ReqResBuffer`].
  #[inline]
  fn send_recv_dual<RRD>(
    &mut self,
    method: Method,
    rrb: ReqResBuffer,
    rrd: RRD,
    uri: &UriRef<'_>,
  ) -> impl Future<Output = crate::Result<Response<ReqResBuffer>>>
  where
    RRD: ReqResData,
    RRD::Body: Lease<[u8]>,
  {
    async move {
      let stream = self.send_req(method, rrd, uri).await?;
      self.recv_res(rrb, stream).await
    }
  }

  /// Sends a request and receives a response using a single [`ReqResBuffer`].
  #[inline]
  fn send_recv_single(
    &mut self,
    method: Method,
    rrb: ReqResBuffer,
    uri: &UriRef<'_>,
  ) -> impl Future<Output = crate::Result<Response<ReqResBuffer>>> {
    async move {
      let stream = self.send_req(method, &rrb, uri).await?;
      self.recv_res(rrb, stream).await
    }
  }

  /// Sends a request
  fn send_req<RRD>(
    &mut self,
    method: Method,
    rrd: RRD,
    uri: &UriRef<'_>,
  ) -> impl Future<Output = crate::Result<Self::Stream>>
  where
    RRD: ReqResData,
    RRD::Body: Lease<[u8]>;
}

impl<T> HttpClient for &mut T
where
  T: HttpClient,
{
  type Stream = T::Stream;

  #[inline]
  async fn recv_res(
    &mut self,
    rrb: ReqResBuffer,
    stream: Self::Stream,
  ) -> crate::Result<Response<ReqResBuffer>> {
    (**self).recv_res(rrb, stream).await
  }

  #[inline]
  async fn send_req<RRD>(
    &mut self,
    method: Method,
    rrd: RRD,
    uri: &UriRef<'_>,
  ) -> crate::Result<Self::Stream>
  where
    RRD: ReqResData,
    RRD::Body: Lease<[u8]>,
  {
    (**self).send_req(method, rrd, uri).await
  }
}

#[cfg(feature = "http2")]
mod http2 {
  use crate::{
    http::{HttpClient, Method, ReqResBuffer, ReqResData, Request, Response},
    http2::{ClientStream, Http2, Http2Buffer, Http2Data, Http2RecvStatus},
    misc::{Lease, LeaseMut, Lock, RefCounter, StreamWriter, UriRef},
  };

  impl<HB, HD, SW> HttpClient for Http2<HD, true>
  where
    HB: LeaseMut<Http2Buffer>,
    HD: RefCounter,
    HD::Item: Lock<Resource = Http2Data<HB, SW, true>>,
    SW: StreamWriter,
  {
    type Stream = ClientStream<HD>;

    #[inline]
    async fn recv_res(
      &mut self,
      rrb: ReqResBuffer,
      mut stream: Self::Stream,
    ) -> crate::Result<Response<ReqResBuffer>> {
      let (hrs, res_rrb) = stream.recv_res(rrb).await?;
      let status_code = match hrs {
        Http2RecvStatus::Eos(elem) => elem,
        _ => return Err(crate::Error::ClosedConnection),
      };
      stream.common().clear(false).await?;
      Ok(Response::http2(res_rrb, status_code))
    }

    #[inline]
    async fn send_req<RRD>(
      &mut self,
      method: Method,
      rrd: RRD,
      uri: &UriRef<'_>,
    ) -> crate::Result<Self::Stream>
    where
      RRD: ReqResData,
      RRD::Body: Lease<[u8]>,
    {
      let mut stream = self.stream().await?;
      if stream.send_req(Request::http2(method, rrd), uri).await?.is_closed() {
        return Err(crate::Error::ClosedConnection);
      }
      Ok(stream)
    }
  }
}

#[cfg(feature = "http-client-pool")]
mod http_client_pool {
  use crate::{
    http::{
      HttpClient, Method, ReqResBuffer, ReqResData, Request, Response,
      client_pool::{ClientPool, ClientPoolResource},
    },
    http2::{ClientStream, Http2, Http2Buffer, Http2Data, Http2RecvStatus},
    misc::{Lease, Lock, RefCounter, StreamWriter, UriRef},
    pool::{ResourceManager, SimplePoolResource},
  };

  impl<AUX, HD, RL, RM, SW> HttpClient for ClientPool<RL, RM>
  where
    HD: RefCounter,
    HD::Item: Lock<Resource = Http2Data<Http2Buffer, SW, true>>,
    RL: Lock<Resource = SimplePoolResource<RM::Resource>>,
    RM: ResourceManager<
        CreateAux = str,
        Error = crate::Error,
        RecycleAux = str,
        Resource = ClientPoolResource<AUX, Http2<HD, true>>,
      >,
    SW: StreamWriter,
    for<'any> RL: 'any,
    for<'any> RM: 'any,
  {
    type Stream = ClientStream<HD>;

    #[inline]
    async fn recv_res(
      &mut self,
      rrb: ReqResBuffer,
      stream: Self::Stream,
    ) -> crate::Result<Response<ReqResBuffer>> {
      (&*self).recv_res(rrb, stream).await
    }

    #[inline]
    async fn send_req<RRD>(
      &mut self,
      method: Method,
      rrd: RRD,
      uri: &UriRef<'_>,
    ) -> crate::Result<Self::Stream>
    where
      RRD: ReqResData,
      RRD::Body: Lease<[u8]>,
    {
      (&*self).send_req(method, rrd, uri).await
    }
  }

  impl<AUX, HD, RL, RM, SW> HttpClient for &ClientPool<RL, RM>
  where
    HD: RefCounter,
    HD::Item: Lock<Resource = Http2Data<Http2Buffer, SW, true>>,
    RL: Lock<Resource = SimplePoolResource<RM::Resource>>,
    RM: ResourceManager<
        CreateAux = str,
        Error = crate::Error,
        RecycleAux = str,
        Resource = ClientPoolResource<AUX, Http2<HD, true>>,
      >,
    SW: StreamWriter,
    for<'any> RL: 'any,
    for<'any> RM: 'any,
  {
    type Stream = ClientStream<HD>;

    #[inline]
    async fn recv_res(
      &mut self,
      rrb: ReqResBuffer,
      mut stream: Self::Stream,
    ) -> crate::Result<Response<ReqResBuffer>> {
      let (hrs, res_rrb) = stream.recv_res(rrb).await?;
      let status_code = match hrs {
        Http2RecvStatus::Eos(elem) => elem,
        _ => return Err(crate::Error::ClosedConnection),
      };
      stream.common().clear(false).await?;
      Ok(Response::http2(res_rrb, status_code))
    }

    #[inline]
    async fn send_req<RRD>(
      &mut self,
      method: Method,
      rrd: RRD,
      uri: &UriRef<'_>,
    ) -> crate::Result<Self::Stream>
    where
      RRD: ReqResData,
      RRD::Body: Lease<[u8]>,
    {
      let mut stream = self.lock(uri).await?.client.stream().await?;
      if stream.send_req(Request::http2(method, rrd), uri).await?.is_closed() {
        return Err(crate::Error::ClosedConnection);
      }
      Ok(stream)
    }
  }
}
