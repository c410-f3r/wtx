use crate::http::{Request, Response};

/// Requests middlewares
pub trait ReqMiddlewares<E, RRD>
where
  E: From<crate::Error>,
{
  /// Modifies or halts requests.
  fn apply_req_middlewares(&self, _: &mut Request<RRD>) -> impl Future<Output = Result<(), E>>;
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

/// Responses middlewares
pub trait ResMiddlewares<E, RRD>
where
  E: From<crate::Error>,
{
  /// Modifies or halts responses.
  fn apply_res_middlewares(&self, _: &mut Response<RRD>) -> impl Future<Output = Result<(), E>>;
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
