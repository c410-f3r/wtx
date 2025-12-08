use crate::{
  collection::Vector,
  http::{ReqBuilder, ReqResBuffer, ReqResData, Response},
  misc::Lease,
};

/// Generic HTTP client
pub trait HttpClient {
  /// If applicable, can be used by clients to poll specific requests returned
  /// by [`HttpClient::send_req`].
  type ReqId;

  /// Receives a response
  fn recv_res(
    &self,
    rrb: ReqResBuffer,
    req_id: Self::ReqId,
  ) -> impl Future<Output = crate::Result<Response<ReqResBuffer>>>;

  /// Sends a request
  fn send_req<RRD>(
    &self,
    enc_buffer: &mut Vector<u8>,
    rb: ReqBuilder<RRD>,
  ) -> impl Future<Output = crate::Result<Self::ReqId>>
  where
    RRD: ReqResData,
    RRD::Body: Lease<[u8]>;

  /// Sends a request a [`ReqResData`] and receives a response using [`ReqResBuffer`].
  #[inline]
  fn send_req_recv_res<RRD>(
    &self,
    mut rrb: ReqResBuffer,
    rb: ReqBuilder<RRD>,
  ) -> impl Future<Output = crate::Result<Response<ReqResBuffer>>>
  where
    RRD: ReqResData,
    RRD::Body: Lease<[u8]>,
  {
    async move {
      let req_id = self.send_req(&mut rrb.body, rb).await?;
      self.recv_res(rrb, req_id).await
    }
  }
}

impl<T> HttpClient for &mut T
where
  T: HttpClient,
{
  type ReqId = T::ReqId;

  #[inline]
  async fn recv_res(
    &self,
    rrb: ReqResBuffer,
    req_id: Self::ReqId,
  ) -> crate::Result<Response<ReqResBuffer>> {
    (**self).recv_res(rrb, req_id).await
  }

  #[inline]
  async fn send_req<RRD>(
    &self,
    enc_buffer: &mut Vector<u8>,
    rb: ReqBuilder<RRD>,
  ) -> crate::Result<Self::ReqId>
  where
    RRD: ReqResData,
    RRD::Body: Lease<[u8]>,
  {
    (**self).send_req(enc_buffer, rb).await
  }
}

#[cfg(feature = "http2")]
mod http2 {
  use crate::{
    collection::Vector,
    http::{HttpClient, ReqBuilder, ReqResBuffer, ReqResData, Response},
    http2::{ClientStream, Http2, Http2Buffer, Http2RecvStatus},
    misc::{Lease, LeaseMut},
    stream::StreamWriter,
  };

  impl<HB, SW> HttpClient for Http2<HB, SW, true>
  where
    HB: LeaseMut<Http2Buffer>,
    SW: StreamWriter,
  {
    type ReqId = ClientStream<HB, SW>;

    #[inline]
    async fn recv_res(
      &self,
      rrb: ReqResBuffer,
      mut req_id: Self::ReqId,
    ) -> crate::Result<Response<ReqResBuffer>> {
      let (hrs, res_rrb) = req_id.recv_res(rrb).await?;
      let status_code = match hrs {
        Http2RecvStatus::ClosedStream(elem) | Http2RecvStatus::Eos(elem) => elem,
        _ => return Err(crate::Error::ClosedHttpConnection),
      };
      req_id.common().clear(false).await?;
      Ok(Response::http2(res_rrb, status_code))
    }

    #[inline]
    async fn send_req<RRD>(
      &self,
      enc_buffer: &mut Vector<u8>,
      rb: ReqBuilder<RRD>,
    ) -> crate::Result<Self::ReqId>
    where
      RRD: ReqResData,
      RRD::Body: Lease<[u8]>,
    {
      let mut req_id = self.stream().await?;
      if req_id.send_req(enc_buffer, rb.into_request()).await?.is_closed() {
        return Err(crate::Error::ClosedHttpConnection);
      }
      Ok(req_id)
    }
  }
}

#[cfg(feature = "http-client-pool")]
mod http_client_pool {
  use crate::{
    collection::Vector,
    http::{
      HttpClient, ReqBuilder, ReqResBuffer, ReqResData, Response,
      client_pool::{ClientPool, ClientPoolResource},
    },
    http2::{ClientStream, Http2, Http2Buffer, Http2RecvStatus},
    misc::{Lease, LeaseMut},
    pool::ResourceManager,
    stream::StreamWriter,
  };

  impl<AUX, HB, RM, SW> HttpClient for ClientPool<RM>
  where
    HB: LeaseMut<Http2Buffer>,
    RM: ResourceManager<
        CreateAux = str,
        Error = crate::Error,
        RecycleAux = str,
        Resource = ClientPoolResource<AUX, Http2<HB, SW, true>>,
      >,
    SW: StreamWriter,
  {
    type ReqId = ClientStream<HB, SW>;

    #[inline]
    async fn recv_res(
      &self,
      rrb: ReqResBuffer,
      req_id: Self::ReqId,
    ) -> crate::Result<Response<ReqResBuffer>> {
      (&self).recv_res(rrb, req_id).await
    }

    #[inline]
    async fn send_req<RRD>(
      &self,
      enc_buffer: &mut Vector<u8>,
      rb: ReqBuilder<RRD>,
    ) -> crate::Result<Self::ReqId>
    where
      RRD: ReqResData,
      RRD::Body: Lease<[u8]>,
    {
      (&self).send_req(enc_buffer, rb).await
    }
  }

  impl<AUX, HB, RM, SW> HttpClient for &ClientPool<RM>
  where
    HB: LeaseMut<Http2Buffer>,
    RM: ResourceManager<
        CreateAux = str,
        Error = crate::Error,
        RecycleAux = str,
        Resource = ClientPoolResource<AUX, Http2<HB, SW, true>>,
      >,
    SW: StreamWriter,
  {
    type ReqId = ClientStream<HB, SW>;

    #[inline]
    async fn recv_res(
      &self,
      rrb: ReqResBuffer,
      mut req_id: Self::ReqId,
    ) -> crate::Result<Response<ReqResBuffer>> {
      let (hrs, res_rrb) = req_id.recv_res(rrb).await?;
      let status_code = match hrs {
        Http2RecvStatus::ClosedStream(elem) | Http2RecvStatus::Eos(elem) => elem,
        _ => return Err(crate::Error::ClosedHttpConnection),
      };
      req_id.common().clear(false).await?;
      Ok(Response::http2(res_rrb, status_code))
    }

    #[inline]
    async fn send_req<RRD>(
      &self,
      enc_buffer: &mut Vector<u8>,
      rb: ReqBuilder<RRD>,
    ) -> crate::Result<Self::ReqId>
    where
      RRD: ReqResData,
      RRD::Body: Lease<[u8]>,
    {
      let mut req_id = self.lock(&rb.rrb.rrd.uri()).await?.client.stream().await?;
      if req_id.send_req(enc_buffer, rb.into_request()).await?.is_closed() {
        return Err(crate::Error::ClosedHttpConnection);
      }
      Ok(req_id)
    }
  }
}
