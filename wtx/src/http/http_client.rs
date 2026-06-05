use crate::{
  http::{MsgBufferString, MsgData, Request, Response},
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
    req_id: Self::ReqId,
  ) -> impl Future<Output = crate::Result<Response<MsgBufferString>>>;

  /// Sends a request
  fn send_req<MD>(&self, req: Request<MD>) -> impl Future<Output = crate::Result<Self::ReqId>>
  where
    MD: MsgData,
    MD::Body: Lease<[u8]>;

  /// Sends a request a [`MsgData`] and receives a response using [`MsgData`].
  #[inline]
  fn send_req_recv_res<MD>(
    &self,
    req: Request<MD>,
  ) -> impl Future<Output = crate::Result<Response<MsgBufferString>>>
  where
    MD: MsgData,
    MD::Body: Lease<[u8]>,
  {
    async move {
      let req_id = self.send_req(req).await?;
      self.recv_res(req_id).await
    }
  }
}

impl<T> HttpClient for &mut T
where
  T: HttpClient,
{
  type ReqId = T::ReqId;

  #[inline]
  async fn recv_res(&self, req_id: Self::ReqId) -> crate::Result<Response<MsgBufferString>> {
    (**self).recv_res(req_id).await
  }

  #[inline]
  async fn send_req<MD>(&self, req: Request<MD>) -> crate::Result<Self::ReqId>
  where
    MD: MsgData,
    MD::Body: Lease<[u8]>,
  {
    (**self).send_req(req).await
  }
}

#[cfg(feature = "http2")]
mod http2 {
  use crate::{
    http::{HttpClient, MsgBufferString, MsgData, Request, Response},
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
    async fn recv_res(&self, mut req_id: Self::ReqId) -> crate::Result<Response<MsgBufferString>> {
      let (hrs, res_rrb) = req_id.recv_res().await?;
      let status_code = match hrs {
        Http2RecvStatus::ClosedStream(elem) | Http2RecvStatus::Eos(elem) => elem,
        _ => return Err(crate::Error::ClosedHttpConnection),
      };
      req_id.common().clear().await?;
      Ok(Response::http2(res_rrb, status_code))
    }

    #[inline]
    async fn send_req<MD>(&self, req: Request<MD>) -> crate::Result<Self::ReqId>
    where
      MD: MsgData,
      MD::Body: Lease<[u8]>,
    {
      let mut req_id = self.stream().await?;
      if req_id.send_req(req).await?.is_closed() {
        return Err(crate::Error::ClosedHttpConnection);
      }
      Ok(req_id)
    }
  }
}

#[cfg(feature = "http-client-pool")]
mod http_client_pool {
  use crate::{
    http::{
      HttpClient, MsgBufferString, MsgData, Request, Response,
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
    async fn recv_res(&self, req_id: Self::ReqId) -> crate::Result<Response<MsgBufferString>> {
      (&self).recv_res(req_id).await
    }

    #[inline]
    async fn send_req<MD>(&self, rb: Request<MD>) -> crate::Result<Self::ReqId>
    where
      MD: MsgData,
      MD::Body: Lease<[u8]>,
    {
      (&self).send_req(rb).await
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
    async fn recv_res(&self, mut req_id: Self::ReqId) -> crate::Result<Response<MsgBufferString>> {
      let (hrs, res_rrb) = req_id.recv_res().await?;
      let status_code = match hrs {
        Http2RecvStatus::ClosedStream(elem) | Http2RecvStatus::Eos(elem) => elem,
        _ => return Err(crate::Error::ClosedHttpConnection),
      };
      req_id.common().clear().await?;
      Ok(Response::http2(res_rrb, status_code))
    }

    #[inline]
    async fn send_req<MD>(&self, req: Request<MD>) -> crate::Result<Self::ReqId>
    where
      MD: MsgData,
      MD::Body: Lease<[u8]>,
    {
      let mut req_id = self.lock(&req.msg_data.uri()).await?.client.stream().await?;
      if req_id.send_req(req).await?.is_closed() {
        return Err(crate::Error::ClosedHttpConnection);
      }
      Ok(req_id)
    }
  }
}
