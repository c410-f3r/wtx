use crate::{
  http::{
    server_framework::{PathManagement, ReqMiddleware, ResMiddleware},
    ReqResData, Request, Response, StatusCode,
  },
  misc::{ArrayVector, Vector},
};
use alloc::string::String;
use core::marker::PhantomData;

/// Redirects requests to specific asynchronous functions based on the set of inner URIs.
#[derive(Debug)]
pub struct Router<CA, E, P, RA, REQM, RESM, RRD> {
  pub(crate) paths: P,
  pub(crate) phantom: PhantomData<(CA, E, RA, RRD)>,
  pub(crate) req_middlewares: REQM,
  pub(crate) res_middlewares: RESM,
  pub(crate) router: matchit::Router<ArrayVector<(&'static str, u8), 8>>,
}

impl<CA, E, P, RA, REQM, RESM, RRD> Router<CA, E, P, RA, REQM, RESM, RRD>
where
  E: From<crate::Error>,
  P: PathManagement<CA, E, RA, RRD>,
{
  /// Creates a new instance with paths and middlewares.
  #[inline]
  pub fn new(paths: P, req_middlewares: REQM, res_middlewares: RESM) -> crate::Result<Self> {
    let router = Self::router(&paths)?;
    Ok(Self { paths, phantom: PhantomData, req_middlewares, res_middlewares, router })
  }

  fn router(paths: &P) -> crate::Result<matchit::Router<ArrayVector<(&'static str, u8), 8>>> {
    let mut vec = Vector::new();
    paths.paths_indices(ArrayVector::new(), &mut vec)?;
    let mut router = matchit::Router::new();
    for array in vec {
      let mut key = String::new();
      for elem in &array {
        key.push_str(elem.0);
      }
      router.insert(key, array)?;
    }
    Ok(router)
  }
}

impl<CA, E, P, RA, RRD> Router<CA, E, P, RA, (), (), RRD>
where
  E: From<crate::Error>,
  P: PathManagement<CA, E, RA, RRD>,
{
  /// Creates a new instance of empty middlewares.
  #[inline]
  pub fn paths(paths: P) -> crate::Result<Self> {
    let router = Self::router(&paths)?;
    Ok(Self { paths, phantom: PhantomData, req_middlewares: (), res_middlewares: (), router })
  }
}

impl<CA, E, P, RA, REQM, RESM, RRD> PathManagement<CA, E, RA, RRD>
  for Router<CA, E, P, RA, REQM, RESM, RRD>
where
  E: From<crate::Error>,
  P: PathManagement<CA, E, RA, RRD>,
  REQM: ReqMiddleware<CA, E, RA, RRD>,
  RESM: ResMiddleware<CA, E, RA, RRD>,
  RRD: ReqResData,
{
  const IS_ROUTER: bool = true;

  #[inline]
  async fn manage_path(
    &self,
    ca: &mut CA,
    path_defs: (u8, &[(&'static str, u8)]),
    ra: &mut RA,
    req: &mut Request<RRD>,
  ) -> Result<StatusCode, E> {
    self.req_middlewares.apply_req_middleware(ca, ra, req).await?;
    let status_code = self.paths.manage_path(ca, path_defs, ra, req).await?;
    let res = Response { rrd: &mut req.rrd, status_code, version: req.version };
    self.res_middlewares.apply_res_middleware(ca, ra, res).await?;
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
