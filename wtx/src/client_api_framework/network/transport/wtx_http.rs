use crate::{
  client_api_framework::{
    Api,
    misc::{
      log_http_req, manage_after_sending_bytes, manage_after_sending_pkg,
      manage_before_sending_bytes, manage_before_sending_pkg,
    },
    network::{
      HttpParams, HttpReqParams, HttpResParams, TransportGroup,
      transport::{
        ReceivingTransport, SendingTransport, Transport, TransportParams, local_send_bytes,
        log_http_res,
      },
    },
    pkg::{Package, PkgsAux},
  },
  http::{HttpClient, ReqBuilder, ReqResBuffer, WTX_USER_AGENT},
  http2::{ClientStream, Http2, Http2Buffer},
  misc::LeaseMut,
  stream::StreamWriter,
};
use core::mem;

impl<HB, SW, TP> ReceivingTransport<TP> for Http2<HB, SW, true>
where
  HB: LeaseMut<Http2Buffer>,
  SW: StreamWriter,
  TP: LeaseMut<HttpParams>,
{
  #[inline]
  async fn recv<A, DRSR>(
    &mut self,
    pkgs_aux: &mut PkgsAux<A, DRSR, TP>,
    req_id: Self::ReqId,
  ) -> Result<(), A::Error>
  where
    A: Api,
  {
    recv(self, pkgs_aux, req_id).await?;
    Ok(())
  }
}
impl<HB, SW, TP> SendingTransport<TP> for Http2<HB, SW, true>
where
  HB: LeaseMut<Http2Buffer>,
  SW: StreamWriter,
  TP: LeaseMut<HttpParams>,
{
  #[inline]
  async fn send_bytes<A, DRSR>(
    &mut self,
    bytes: &[u8],
    pkgs_aux: &mut PkgsAux<A, DRSR, TP>,
  ) -> Result<Self::ReqId, A::Error>
  where
    A: Api,
  {
    send_bytes(bytes, self, pkgs_aux).await
  }

  #[inline]
  async fn send_pkg<A, DRSR, P>(
    &mut self,
    pkg: &mut P,
    pkgs_aux: &mut PkgsAux<A, DRSR, TP>,
  ) -> Result<Self::ReqId, A::Error>
  where
    A: Api,
    P: Package<A, DRSR, Self::Inner, TP>,
  {
    send_pkg(self, pkg, pkgs_aux).await
  }
}
impl<HB, SW, TP> Transport<TP> for Http2<HB, SW, true>
where
  SW: StreamWriter,
{
  const GROUP: TransportGroup = TransportGroup::HTTP;
  type Inner = Self;
  type ReqId = ClientStream<HB, SW>;
}

fn manage_params<A, DRSR, TP>(
  bytes_len: usize,
  pkgs_aux: &mut PkgsAux<A, DRSR, TP>,
) -> Result<(), A::Error>
where
  A: Api,
  TP: LeaseMut<HttpParams>,
{
  let tp = pkgs_aux.tp.lease_mut();
  let params = &mut tp.ext_params_mut().0;
  let HttpReqParams { host, method, mime, rrb, user_agent_custom, user_agent_default } = params;
  let mut rb = ReqBuilder::method(*method, rrb);
  if *host {
    let _ = rb.host(None)?;
  }
  if *user_agent_default {
    let _ = rb.user_agent(WTX_USER_AGENT)?;
  } else if let Some(elem) = user_agent_custom
    && !elem.is_empty()
  {
    let _ = rb.user_agent(elem)?;
  }
  if let Some(elem) = mime
    && bytes_len > 0
  {
    let _ = rb.content_type(*elem)?;
  }
  Ok(())
}

async fn recv<A, DRSR, HB, SW, TP>(
  client: &mut Http2<HB, SW, true>,
  pkgs_aux: &mut PkgsAux<A, DRSR, TP>,
  req_id: ClientStream<HB, SW>,
) -> Result<(), A::Error>
where
  A: Api,
  HB: LeaseMut<Http2Buffer>,
  SW: StreamWriter,
  TP: LeaseMut<HttpParams>,
{
  let should_log_body = pkgs_aux.should_log_body();
  let tp = pkgs_aux.tp.lease_mut();
  let (req_params, res_params) = tp.ext_params_mut();
  let HttpReqParams { rrb, .. } = req_params;
  let HttpResParams { status_code } = res_params;
  let mut local_rrb = ReqResBuffer::empty();
  mem::swap(&mut local_rrb.body, &mut pkgs_aux.bytes_buffer);
  mem::swap(&mut local_rrb.headers, &mut rrb.headers);
  let mut res = client.recv_res(local_rrb, req_id).await?;
  mem::swap(&mut res.rrd.body, &mut pkgs_aux.bytes_buffer);
  mem::swap(&mut res.rrd.headers, &mut rrb.headers);
  *status_code = res.status_code;
  log_http_res(
    &pkgs_aux.bytes_buffer,
    should_log_body,
    res.status_code,
    TransportGroup::HTTP,
    &rrb.uri,
  );
  Ok(())
}

