use crate::http::{Request, StatusCode};
use core::future::Future;

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
    req: &mut Request<RRD>,
    req_path_indcs: [usize; 2],
  ) -> impl Future<Output = Result<StatusCode, E>>;
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
    req: &mut Request<RRD>,
    req_path_indcs: [usize; 2],
  ) -> Result<StatusCode, E> {
    (*self).manage_path(is_init, matching_path, req, req_path_indcs).await
  }
}
