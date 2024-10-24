use crate::{
  http::{ReqResBuffer, Request, Response},
  misc::{FnFut, FnFutWrapper},
};
use core::future::Future;

/// Request middleware
pub trait ReqMiddleware<CA, E, RA>
where
  E: From<crate::Error>,
{
  /// Modifies or halts requests.
  fn apply_req_middleware(
    &self,
    ca: &mut CA,
    ra: &mut RA,
    req: &mut Request<ReqResBuffer>,
  ) -> impl Future<Output = Result<(), E>>;
}

impl<CA, E, RA, T> ReqMiddleware<CA, E, RA> for &T
where
  E: From<crate::Error>,
  T: for<'any> FnFut<
    (&'any mut CA, &'any mut RA, &'any mut Request<ReqResBuffer>),
    Result = Result<(), E>,
  >,
{
  #[inline]
  async fn apply_req_middleware(
    &self,
    ca: &mut CA,
    ra: &mut RA,
    req: &mut Request<ReqResBuffer>,
  ) -> Result<(), E> {
    self.call((ca, ra, req)).await?;
    Ok(())
  }
}

impl<CA, E, RA, T> ReqMiddleware<CA, E, RA> for [T]
where
  E: From<crate::Error>,
  T: for<'any> FnFut<
    (&'any mut CA, &'any mut RA, &'any mut Request<ReqResBuffer>),
    Result = Result<(), E>,
  >,
{
  #[inline]
  async fn apply_req_middleware(
    &self,
    ca: &mut CA,
    ra: &mut RA,
    req: &mut Request<ReqResBuffer>,
  ) -> Result<(), E> {
    for elem in self {
      elem.call((ca, ra, req)).await?;
    }
    Ok(())
  }
}

impl<CA, E, F, RA> ReqMiddleware<CA, E, RA>
  for FnFutWrapper<(&mut CA, &mut RA, &mut Request<ReqResBuffer>), F>
where
  F: for<'any> FnFut<
    (&'any mut CA, &'any mut RA, &'any mut Request<ReqResBuffer>),
    Result = Result<(), E>,
  >,
  E: From<crate::Error>,
{
  #[inline]
  async fn apply_req_middleware(
    &self,
    ca: &mut CA,
    ra: &mut RA,
    req: &mut Request<ReqResBuffer>,
  ) -> Result<(), E> {
    self.0.call((ca, ra, req)).await?;
    Ok(())
  }
}

/// Response middleware
pub trait ResMiddleware<CA, E, RA>
where
  E: From<crate::Error>,
{
  /// Modifies or halts responses.
  fn apply_res_middleware(
    &self,
    ca: &mut CA,
    ra: &mut RA,
    res: Response<&mut ReqResBuffer>,
  ) -> impl Future<Output = Result<(), E>>;
}

impl<CA, E, RA, T> ResMiddleware<CA, E, RA> for &T
where
  E: From<crate::Error>,
  T: for<'any> FnFut<
    (&'any mut CA, &'any mut RA, Response<&'any mut ReqResBuffer>),
    Result = Result<(), E>,
  >,
{
  #[inline]
  async fn apply_res_middleware(
    &self,
    ca: &mut CA,
    ra: &mut RA,
    res: Response<&mut ReqResBuffer>,
  ) -> Result<(), E> {
    self.call((ca, ra, res)).await?;
    Ok(())
  }
}

impl<CA, E, RA, T> ResMiddleware<CA, E, RA> for [T]
where
  E: From<crate::Error>,
  T: for<'any> FnFut<
    (&'any mut CA, &'any mut RA, Response<&'any mut ReqResBuffer>),
    Result = Result<(), E>,
  >,
{
  #[inline]
  async fn apply_res_middleware(
    &self,
    ca: &mut CA,
    ra: &mut RA,
    res: Response<&mut ReqResBuffer>,
  ) -> Result<(), E> {
    for elem in self {
      let local_res =
        Response { rrd: &mut *res.rrd, status_code: res.status_code, version: res.version };
      elem.call((ca, ra, local_res)).await?;
    }
    Ok(())
  }
}

impl<CA, E, F, RA> ResMiddleware<CA, E, RA>
  for FnFutWrapper<(&mut CA, &mut RA, Response<&mut ReqResBuffer>), F>
where
  F: for<'any> FnFut<
    (&'any mut CA, &'any mut RA, Response<&'any mut ReqResBuffer>),
    Result = Result<(), E>,
  >,
  E: From<crate::Error>,
{
  #[inline]
  async fn apply_res_middleware(
    &self,
    ca: &mut CA,
    ra: &mut RA,
    res: Response<&mut ReqResBuffer>,
  ) -> Result<(), E> {
    self.0.call((ca, ra, res)).await?;
    Ok(())
  }
}