async fn send_bytes<A, DRSR, HB, SW, TP>(
  bytes: &[u8],
  client: &mut Http2<HB, SW, true>,
  pkgs_aux: &mut PkgsAux<A, DRSR, TP>,
) -> Result<ClientStream<HB, SW>, A::Error>
where
  A: Api,
  HB: LeaseMut<Http2Buffer>,
  SW: StreamWriter,
  TP: LeaseMut<HttpParams>,
{
  manage_before_sending_bytes(pkgs_aux).await?;
  let local_bytes0 = local_send_bytes(bytes, &pkgs_aux.bytes_buffer, pkgs_aux.send_bytes_buffer);
  log_http_req::<_, TP>(
    local_bytes0,
    pkgs_aux.should_log_body(),
    pkgs_aux.tp.lease().ext_req_params().method,
    client,
    &pkgs_aux.tp.lease().ext_req_params().rrb.uri,
  );
  manage_params(local_bytes0.len(), pkgs_aux)?;
  let HttpReqParams { method, rrb, .. } = &mut pkgs_aux.tp.lease_mut().ext_params_mut().0;
  let local_bytes = local_send_bytes(bytes, &pkgs_aux.bytes_buffer, pkgs_aux.send_bytes_buffer);
  let rb = ReqBuilder::method(*method, (local_bytes, &rrb.headers, rrb.uri.to_ref()));
  let rslt = client.send_req(&mut rrb.body, rb).await?;
  manage_after_sending_bytes(pkgs_aux).await?;
  Ok(rslt)
}

async fn send_pkg<A, DRSR, HB, P, SW, TP>(
  client: &mut Http2<HB, SW, true>,
  pkg: &mut P,
  pkgs_aux: &mut PkgsAux<A, DRSR, TP>,
) -> Result<ClientStream<HB, SW>, A::Error>
where
  A: Api,
  HB: LeaseMut<Http2Buffer>,
  P: Package<A, DRSR, Http2<HB, SW, true>, TP>,
  SW: StreamWriter,
  TP: LeaseMut<HttpParams>,
{
  manage_before_sending_pkg(pkg, pkgs_aux, client).await?;
  log_http_req::<_, TP>(
    &pkgs_aux.bytes_buffer,
    pkgs_aux.should_log_body(),
    pkgs_aux.tp.lease().ext_req_params().method,
    client,
    &pkgs_aux.tp.lease().ext_req_params().rrb.uri,
  );
  manage_params(pkgs_aux.bytes_buffer.len(), pkgs_aux)?;
  let HttpReqParams { method, rrb, .. } = &mut pkgs_aux.tp.lease_mut().ext_params_mut().0;
  let rb = ReqBuilder::method(*method, (&pkgs_aux.bytes_buffer, &rrb.headers, rrb.uri.to_ref()));
  let rslt = client.send_req(&mut rrb.body, rb).await?;
  manage_after_sending_pkg(pkg, pkgs_aux, client).await?;
  Ok(rslt)
}

#[cfg(feature = "http-client-pool")]
mod http_client_pool {
  use crate::{
    client_api_framework::{
      Api,
      network::{
        HttpParams, TransportGroup,
        transport::{
          ReceivingTransport, SendingTransport, Transport, TransportParams,
          wtx_http::{recv, send_bytes, send_pkg},
        },
      },
      pkg::{Package, PkgsAux},
    },
    http::client_pool::{ClientPool, ClientPoolResource},
    http2::{ClientStream, Http2, Http2Buffer},
    misc::LeaseMut,
    pool::ResourceManager,
    stream::StreamWriter,
  };

