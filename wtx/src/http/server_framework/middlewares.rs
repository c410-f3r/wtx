use crate::{
  http::{Request, Response},
  misc::FnFut1,
};
use core::future::Future;

/// Requests middlewares
pub trait ReqMiddlewares<E, RRD>
where
  E: From<crate::Error>,
{
  /// Modifies or halts requests.
  fn apply_req_middlewares(&self, _: &mut Request<RRD>) -> impl Future<Output = Result<(), E>>;
}

impl<E, RRD, T> ReqMiddlewares<E, RRD> for &T
where
  E: From<crate::Error>,
  T: for<'any> FnFut1<&'any mut Request<RRD>, Result = Result<(), E>>,
{
  #[inline]
  async fn apply_req_middlewares(&self, req: &mut Request<RRD>) -> Result<(), E> {
    (*self)(req).await?;
    Ok(())
  }
}

impl<E, RRD> ReqMiddlewares<E, RRD> for ()
where
  E: From<crate::Error>,
{
  #[inline]
  async fn apply_req_middlewares(&self, _: &mut Request<RRD>) -> Result<(), E> {
    Ok(())
  }
}

impl<E, RRD, T> ReqMiddlewares<E, RRD> for [T]
where
  E: From<crate::Error>,
  T: for<'any> FnFut1<&'any mut Request<RRD>, Result = Result<(), E>>,
{
  #[inline]
  async fn apply_req_middlewares(&self, req: &mut Request<RRD>) -> Result<(), E> {
    for elem in self {
      (elem)(req).await?;
    }
    Ok(())
  }
}

/// Responses middlewares
pub trait ResMiddlewares<E, RRD>
where
  E: From<crate::Error>,
{
  /// Modifies or halts responses.
  fn apply_res_middlewares(&self, _: Response<&mut RRD>) -> impl Future<Output = Result<(), E>>;
}

impl<E, RRD, T> ResMiddlewares<E, RRD> for &T
where
  E: From<crate::Error>,
  T: for<'req> FnFut1<Response<&'req mut RRD>, Result = Result<(), E>>,
{
  #[inline]
  async fn apply_res_middlewares(&self, res: Response<&mut RRD>) -> Result<(), E> {
    (*self)(res).await?;
    Ok(())
  }
}

impl<E, RRD> ResMiddlewares<E, RRD> for ()
where
  E: From<crate::Error>,
{
  #[inline]
  async fn apply_res_middlewares(&self, _: Response<&mut RRD>) -> Result<(), E> {
    Ok(())
  }
}

impl<E, RRD, T> ResMiddlewares<E, RRD> for [T]
where
  E: From<crate::Error>,
  T: for<'req, 'res> FnFut1<&'res mut Response<&'req mut RRD>, Result = Result<(), E>>,
{
  #[inline]
  async fn apply_res_middlewares(&self, mut res: Response<&mut RRD>) -> Result<(), E> {
    for elem in self {
      (elem)(&mut res).await?;
    }
    Ok(())
  }
}
