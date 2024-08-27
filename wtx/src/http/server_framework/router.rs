use crate::{
  http::{
    server_framework::{PathManagement, ReqMiddlewares, ResMiddlewares},
    HttpError, ReqResData, Request, Response, StatusCode,
  },
  misc::str_pos1,
};

/// Redirects requests to specific asynchronous functions based on the set of inner URIs.
#[derive(Debug)]
pub struct Router<P, REQM, RESM> {
  pub(crate) paths: P,
  pub(crate) req_middlewares: REQM,
  pub(crate) res_middlewares: RESM,
}

impl<P, REQM, RESM> Router<P, REQM, RESM> {
  /// Creates a new instance with paths and middlewares.
  #[inline]
  pub fn new(paths: P, req_middlewares: REQM, res_middlewares: RESM) -> Self {
    Self { paths, req_middlewares, res_middlewares }
  }
}

impl<P> Router<P, (), ()> {
  /// Creates a new instance of empty middlewares.
  #[inline]
  pub fn paths(paths: P) -> Self {
    Self { paths, req_middlewares: (), res_middlewares: () }
  }
}

impl<E, P, REQM, RESM, RRD> PathManagement<E, RRD> for Router<P, REQM, RESM>
where
  E: From<crate::Error>,
  P: PathManagement<E, RRD>,
  REQM: ReqMiddlewares<E, RRD>,
  RESM: ResMiddlewares<E, RRD>,
  RRD: ReqResData,
{
  #[inline]
  async fn manage_path(
    &self,
    is_init: bool,
    _: &'static str,
    req: &mut Request<RRD>,
    [_begin, end]: [usize; 2],
  ) -> Result<StatusCode, E> {
    let uri = req.rrd.uri();
    let uri_str = uri.as_str();
    let (local_begin, rest) = if is_init {
      let begin = uri_str
        .split_once("://")
        .and_then(|(before, after)| {
          let idx = str_pos1(after, b'/')?;
          Some(before.len().wrapping_add(3).wrapping_add(idx))
        })
        .unwrap_or(uri_str.len());
      (begin, uri_str.get(begin.wrapping_add(1)..).unwrap_or_default())
    } else {
      (
        end.wrapping_add(1),
        match uri_str.get(end.wrapping_add(2)..) {
          Some(elem) if !elem.is_empty() => elem,
          _ => return Err(E::from(HttpError::UriMismatch.into())),
        },
      )
    };
    let local_end = str_pos1(rest, b'/')
      .map_or_else(|| uri_str.len(), |element| element.wrapping_add(local_begin));
    self.req_middlewares.apply_req_middlewares(req).await?;
    let req_path_indcs = [local_begin, local_end];
    let status_code = self.paths.manage_path(false, "", req, req_path_indcs).await?;
    let res = Response { rrd: &mut req.rrd, status_code, version: req.version };
    self.res_middlewares.apply_res_middlewares(res).await?;
    Ok(status_code)
  }
}