  impl<AUX, HB, RM, SW, TP> ReceivingTransport<TP> for ClientPool<RM>
  where
    HB: LeaseMut<Http2Buffer>,
    RM: ResourceManager<
        CreateAux = str,
        Error = crate::Error,
        RecycleAux = str,
        Resource = ClientPoolResource<AUX, Http2<HB, SW, true>>,
      >,
    SW: StreamWriter,
    TP: LeaseMut<HttpParams>,
  {
    #[inline]
    async fn recv<A, DRSR>(
      &mut self,
      pkgs_aux: &mut PkgsAux<A, DRSR, TP>,
      req_id: Self::ReqId,
    ) -> Result<(), A::Error>
    where
      A: Api,
    {
      recv(
        &mut self
          .lock(&pkgs_aux.tp.lease_mut().ext_req_params_mut().rrb.uri.to_ref())
          .await?
          .client,
        pkgs_aux,
        req_id,
      )
      .await?;
      Ok(())
    }
  }
  impl<AUX, HB, RM, SW, TP> SendingTransport<TP> for ClientPool<RM>
  where
    HB: LeaseMut<Http2Buffer>,
    RM: ResourceManager<
        CreateAux = str,
        Error = crate::Error,
        RecycleAux = str,
        Resource = ClientPoolResource<AUX, Http2<HB, SW, true>>,
      >,
    SW: StreamWriter,
    TP: LeaseMut<HttpParams>,
  {
    #[inline]
    async fn send_bytes<A, DRSR>(
      &mut self,
      bytes: &[u8],
      pkgs_aux: &mut PkgsAux<A, DRSR, TP>,
    ) -> Result<Self::ReqId, A::Error>
    where
      A: Api,
    {
      send_bytes(
        bytes,
        &mut self
          .lock(&pkgs_aux.tp.lease_mut().ext_req_params_mut().rrb.uri.to_ref())
          .await?
          .client,
        pkgs_aux,
      )
      .await
    }

    #[inline]
    async fn send_pkg<A, DRSR, P>(
      &mut self,
      pkg: &mut P,
      pkgs_aux: &mut PkgsAux<A, DRSR, TP>,
    ) -> Result<Self::ReqId, A::Error>
    where
      A: Api,
      P: Package<A, DRSR, Self::Inner, TP>,
    {
      send_pkg(
        &mut self
          .lock(&pkgs_aux.tp.lease_mut().ext_req_params_mut().rrb.uri.to_ref())
          .await?
          .client,
        pkg,
        pkgs_aux,
      )
      .await
    }
  }
  impl<AUX, HB, RM, SW, TP> Transport<TP> for ClientPool<RM>
  where
    RM: ResourceManager<
        CreateAux = str,
        Error = crate::Error,
        RecycleAux = str,
        Resource = ClientPoolResource<AUX, Http2<HB, SW, true>>,
      >,
    SW: StreamWriter,
  {
    const GROUP: TransportGroup = TransportGroup::HTTP;
    type Inner = Http2<HB, SW, true>;
    type ReqId = ClientStream<HB, SW>;
  }

  impl<AUX, HB, RM, SW, TP> ReceivingTransport<TP> for &ClientPool<RM>
  where
    HB: LeaseMut<Http2Buffer>,
    RM: ResourceManager<
        CreateAux = str,
        Error = crate::Error,
        RecycleAux = str,
        Resource = ClientPoolResource<AUX, Http2<HB, SW, true>>,
      >,
    SW: StreamWriter,
    TP: LeaseMut<HttpParams>,
  {
    #[inline]
    async fn recv<A, DRSR>(
      &mut self,
      pkgs_aux: &mut PkgsAux<A, DRSR, TP>,
      req_id: Self::ReqId,
    ) -> Result<(), A::Error>
    where
      A: Api,
    {
      recv(
        &mut self
          .lock(&pkgs_aux.tp.lease_mut().ext_req_params_mut().rrb.uri.to_ref())
          .await?
          .client,
        pkgs_aux,
        req_id,
      )
      .await?;
      Ok(())
    }
  }
  impl<AUX, HB, RM, SW, TP> SendingTransport<TP> for &ClientPool<RM>
  where
    HB: LeaseMut<Http2Buffer>,
    RM: ResourceManager<
        CreateAux = str,
        Error = crate::Error,
        RecycleAux = str,
        Resource = ClientPoolResource<AUX, Http2<HB, SW, true>>,
      >,
    SW: StreamWriter,
    TP: LeaseMut<HttpParams>,
  {
    #[inline]
    async fn send_bytes<A, DRSR>(
      &mut self,
      bytes: &[u8],
      pkgs_aux: &mut PkgsAux<A, DRSR, TP>,
    ) -> Result<Self::ReqId, A::Error>
    where
      A: Api,
    {
      send_bytes(
        bytes,
        &mut self
          .lock(&pkgs_aux.tp.lease_mut().ext_req_params_mut().rrb.uri.to_ref())
          .await?
          .client,
        pkgs_aux,
      )
      .await
    }

    #[inline]
    async fn send_pkg<A, DRSR, P>(
      &mut self,
      pkg: &mut P,
      pkgs_aux: &mut PkgsAux<A, DRSR, TP>,
    ) -> Result<Self::ReqId, A::Error>
    where
      A: Api,
      P: Package<A, DRSR, Self::Inner, TP>,
    {
      send_pkg(
        &mut self
          .lock(&pkgs_aux.tp.lease_mut().ext_req_params_mut().rrb.uri.to_ref())
          .await?
          .client,
        pkg,
        pkgs_aux,
      )
      .await
    }
  }
  impl<AUX, HB, RM, SW, TP> Transport<TP> for &ClientPool<RM>
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
    const GROUP: TransportGroup = TransportGroup::HTTP;
    type Inner = Http2<HB, SW, true>;
    type ReqId = ClientStream<HB, SW>;
  }
}
