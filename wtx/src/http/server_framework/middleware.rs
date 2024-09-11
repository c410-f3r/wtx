use crate::{
  http::{Request, Response},
  misc::{FnFut, FnFutWrapper},
};
use core::future::Future;

/// Request middleware
pub trait ReqMiddleware<CA, E, RA, RRD>
where
  E: From<crate::Error>,
{
  /// Modifies or halts requests.
  fn apply_req_middleware(
    &self,
    ca: &mut CA,
    ra: &mut RA,
    req: &mut Request<RRD>,
  ) -> impl Future<Output = Result<(), E>>;
}

impl<CA, E, RA, RRD, T> ReqMiddleware<CA, E, RA, RRD> for &T
where
  E: From<crate::Error>,
  T: for<'any> FnFut<(&'any mut CA, &'any mut RA, &'any mut Request<RRD>), Result = Result<(), E>>,
{
  #[inline]
  async fn apply_req_middleware(
    &self,
    ca: &mut CA,
    ra: &mut RA,
    req: &mut Request<RRD>,
  ) -> Result<(), E> {
    self.call((ca, ra, req)).await?;
    Ok(())
  }
}

impl<CA, E, RA, RRD, T> ReqMiddleware<CA, E, RA, RRD> for [T]
where
  E: From<crate::Error>,
  T: for<'any> FnFut<(&'any mut CA, &'any mut RA, &'any mut Request<RRD>), Result = Result<(), E>>,
{
  #[inline]
  async fn apply_req_middleware(
    &self,
    ca: &mut CA,
    ra: &mut RA,
    req: &mut Request<RRD>,
  ) -> Result<(), E> {
    for elem in self {
      elem.call((ca, ra, req)).await?;
    }
    Ok(())
  }
}

impl<CA, E, F, RA, RRD> ReqMiddleware<CA, E, RA, RRD>
  for FnFutWrapper<(&mut CA, &mut RA, &mut Request<RRD>), F>
where
  F: for<'any> FnFut<(&'any mut CA, &'any mut RA, &'any mut Request<RRD>), Result = Result<(), E>>,
  E: From<crate::Error>,
{
  #[inline]
  async fn apply_req_middleware(
    &self,
    ca: &mut CA,
    ra: &mut RA,
    req: &mut Request<RRD>,
  ) -> Result<(), E> {
    self.0.call((ca, ra, req)).await?;
    Ok(())
  }
}

/// Response middleware
pub trait ResMiddleware<CA, E, RA, RRD>
where
  E: From<crate::Error>,
{
  /// Modifies or halts responses.
  fn apply_res_middleware(
    &self,
    ca: &mut CA,
    ra: &mut RA,
    res: Response<&mut RRD>,
  ) -> impl Future<Output = Result<(), E>>;
}

impl<CA, E, RA, RRD, T> ResMiddleware<CA, E, RA, RRD> for &T
where
  E: From<crate::Error>,
  T: for<'any> FnFut<(&'any mut CA, &'any mut RA, Response<&'any mut RRD>), Result = Result<(), E>>,
{
  #[inline]
  async fn apply_res_middleware(
    &self,
    ca: &mut CA,
    ra: &mut RA,
    res: Response<&mut RRD>,
  ) -> Result<(), E> {
    self.call((ca, ra, res)).await?;
    Ok(())
  }
}

impl<CA, E, RA, RRD, T> ResMiddleware<CA, E, RA, RRD> for [T]
where
  E: From<crate::Error>,
  T: for<'any> FnFut<(&'any mut CA, &'any mut RA, Response<&'any mut RRD>), Result = Result<(), E>>,
{
  #[inline]
  async fn apply_res_middleware(
    &self,
    ca: &mut CA,
    ra: &mut RA,
    res: Response<&mut RRD>,
  ) -> Result<(), E> {
    for elem in self {
      let local_res =
        Response { rrd: &mut *res.rrd, status_code: res.status_code, version: res.version };
      elem.call((ca, ra, local_res)).await?;
    }
    Ok(())
  }
}

impl<CA, E, F, RA, RRD> ResMiddleware<CA, E, RA, RRD>
  for FnFutWrapper<(&mut CA, &mut RA, Response<&mut RRD>), F>
where
  F: for<'any> FnFut<(&'any mut CA, &'any mut RA, Response<&'any mut RRD>), Result = Result<(), E>>,
  E: From<crate::Error>,
{
  #[inline]
  async fn apply_res_middleware(
    &self,
    ca: &mut CA,
    ra: &mut RA,
    res: Response<&mut RRD>,
  ) -> Result<(), E> {
    self.0.call((ca, ra, res)).await?;
    Ok(())
  }
}
