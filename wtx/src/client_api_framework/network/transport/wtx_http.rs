use crate::{
  client_api_framework::{
    Api, SendBytesSource,
    misc::{
      log_http_req, manage_after_sending_bytes, manage_after_sending_pkg,
      manage_before_sending_bytes, manage_before_sending_pkg,
    },
    network::{
      HttpParams, HttpReqParams, HttpResParams, TransportGroup,
      transport::{ReceivingTransport, SendingTransport, Transport, TransportParams, log_http_res},
    },
    pkg::{Package, PkgsAux},
  },
  http::{HttpClient, ReqResBuffer, ResBuilder, WTX_USER_AGENT},
  http2::{ClientStream, Http2, Http2Buffer, Http2Data},
  misc::LeaseMut,
  stream::StreamWriter,
  sync::{Lock, RefCounter},
};
use core::mem;

impl<HD, SW, TP> ReceivingTransport<TP> for Http2<HD, true>
where
  HD: RefCounter,
  HD::Item: Lock<Resource = Http2Data<Http2Buffer, SW, true>>,
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
impl<HD, SW, TP> SendingTransport<TP> for Http2<HD, true>
where
  HD: RefCounter,
  HD::Item: Lock<Resource = Http2Data<Http2Buffer, SW, true>>,
  SW: StreamWriter,
  TP: LeaseMut<HttpParams>,
{
  #[inline]
  async fn send_bytes<A, DRSR>(
    &mut self,
    bytes: SendBytesSource<'_>,
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
impl<HD, SW, TP> Transport<TP> for Http2<HD, true>
where
  HD: RefCounter,
  HD::Item: Lock<Resource = Http2Data<Http2Buffer, SW, true>>,
  SW: StreamWriter,
{
  const GROUP: TransportGroup = TransportGroup::HTTP;
  type Inner = Self;
  type ReqId = ClientStream<HD>;
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
  let HttpReqParams { headers, host, method: _, mime, uri, user_agent_custom, user_agent_default } =
    params;
  let mut rb = ResBuilder::ok(headers);
  if *host {
    let _ = rb.host(format_args!("{}", uri.host()))?;
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

async fn recv<A, DRSR, HD, SW, TP>(
  client: &mut Http2<HD, true>,
  pkgs_aux: &mut PkgsAux<A, DRSR, TP>,
  req_id: ClientStream<HD>,
) -> Result<(), A::Error>
where
  A: Api,
  HD: RefCounter,
  HD::Item: Lock<Resource = Http2Data<Http2Buffer, SW, true>>,
  SW: StreamWriter,
  TP: LeaseMut<HttpParams>,
{
  let tp = pkgs_aux.tp.lease_mut();
  let (req_params, res_params) = tp.ext_params_mut();
  let HttpReqParams {
    headers,
    host: _,
    method: _,
    mime: _,
    uri,
    user_agent_custom: _,
    user_agent_default: _,
  } = req_params;
  let HttpResParams { status_code } = res_params;
  let mut rrb = ReqResBuffer::empty();
  mem::swap(&mut rrb.body, &mut pkgs_aux.byte_buffer);
  mem::swap(&mut rrb.headers, headers);
  rrb.clear();
  let mut res = client.recv_res(rrb, req_id).await?;
  mem::swap(&mut res.rrd.body, &mut pkgs_aux.byte_buffer);
  mem::swap(&mut res.rrd.headers, headers);
  *status_code = res.status_code;
  log_http_res(
    &pkgs_aux.byte_buffer,
    pkgs_aux.log_body.1,
    res.status_code,
    TransportGroup::HTTP,
    uri,
  );
  Ok(())
}

async fn send_bytes<A, DRSR, HD, SW, TP>(
  bytes: SendBytesSource<'_>,
  client: &mut Http2<HD, true>,
  pkgs_aux: &mut PkgsAux<A, DRSR, TP>,
) -> Result<ClientStream<HD>, A::Error>
where
  A: Api,
  HD: RefCounter,
  HD::Item: Lock<Resource = Http2Data<Http2Buffer, SW, true>>,
  SW: StreamWriter,
  TP: LeaseMut<HttpParams>,
{
  manage_before_sending_bytes(pkgs_aux).await?;
  log_http_req::<_, TP>(
    bytes.bytes(&pkgs_aux.byte_buffer),
    pkgs_aux.log_body.1,
    pkgs_aux.tp.lease().ext_req_params().method,
    client,
    &pkgs_aux.tp.lease().ext_req_params().uri,
  );
  manage_params(bytes.bytes(&pkgs_aux.byte_buffer).len(), pkgs_aux)?;
  let params = pkgs_aux.tp.lease_mut();
  let HttpReqParams {
    headers,
    host: _,
    method,
    mime: _,
    uri,
    user_agent_custom: _,
    user_agent_default: _,
  } = &mut params.ext_params_mut().0;
  let rrd = (bytes.bytes(&pkgs_aux.byte_buffer), headers);
  let rslt = client.send_req(*method, rrd, &uri.to_ref()).await?;
  manage_after_sending_bytes(pkgs_aux).await?;
  Ok(rslt)
}

async fn send_pkg<A, DRSR, HD, P, SW, TP>(
  client: &mut Http2<HD, true>,
  pkg: &mut P,
  pkgs_aux: &mut PkgsAux<A, DRSR, TP>,
) -> Result<ClientStream<HD>, A::Error>
where
  A: Api,
  HD: RefCounter,
  HD::Item: Lock<Resource = Http2Data<Http2Buffer, SW, true>>,
  P: Package<A, DRSR, Http2<HD, true>, TP>,
  SW: StreamWriter,
  TP: LeaseMut<HttpParams>,
{
  manage_before_sending_pkg(pkg, pkgs_aux, client).await?;
  log_http_req::<_, TP>(
    &pkgs_aux.byte_buffer,
    pkgs_aux.log_body.1,
    pkgs_aux.tp.lease().ext_req_params().method,
    client,
    &pkgs_aux.tp.lease().ext_req_params().uri,
  );
  manage_params(pkgs_aux.byte_buffer.len(), pkgs_aux)?;
  let params = pkgs_aux.tp.lease_mut();
  let HttpReqParams {
    headers,
    host: _,
    method,
    mime: _,
    uri,
    user_agent_custom: _,
    user_agent_default: _,
  } = &mut params.ext_params_mut().0;
  let rslt = client.send_req(*method, (&pkgs_aux.byte_buffer, headers), &uri.to_ref()).await?;
  manage_after_sending_pkg(pkg, pkgs_aux, client).await?;
  Ok(rslt)
}

#[cfg(feature = "http-client-pool")]
mod http_client_pool {
  use crate::{
    client_api_framework::{
      Api, SendBytesSource,
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
    http2::{ClientStream, Http2, Http2Buffer, Http2Data},
    misc::LeaseMut,
    pool::{ResourceManager, SimplePoolResource},
    stream::StreamWriter,
    sync::{Lock, RefCounter},
  };

  impl<AUX, HD, RL, RM, SW, TP> ReceivingTransport<TP> for ClientPool<RL, RM>
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
    TP: LeaseMut<HttpParams>,
    for<'any> RL: 'any,
    for<'any> RM: 'any,
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
        &mut self.lock(&pkgs_aux.tp.lease_mut().ext_req_params_mut().uri.to_ref()).await?.client,
        pkgs_aux,
        req_id,
      )
      .await?;
      Ok(())
    }
  }
  impl<AUX, HD, RL, RM, SW, TP> SendingTransport<TP> for ClientPool<RL, RM>
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
    TP: LeaseMut<HttpParams>,
    for<'any> RL: 'any,
    for<'any> RM: 'any,
  {
    #[inline]
    async fn send_bytes<A, DRSR>(
      &mut self,
      bytes: SendBytesSource<'_>,
      pkgs_aux: &mut PkgsAux<A, DRSR, TP>,
    ) -> Result<Self::ReqId, A::Error>
    where
      A: Api,
    {
      send_bytes(
        bytes,
        &mut self.lock(&pkgs_aux.tp.lease_mut().ext_req_params_mut().uri.to_ref()).await?.client,
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
        &mut self.lock(&pkgs_aux.tp.lease_mut().ext_req_params_mut().uri.to_ref()).await?.client,
        pkg,
        pkgs_aux,
      )
      .await
    }
  }
  impl<AUX, HD, RL, RM, SW, TP> Transport<TP> for ClientPool<RL, RM>
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
  {
    const GROUP: TransportGroup = TransportGroup::HTTP;
    type Inner = Http2<HD, true>;
    type ReqId = ClientStream<HD>;
  }

  impl<AUX, HD, RL, RM, SW, TP> ReceivingTransport<TP> for &ClientPool<RL, RM>
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
    TP: LeaseMut<HttpParams>,
    for<'any> RL: 'any,
    for<'any> RM: 'any,
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
        &mut self.lock(&pkgs_aux.tp.lease_mut().ext_req_params_mut().uri.to_ref()).await?.client,
        pkgs_aux,
        req_id,
      )
      .await?;
      Ok(())
    }
  }
  impl<AUX, HD, RL, RM, SW, TP> SendingTransport<TP> for &ClientPool<RL, RM>
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
    TP: LeaseMut<HttpParams>,
    for<'any> RL: 'any,
    for<'any> RM: 'any,
  {
    #[inline]
    async fn send_bytes<A, DRSR>(
      &mut self,
      bytes: SendBytesSource<'_>,
      pkgs_aux: &mut PkgsAux<A, DRSR, TP>,
    ) -> Result<Self::ReqId, A::Error>
    where
      A: Api,
    {
      send_bytes(
        bytes,
        &mut self.lock(&pkgs_aux.tp.lease_mut().ext_req_params_mut().uri.to_ref()).await?.client,
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
        &mut self.lock(&pkgs_aux.tp.lease_mut().ext_req_params_mut().uri.to_ref()).await?.client,
        pkg,
        pkgs_aux,
      )
      .await
    }
  }
  impl<AUX, HD, RL, RM, SW, TP> Transport<TP> for &ClientPool<RL, RM>
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
    const GROUP: TransportGroup = TransportGroup::HTTP;
    type Inner = Http2<HD, true>;
    type ReqId = ClientStream<HD>;
  }
}
