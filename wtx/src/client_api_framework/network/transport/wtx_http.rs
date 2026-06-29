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
        ReceivingTransport, SendingTransport, Transport, TransportParams as _, local_send_bytes,
        log_http_res,
      },
    },
    pkg::{Package, PkgsAux},
  },
  http::{HttpClient as _, MsgBufferString, ReqBuilder, WTX_USER_AGENT},
  http2::{ClientStream, Http2},
  misc::LeaseMut,
  stream::StreamWriter,
  tls::TlsMode,
};
use core::mem;

impl<SW, TM, TP> ReceivingTransport<TP> for Http2<SW, TM, true>
where
  SW: StreamWriter,
  TM: TlsMode,
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
impl<SW, TM, TP> SendingTransport<TP> for Http2<SW, TM, true>
where
  SW: StreamWriter,
  TM: TlsMode,
  TP: LeaseMut<HttpParams>,
{
  #[inline]
  async fn send_bytes<A, DRSR>(
    &mut self,
    bytes: Option<&[u8]>,
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
impl<SW, TM, TP> Transport<TP> for Http2<SW, TM, true>
where
  SW: StreamWriter,
{
  const GROUP: TransportGroup = TransportGroup::HTTP;
  type Inner = Self;
  type ReqId = ClientStream<SW, TM>;
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
  let params = &mut *tp.ext_params_mut().0;
  let HttpReqParams { host, method, mime, msg_buffer, user_agent_custom, user_agent_default } =
    params;
  let mut rb = ReqBuilder::new(*method, msg_buffer);
  if *host {
    let _ = rb.host::<()>(None)?;
  }
  if *user_agent_default {
    let _ = rb.user_agent(&[WTX_USER_AGENT])?;
  } else if let Some(elem) = user_agent_custom
    && !elem.is_empty()
  {
    let _ = rb.user_agent(&[*elem])?;
  }
  if let Some(elem) = mime
    && bytes_len > 0
  {
    let _ = rb.content_type(*elem)?;
  }
  Ok(())
}

async fn recv<A, DRSR, SW, TM, TP>(
  client: &mut Http2<SW, TM, true>,
  pkgs_aux: &mut PkgsAux<A, DRSR, TP>,
  req_id: ClientStream<SW, TM>,
) -> Result<(), A::Error>
where
  A: Api,
  SW: StreamWriter,
  TM: TlsMode,
  TP: LeaseMut<HttpParams>,
{
  let log_data = pkgs_aux.log_data;
  let tp = pkgs_aux.tp.lease_mut();
  let (req_params, resp_params) = tp.ext_params_mut();
  let HttpReqParams { msg_buffer, .. } = req_params;
  let HttpResParams { status_code } = resp_params;
  let mut local_rrb = MsgBufferString::default();
  mem::swap(&mut local_rrb.body, &mut pkgs_aux.bytes_buffer);
  mem::swap(&mut local_rrb.headers, &mut msg_buffer.headers);
  let mut res = client.recv_res(req_id).await?;
  mem::swap(&mut res.msg_data.body, &mut pkgs_aux.bytes_buffer);
  mem::swap(&mut res.msg_data.headers, &mut msg_buffer.headers);
  *status_code = res.status_code;
  log_http_res(
    &pkgs_aux.bytes_buffer,
    log_data,
    res.status_code,
    TransportGroup::HTTP,
    &msg_buffer.uri,
  );
  Ok(())
}

async fn send_bytes<A, DRSR, SW, TM, TP>(
  bytes: Option<&[u8]>,
  client: &mut Http2<SW, TM, true>,
  pkgs_aux: &mut PkgsAux<A, DRSR, TP>,
) -> Result<ClientStream<SW, TM>, A::Error>
where
  A: Api,
  SW: StreamWriter,
  TM: TlsMode,
  TP: LeaseMut<HttpParams>,
{
  manage_before_sending_bytes(pkgs_aux).await?;
  let PkgsAux { log_data, tp, .. } = pkgs_aux;
  {
    let HttpReqParams { method, msg_buffer, .. } = tp.lease_mut().ext_params_mut().0;
    let local_bytes0 = local_send_bytes(bytes, &pkgs_aux.bytes_buffer);
    log_http_req::<_, TP>(local_bytes0, *log_data, *method, client, &msg_buffer.uri);
    manage_params(local_bytes0.len(), pkgs_aux)?;
  }
  {
    let HttpReqParams { method, msg_buffer, .. } = pkgs_aux.tp.lease_mut().ext_params_mut().0;
    let local_bytes1 = local_send_bytes(bytes, &pkgs_aux.bytes_buffer);
    let rb = ReqBuilder::new(*method, (local_bytes1, &msg_buffer.headers, msg_buffer.uri.to_ref()));
    let rslt = client.send_req(&mut msg_buffer.body, rb.into_request()).await?;
    manage_after_sending_bytes(pkgs_aux).await?;
    Ok(rslt)
  }
}

async fn send_pkg<A, DRSR, P, SW, TM, TP>(
  client: &mut Http2<SW, TM, true>,
  pkg: &mut P,
  pkgs_aux: &mut PkgsAux<A, DRSR, TP>,
) -> Result<ClientStream<SW, TM>, A::Error>
where
  A: Api,
  P: Package<A, DRSR, Http2<SW, TM, true>, TP>,
  SW: StreamWriter,
  TM: TlsMode,
  TP: LeaseMut<HttpParams>,
{
  manage_before_sending_pkg(pkg, pkgs_aux, client).await?;
  log_http_req::<_, TP>(
    &pkgs_aux.bytes_buffer,
    pkgs_aux.log_data,
    pkgs_aux.tp.lease().ext_req_params().method,
    client,
    &pkgs_aux.tp.lease().ext_req_params().msg_buffer.uri,
  );
  manage_params(pkgs_aux.bytes_buffer.len(), pkgs_aux)?;
  let HttpReqParams { method, msg_buffer, .. } = &mut *pkgs_aux.tp.lease_mut().ext_params_mut().0;
  let rb = ReqBuilder::new(
    *method,
    (&pkgs_aux.bytes_buffer, &msg_buffer.headers, msg_buffer.uri.to_ref()),
  );
  let rslt = client.send_req(&mut msg_buffer.body, rb.into_request()).await?;
  manage_after_sending_pkg(pkg, pkgs_aux, client).await?;
  Ok(rslt)
}

#[cfg(feature = "http2-client-pool")]
mod http_client_pool {
  use crate::{
    client_api_framework::{
      Api,
      network::{
        HttpParams, TransportGroup,
        transport::{
          ReceivingTransport, SendingTransport, Transport, TransportParams as _,
          wtx_http::{recv, send_bytes, send_pkg},
        },
      },
      pkg::{Package, PkgsAux},
    },
    http::http2_client_pool::{Http2ClientPool, Http2RM, Http2Resource},
    http2::{ClientStream, Http2},
    misc::LeaseMut,
    pool::ResourceManager,
    stream::StreamWriter,
    tls::TlsMode,
  };

  impl<EX, SW, TM, TP> ReceivingTransport<TP> for Http2ClientPool<EX, TM>
  where
    SW: StreamWriter,
    TM: TlsMode,
    TP: LeaseMut<HttpParams>,
    Http2RM<EX, TM>: ResourceManager<
        CreateAux = str,
        Error = crate::Error,
        RecycleAux = str,
        Resource = Http2Resource<SW, TM>,
      >,
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
          .lock(&pkgs_aux.tp.lease_mut().ext_req_params_mut().msg_buffer.uri.to_ref())
          .await?
          .client,
        pkgs_aux,
        req_id,
      )
      .await?;
      Ok(())
    }
  }
  impl<EX, SW, TM, TP> SendingTransport<TP> for Http2ClientPool<EX, TM>
  where
    SW: StreamWriter,
    TM: TlsMode,
    TP: LeaseMut<HttpParams>,
    Http2RM<EX, TM>: ResourceManager<
        CreateAux = str,
        Error = crate::Error,
        RecycleAux = str,
        Resource = Http2Resource<SW, TM>,
      >,
  {
    #[inline]
    async fn send_bytes<A, DRSR>(
      &mut self,
      bytes: Option<&[u8]>,
      pkgs_aux: &mut PkgsAux<A, DRSR, TP>,
    ) -> Result<Self::ReqId, A::Error>
    where
      A: Api,
    {
      send_bytes(
        bytes,
        &mut self
          .lock(&pkgs_aux.tp.lease_mut().ext_req_params_mut().msg_buffer.uri.to_ref())
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
          .lock(&pkgs_aux.tp.lease_mut().ext_req_params_mut().msg_buffer.uri.to_ref())
          .await?
          .client,
        pkg,
        pkgs_aux,
      )
      .await
    }
  }
  impl<EX, SW, TM, TP> Transport<TP> for Http2ClientPool<EX, TM>
  where
    SW: StreamWriter,
    Http2RM<EX, TM>: ResourceManager<
        CreateAux = str,
        Error = crate::Error,
        RecycleAux = str,
        Resource = Http2Resource<SW, TM>,
      >,
  {
    const GROUP: TransportGroup = TransportGroup::HTTP;
    type Inner = Http2<SW, TM, true>;
    type ReqId = ClientStream<SW, TM>;
  }

  impl<EX, SW, TM, TP> ReceivingTransport<TP> for &Http2ClientPool<EX, TM>
  where
    SW: StreamWriter,
    TM: TlsMode,
    TP: LeaseMut<HttpParams>,
    Http2RM<EX, TM>: ResourceManager<
        CreateAux = str,
        Error = crate::Error,
        RecycleAux = str,
        Resource = Http2Resource<SW, TM>,
      >,
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
          .lock(&pkgs_aux.tp.lease_mut().ext_req_params_mut().msg_buffer.uri.to_ref())
          .await?
          .client,
        pkgs_aux,
        req_id,
      )
      .await?;
      Ok(())
    }
  }
  impl<EX, SW, TM, TP> SendingTransport<TP> for &Http2ClientPool<EX, TM>
  where
    SW: StreamWriter,
    TM: TlsMode,
    TP: LeaseMut<HttpParams>,
    Http2RM<EX, TM>: ResourceManager<
        CreateAux = str,
        Error = crate::Error,
        RecycleAux = str,
        Resource = Http2Resource<SW, TM>,
      >,
  {
    #[inline]
    async fn send_bytes<A, DRSR>(
      &mut self,
      bytes: Option<&[u8]>,
      pkgs_aux: &mut PkgsAux<A, DRSR, TP>,
    ) -> Result<Self::ReqId, A::Error>
    where
      A: Api,
    {
      send_bytes(
        bytes,
        &mut self
          .lock(&pkgs_aux.tp.lease_mut().ext_req_params_mut().msg_buffer.uri.to_ref())
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
          .lock(&pkgs_aux.tp.lease_mut().ext_req_params_mut().msg_buffer.uri.to_ref())
          .await?
          .client,
        pkg,
        pkgs_aux,
      )
      .await
    }
  }
  impl<EX, SW, TM, TP> Transport<TP> for &Http2ClientPool<EX, TM>
  where
    SW: StreamWriter,
    Http2RM<EX, TM>: ResourceManager<
        CreateAux = str,
        Error = crate::Error,
        RecycleAux = str,
        Resource = Http2Resource<SW, TM>,
      >,
  {
    const GROUP: TransportGroup = TransportGroup::HTTP;
    type Inner = Http2<SW, TM, true>;
    type ReqId = ClientStream<SW, TM>;
  }
}
