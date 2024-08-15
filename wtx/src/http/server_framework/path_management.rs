use crate::http::{server_framework::Path, HttpError, ReqResData, Request, Response};

/// Used by all structures that somehow interact with incoming requests.
pub trait PathManagement<E, RRD>
where
  E: From<crate::Error>,
{
  /// Creates a response based on a request.
  fn manage_path(
    &self,
    is_init: bool,
    matching_path: &'static str,
    req: Request<RRD>,
    req_path_indcs: [usize; 2],
  ) -> impl Future<Output = Result<Response<RRD>, E>>;
}

impl<E, RRD, T> PathManagement<E, RRD> for &T
where
  E: From<crate::Error>,
  T: PathManagement<E, RRD>,
{
  #[inline]
  async fn manage_path(
    &self,
    is_init: bool,
    matching_path: &'static str,
    req: Request<RRD>,
    req_path_indcs: [usize; 2],
  ) -> Result<Response<RRD>, E> {
    (*self).manage_path(is_init, matching_path, req, req_path_indcs).await
  }
}

impl<E, RRD, T> PathManagement<E, RRD> for [Path<T>]
where
  E: From<crate::Error>,
  RRD: ReqResData,
  T: PathManagement<E, RRD>,
{
  #[inline]
  async fn manage_path(
    &self,
    is_init: bool,
    matching_path: &'static str,
    req: Request<RRD>,
    [begin, end]: [usize; 2],
  ) -> Result<Response<RRD>, E> {
    let req_uri = req.rrd.uri();
    let req_path = req_uri.as_str().get(begin..end).unwrap_or_default();
    for elem in self {
      if elem.name.starts_with(req_path) {
        return elem.value.manage_path(is_init, matching_path, req, [begin, end]).await;
      }
    }
    Err(E::from(HttpError::UriMismatch.into()))
  }
}
