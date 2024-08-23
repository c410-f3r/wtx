use crate::{
  http::{Request, Response},
  misc::FnFut,
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
  T: for<'any> FnFut<&'any mut Request<RRD>, Result<(), E>>,
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
  T: for<'any> FnFut<&'any mut Request<RRD>, Result<(), E>>,
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
  fn apply_res_middlewares(&self, _: &mut Response<RRD>) -> impl Future<Output = Result<(), E>>;
}

impl<E, RRD, T> ResMiddlewares<E, RRD> for &T
where
  E: From<crate::Error>,
  T: for<'any> FnFut<&'any mut Response<RRD>, Result<(), E>>,
{
  #[inline]
  async fn apply_res_middlewares(&self, req: &mut Response<RRD>) -> Result<(), E> {
    (*self)(req).await?;
    Ok(())
  }
}

impl<E, RRD> ResMiddlewares<E, RRD> for ()
where
  E: From<crate::Error>,
{
  #[inline]
  async fn apply_res_middlewares(&self, _: &mut Response<RRD>) -> Result<(), E> {
    Ok(())
  }
}

impl<E, RRD, T> ResMiddlewares<E, RRD> for [T]
where
  E: From<crate::Error>,
  T: for<'any> FnFut<&'any mut Response<RRD>, Result<(), E>>,
{
  #[inline]
  async fn apply_res_middlewares(&self, req: &mut Response<RRD>) -> Result<(), E> {
    for elem in self {
      (elem)(req).await?;
    }
    Ok(())
  }
}
