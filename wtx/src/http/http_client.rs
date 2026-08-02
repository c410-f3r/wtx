use crate::{
  collections::Vector,
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
  fn send_req<MD>(
    &self,
    enc_buffer: &mut Vector<u8>,
    req: Request<MD>,
  ) -> impl Future<Output = crate::Result<Self::ReqId>>
  where
    MD: MsgData,
    MD::Body: Lease<[u8]>;

  /// Sends a request a [`MsgData`] and receives a response using [`MsgData`].
  #[inline]
  fn send_req_recv_res<MD>(
    &self,
    enc_buffer: &mut Vector<u8>,
    req: Request<MD>,
  ) -> impl Future<Output = crate::Result<Response<MsgBufferString>>>
  where
    MD: MsgData,
    MD::Body: Lease<[u8]>,
  {
    async move {
      let req_id = self.send_req(enc_buffer, req).await?;
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
  async fn send_req<MD>(
    &self,
    enc_buffer: &mut Vector<u8>,
    req: Request<MD>,
  ) -> crate::Result<Self::ReqId>
  where
    MD: MsgData,
    MD::Body: Lease<[u8]>,
  {
    (**self).send_req(enc_buffer, req).await
  }
}

#[cfg(feature = "http2")]
mod http2 {
  use crate::{
    collections::Vector,
    http::{HttpClient, MsgBufferString, MsgData, Request, Response},
    http2::{ClientStream, Http2, Http2RecvStatus},
    misc::Lease,
    net::StreamWriter,
    tls::TlsCtx,
  };

  impl<SW, TCX> HttpClient for Http2<SW, TCX, true>
  where
    SW: StreamWriter,
    TCX: TlsCtx,
  {
    type ReqId = ClientStream<SW, TCX>;

    #[inline]
    async fn recv_res(&self, mut req_id: Self::ReqId) -> crate::Result<Response<MsgBufferString>> {
      let (hrs, res_rrb) = req_id.recv_res().await?;
      let (Http2RecvStatus::ClosedStream(status_code) | Http2RecvStatus::Eos(status_code)) = hrs
      else {
        return Err(crate::Error::ClosedHttpConnection);
      };
      req_id.common().clear().await?;
      Ok(Response::new(res_rrb, status_code))
    }

    #[inline]
    async fn send_req<MD>(
      &self,
      enc_buffer: &mut Vector<u8>,
      req: Request<MD>,
    ) -> crate::Result<Self::ReqId>
    where
      MD: MsgData,
      MD::Body: Lease<[u8]>,
    {
      let mut req_id = self.stream().await?;
      if req_id.send_req(enc_buffer, req).await?.is_closed() {
        return Err(crate::Error::ClosedHttpConnection);
      }
      Ok(req_id)
    }
  }
}

#[cfg(feature = "http2-client-pool")]
mod http_client_pool {
  use crate::{
    collections::Vector,
    http::{
      HttpClient, MsgBufferString, MsgData, Request, Response,
      http2_client_pool::{Http2ClientPool, Http2ClientPoolResource, Http2RM},
    },
    http2::{ClientStream, Http2RecvStatus},
    misc::Lease,
    net::StreamWriter,
    pool::ResourceManager,
    tls::TlsCtx,
  };

  impl<AUX, EX, SW, TCX> HttpClient for Http2ClientPool<AUX, EX, TCX>
  where
    SW: StreamWriter,
    TCX: TlsCtx,
    Http2RM<AUX, EX, TCX>: ResourceManager<
        CreateAux = str,
        Error = crate::Error,
        RecycleAux = str,
        Resource = Http2ClientPoolResource<AUX, SW, TCX>,
      >,
  {
    type ReqId = ClientStream<SW, TCX>;

    #[inline]
    async fn recv_res(&self, req_id: Self::ReqId) -> crate::Result<Response<MsgBufferString>> {
      (&self).recv_res(req_id).await
    }

    #[inline]
    async fn send_req<MD>(
      &self,
      enc_buffer: &mut Vector<u8>,
      req: Request<MD>,
    ) -> crate::Result<Self::ReqId>
    where
      MD: MsgData,
      MD::Body: Lease<[u8]>,
    {
      (&self).send_req(enc_buffer, req).await
    }

    #[inline]
    async fn send_req_recv_res<MD>(
      &self,
      enc_buffer: &mut Vector<u8>,
      req: Request<MD>,
    ) -> crate::Result<Response<MsgBufferString>>
    where
      MD: MsgData,
      MD::Body: Lease<[u8]>,
    {
      let req_id = self.send_req(enc_buffer, req).await?;
      self.recv_res(req_id).await
    }
  }

  impl<AUX, EX, SW, TCX> HttpClient for &Http2ClientPool<AUX, EX, TCX>
  where
    SW: StreamWriter,
    TCX: TlsCtx,
    Http2RM<AUX, EX, TCX>: ResourceManager<
        CreateAux = str,
        Error = crate::Error,
        RecycleAux = str,
        Resource = Http2ClientPoolResource<AUX, SW, TCX>,
      >,
  {
    type ReqId = ClientStream<SW, TCX>;

    #[inline]
    async fn recv_res(&self, mut req_id: Self::ReqId) -> crate::Result<Response<MsgBufferString>> {
      let (hrs, res_rrb) = req_id.recv_res().await?;
      let (Http2RecvStatus::ClosedStream(status_code) | Http2RecvStatus::Eos(status_code)) = hrs
      else {
        return Err(crate::Error::ClosedHttpConnection);
      };
      req_id.common().clear().await?;
      Ok(Response::new(res_rrb, status_code))
    }

    #[inline]
    async fn send_req<MD>(
      &self,
      enc_buffer: &mut Vector<u8>,
      req: Request<MD>,
    ) -> crate::Result<Self::ReqId>
    where
      MD: MsgData,
      MD::Body: Lease<[u8]>,
    {
      let mut req_id = self.lock(&req.msg_data.uri()).await?.client.stream().await?;
      if req_id.send_req(enc_buffer, req).await?.is_closed() {
        return Err(crate::Error::ClosedHttpConnection);
      }
      Ok(req_id)
    }
  }
}
