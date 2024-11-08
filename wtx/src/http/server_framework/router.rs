use crate::{
  http::{
    server_framework::{Middleware, PathManagement},
    ReqResBuffer, Request, Response, StatusCode,
  },
  misc::{ArrayVector, Vector},
};
use core::{marker::PhantomData, ops::ControlFlow};

/// Redirects requests to specific asynchronous functions based on the set of inner URIs.
#[derive(Debug)]
pub struct Router<CA, E, M, P, SA> {
  pub(crate) middlewares: M,
  pub(crate) paths: P,
  pub(crate) phantom: PhantomData<(CA, E, SA)>,
  #[cfg(feature = "matchit")]
  pub(crate) router: matchit::Router<ArrayVector<(&'static str, u8), 8>>,
}

impl<CA, E, M, P, SA> Router<CA, E, M, P, SA>
where
  E: From<crate::Error>,
  P: PathManagement<CA, E, SA>,
{
  /// Creates a new instance with paths and middlewares.
  #[inline]
  pub fn new(paths: P, middlewares: M) -> crate::Result<Self> {
    #[cfg(feature = "matchit")]
    let router = Self::router(&paths)?;
    Ok(Self {
      middlewares,
      paths,
      phantom: PhantomData,
      #[cfg(feature = "matchit")]
      router,
    })
  }

  #[cfg(feature = "matchit")]
  fn router(paths: &P) -> crate::Result<matchit::Router<ArrayVector<(&'static str, u8), 8>>> {
    let mut vec = Vector::new();
    paths.paths_indices(ArrayVector::new(), &mut vec)?;
    let mut router = matchit::Router::new();
    for array in vec {
      let mut key = alloc::string::String::new();
      for elem in &array {
        key.push_str(elem.0);
      }
      router.insert(key, array)?;
    }
    Ok(router)
  }
}

impl<CA, E, P, SA> Router<CA, E, (), P, SA>
where
  E: From<crate::Error>,
  P: PathManagement<CA, E, SA>,
{
  /// Creates a new instance of empty middlewares.
  #[inline]
  pub fn paths(paths: P) -> crate::Result<Self> {
    #[cfg(feature = "matchit")]
    let router = Self::router(&paths)?;
    Ok(Self {
      middlewares: (),
      paths,
      phantom: PhantomData,
      #[cfg(feature = "matchit")]
      router,
    })
  }
}

impl<CA, E, M, P, SA> PathManagement<CA, E, SA> for Router<CA, E, M, P, SA>
where
  E: From<crate::Error>,
  M: Middleware<CA, E, SA>,
  P: PathManagement<CA, E, SA>,
{
  const IS_ROUTER: bool = true;

  #[inline]
  async fn manage_path(
    &self,
    conn_aux: &mut CA,
    path_defs: (u8, &[(&'static str, u8)]),
    req: &mut Request<ReqResBuffer>,
    stream_aux: &mut SA,
  ) -> Result<StatusCode, E> {
    let mw_aux = &mut self.middlewares.aux();
    if let ControlFlow::Break(el) = self.middlewares.req(conn_aux, mw_aux, req, stream_aux).await? {
      return Ok(el);
    }
    let status_code = self.paths.manage_path(conn_aux, path_defs, req, stream_aux).await?;
    let res = Response { rrd: &mut req.rrd, status_code, version: req.version };
    if let ControlFlow::Break(el) = self.middlewares.res(conn_aux, mw_aux, res, stream_aux).await? {
      return Ok(el);
    }
    Ok(status_code)
  }

  #[inline]
  fn paths_indices(
    &self,
    prev: ArrayVector<(&'static str, u8), 8>,
    vec: &mut Vector<ArrayVector<(&'static str, u8), 8>>,
  ) -> crate::Result<()> {
    self.paths.paths_indices(prev, vec)
  }
}
