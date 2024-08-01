use crate::{
  client_api_framework::{
    misc::{manage_after_sending_related, manage_before_sending_related},
    network::{
      transport::{Transport, TransportParams},
      HttpParams, HttpReqParams, TransportGroup,
    },
    pkg::{Package, PkgsAux},
    Api,
  },
  http::{Client, Header, Headers, KnownHeaderName, ReqResBuffer},
  http2::{Http2, Http2Buffer, Http2Data},
  misc::{Lock, RefCounter, Stream, Vector},
  pool::{ResourceManager, SimplePoolResource},
};
use core::{mem, ops::Range};

impl<DRSR, HD, RL, RM, S> Transport<DRSR> for Client<RL, RM>
where
  HD: RefCounter + 'static,
  HD::Item: Lock<Resource = Http2Data<Http2Buffer<ReqResBuffer>, ReqResBuffer, S, true>>,
  RL: Lock<Resource = SimplePoolResource<RM::Resource>>,
  RM: ResourceManager<
    CreateAux = str,
    Error = crate::Error,
    RecycleAux = str,
    Resource = Http2<HD, true>,
  >,
  S: Stream,
  for<'any> RL: 'any,
  for<'any> RM: 'any,
{
  const GROUP: TransportGroup = TransportGroup::HTTP;
  type Params = HttpParams;

  #[inline]
  async fn send<A, P>(
    &mut self,
    pkg: &mut P,
    pkgs_aux: &mut PkgsAux<A, DRSR, HttpParams>,
  ) -> Result<(), A::Error>
  where
    A: Api,
    P: Package<A, DRSR, HttpParams>,
  {
    response(self, pkg, pkgs_aux).await?;
    Ok(())
  }

  #[inline]
  async fn send_recv<A, P>(
    &mut self,
    pkg: &mut P,
    pkgs_aux: &mut PkgsAux<A, DRSR, HttpParams>,
  ) -> Result<Range<usize>, A::Error>
  where
    A: Api,
    P: Package<A, DRSR, HttpParams>,
  {
    response(self, pkg, pkgs_aux).await?;
    Ok(0..pkgs_aux.byte_buffer.len())
  }
}

impl<DRSR, HD, RL, RM, S> Transport<DRSR> for &Client<RL, RM>
where
  HD: RefCounter + 'static,
  HD::Item: Lock<Resource = Http2Data<Http2Buffer<ReqResBuffer>, ReqResBuffer, S, true>>,
  RL: Lock<Resource = SimplePoolResource<RM::Resource>>,
  RM: ResourceManager<
    CreateAux = str,
    Error = crate::Error,
    RecycleAux = str,
    Resource = Http2<HD, true>,
  >,
  S: Stream,
  for<'any> RL: 'any,
  for<'any> RM: 'any,
{
  const GROUP: TransportGroup = TransportGroup::HTTP;
  type Params = HttpParams;

  #[inline]
  async fn send<A, P>(
    &mut self,
    pkg: &mut P,
    pkgs_aux: &mut PkgsAux<A, DRSR, HttpParams>,
  ) -> Result<(), A::Error>
  where
    A: Api,
    P: Package<A, DRSR, HttpParams>,
  {
    response(self, pkg, pkgs_aux).await?;
    Ok(())
  }

  #[inline]
  async fn send_recv<A, P>(
    &mut self,
    pkg: &mut P,
    pkgs_aux: &mut PkgsAux<A, DRSR, HttpParams>,
  ) -> Result<Range<usize>, A::Error>
  where
    A: Api,
    P: Package<A, DRSR, HttpParams>,
  {
    response(self, pkg, pkgs_aux).await?;
    Ok(0..pkgs_aux.byte_buffer.len())
  }
}

async fn response<A, DRSR, HD, P, RL, RM, S>(
  client: &Client<RL, RM>,
  pkg: &mut P,
  pkgs_aux: &mut PkgsAux<A, DRSR, HttpParams>,
) -> Result<(), A::Error>
where
  A: Api,
  P: Package<A, DRSR, HttpParams>,
  HD: RefCounter + 'static,
  HD::Item: Lock<Resource = Http2Data<Http2Buffer<ReqResBuffer>, ReqResBuffer, S, true>>,
  RL: Lock<Resource = SimplePoolResource<RM::Resource>>,
  RM: ResourceManager<
    CreateAux = str,
    Error = crate::Error,
    RecycleAux = str,
    Resource = Http2<HD, true>,
  >,
  S: Stream,
  for<'any> RL: 'any,
  for<'any> RM: 'any,
{
  pkgs_aux.byte_buffer.clear();
  pkgs_aux.tp.ext_req_params_mut().headers.clear();
  manage_before_sending_related(pkg, pkgs_aux, &*client).await?;
  let HttpReqParams { headers, method, mime, uri, user_agent } = pkgs_aux.tp.ext_req_params_mut();
  if let Some(elem) = mime {
    headers.push_front(
      Header {
        is_sensitive: false,
        is_trailer: false,
        name: KnownHeaderName::ContentType.into(),
        value: elem._as_str().as_bytes(),
      },
      &[],
    )?;
  }
  if let Some(elem) = user_agent {
    headers.push_front(
      Header {
        is_sensitive: false,
        is_trailer: false,
        name: KnownHeaderName::UserAgent.into(),
        value: elem._as_str().as_bytes(),
      },
      &[],
    )?;
  }
  let rrb = {
    let (mut rrb_data, mut rrb_headers) = (Vector::new(), Headers::new(0));
    mem::swap(&mut rrb_data, &mut pkgs_aux.byte_buffer);
    mem::swap(&mut rrb_headers, headers);
    ReqResBuffer::new(rrb_data, rrb_headers)
  };
  {
    let res = (*client).send(*method, rrb, &uri.to_ref()).await?;
    let (mut rrb_data, mut rrb_headers) = res.rrd.into_parts();
    mem::swap(&mut pkgs_aux.byte_buffer, &mut rrb_data);
    mem::swap(headers, &mut rrb_headers);
    pkgs_aux.tp.ext_res_params_mut().status_code = res.status_code;
  }
  manage_after_sending_related(pkg, pkgs_aux).await?;
  pkgs_aux.tp.reset();
  Ok(())
}
