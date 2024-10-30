use crate::{
  http::{ReqResBuffer, Request, Response},
  misc::{FnFut, FnFutWrapper},
};
use core::future::Future;

/// Request middleware
pub trait ReqMiddleware<CA, E, SA>
where
  E: From<crate::Error>,
{
  /// Modifies or halts requests.
  fn apply_req_middleware(
    &self,
    conn_aux: &mut CA,
    req: &mut Request<ReqResBuffer>,
    stream_aux: &mut SA,
  ) -> impl Future<Output = Result<(), E>>;
}

impl<CA, E, SA, T> ReqMiddleware<CA, E, SA> for &T
where
  E: From<crate::Error>,
  T: for<'any> FnFut<
    (&'any mut CA, &'any mut SA, &'any mut Request<ReqResBuffer>),
    Result = Result<(), E>,
  >,
{
  #[inline]
  async fn apply_req_middleware(
    &self,
    conn_aux: &mut CA,
    req: &mut Request<ReqResBuffer>,
    stream_aux: &mut SA,
  ) -> Result<(), E> {
    self.call((conn_aux, stream_aux, req)).await?;
    Ok(())
  }
}

impl<CA, E, SA, T> ReqMiddleware<CA, E, SA> for [T]
where
  E: From<crate::Error>,
  T: for<'any> FnFut<
    (&'any mut CA, &'any mut SA, &'any mut Request<ReqResBuffer>),
    Result = Result<(), E>,
  >,
{
  #[inline]
  async fn apply_req_middleware(
    &self,
    conn_aux: &mut CA,
    req: &mut Request<ReqResBuffer>,
    stream_aux: &mut SA,
  ) -> Result<(), E> {
    for elem in self {
      elem.call((conn_aux, stream_aux, req)).await?;
    }
    Ok(())
  }
}

impl<CA, E, F, SA> ReqMiddleware<CA, E, SA>
  for FnFutWrapper<(&mut CA, &mut SA, &mut Request<ReqResBuffer>), F>
where
  F: for<'any> FnFut<
    (&'any mut CA, &'any mut SA, &'any mut Request<ReqResBuffer>),
    Result = Result<(), E>,
  >,
  E: From<crate::Error>,
{
  #[inline]
  async fn apply_req_middleware(
    &self,
    conn_aux: &mut CA,
    req: &mut Request<ReqResBuffer>,
    stream_aux: &mut SA,
  ) -> Result<(), E> {
    self.0.call((conn_aux, stream_aux, req)).await?;
    Ok(())
  }
}

/// Response middleware
pub trait ResMiddleware<CA, E, SA>
where
  E: From<crate::Error>,
{
  /// Modifies or halts responses.
  fn apply_res_middleware(
    &self,
    conn_aux: &mut CA,
    res: Response<&mut ReqResBuffer>,
    stream_aux: &mut SA,
  ) -> impl Future<Output = Result<(), E>>;
}

impl<CA, E, SA, T> ResMiddleware<CA, E, SA> for &T
where
  E: From<crate::Error>,
  T: for<'any> FnFut<
    (&'any mut CA, &'any mut SA, Response<&'any mut ReqResBuffer>),
    Result = Result<(), E>,
  >,
{
  #[inline]
  async fn apply_res_middleware(
    &self,
    conn_aux: &mut CA,
    res: Response<&mut ReqResBuffer>,
    stream_aux: &mut SA,
  ) -> Result<(), E> {
    self.call((conn_aux, stream_aux, res)).await?;
    Ok(())
  }
}

impl<CA, E, SA, T> ResMiddleware<CA, E, SA> for [T]
where
  E: From<crate::Error>,
  T: for<'any> FnFut<
    (&'any mut CA, &'any mut SA, Response<&'any mut ReqResBuffer>),
    Result = Result<(), E>,
  >,
{
  #[inline]
  async fn apply_res_middleware(
    &self,
    conn_aux: &mut CA,
    res: Response<&mut ReqResBuffer>,
    stream_aux: &mut SA,
  ) -> Result<(), E> {
    for elem in self {
      let local_res =
        Response { rrd: &mut *res.rrd, status_code: res.status_code, version: res.version };
      elem.call((conn_aux, stream_aux, local_res)).await?;
    }
    Ok(())
  }
}

impl<CA, E, F, SA> ResMiddleware<CA, E, SA>
  for FnFutWrapper<(&mut CA, &mut SA, Response<&mut ReqResBuffer>), F>
where
  F: for<'any> FnFut<
    (&'any mut CA, &'any mut SA, Response<&'any mut ReqResBuffer>),
    Result = Result<(), E>,
  >,
  E: From<crate::Error>,
{
  #[inline]
  async fn apply_res_middleware(
    &self,
    conn_aux: &mut CA,
    res: Response<&mut ReqResBuffer>,
    stream_aux: &mut SA,
  ) -> Result<(), E> {
    self.0.call((conn_aux, stream_aux, res)).await?;
    Ok(())
  }
}
