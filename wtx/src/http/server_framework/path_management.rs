use crate::{
  http::{ReqResBuffer, Request, StatusCode},
  misc::{ArrayVector, Vector},
};
use core::future::Future;

/// Used by all structures that somehow interact with incoming requests.
pub trait PathManagement<CA, E, SA>
where
  E: From<crate::Error>,
{
  /// If the implementation is a router.
  const IS_ROUTER: bool;

  /// Creates a response based on a request.
  fn manage_path(
    &self,
    conn_aux: &mut CA,
    path_defs: (u8, &[(&'static str, u8)]),
    req: &mut Request<ReqResBuffer>,
    stream_aux: &mut SA,
  ) -> impl Future<Output = Result<StatusCode, E>>;

  /// Used internally to fill `vec` with the indices of all nested routes.
  ///
  /// ```txt
  /// 0     1     2
  /// |\    |    /|\
  /// | \   |   / | \
  /// 0  1  0  0  1  2
  ///                |
  ///                |
  ///                0
  /// ```
  fn paths_indices(
    &self,
    prev: ArrayVector<(&'static str, u8), 8>,
    vec: &mut Vector<ArrayVector<(&'static str, u8), 8>>,
  ) -> crate::Result<()>;
}

impl<CA, E, SA, T> PathManagement<CA, E, SA> for &T
where
  E: From<crate::Error>,
  T: PathManagement<CA, E, SA>,
{
  const IS_ROUTER: bool = T::IS_ROUTER;

  #[inline]
  async fn manage_path(
    &self,
    conn_aux: &mut CA,
    path_defs: (u8, &[(&'static str, u8)]),
    req: &mut Request<ReqResBuffer>,
    stream_aux: &mut SA,
  ) -> Result<StatusCode, E> {
    (*self).manage_path(conn_aux, path_defs, req, stream_aux).await
  }

  #[inline]
  fn paths_indices(
    &self,
    prev: ArrayVector<(&'static str, u8), 8>,
    vec: &mut Vector<ArrayVector<(&'static str, u8), 8>>,
  ) -> crate::Result<()> {
    (*self).paths_indices(prev, vec)
  }
}

#[cfg(all(feature = "_async-tests", test))]
mod tests {
  use crate::{
    http::{
      server_framework::{get, PathManagement, Router, StateClean},
      ReqResBuffer, StatusCode,
    },
    misc::{ArrayVector, Vector},
  };

  //     /a          /f/g          /i/j/k
  //   /  |  \       |
  //  /   |   \      |
  // /b  /c/d  /d    /h
  //           |
  //           |
  //           /e
  #[tokio::test]
  async fn paths_indices() {
    let paths = paths!(
      ("/a", get(endpoint)),
      ("/a", Router::paths(paths!(("/b", get(endpoint)))).unwrap()),
      ("/a", Router::paths(paths!(("/c/d", get(endpoint)))).unwrap()),
      (
        "/a",
        Router::paths(paths!(("/d", Router::paths(paths!(("/e", get(endpoint)))).unwrap())))
          .unwrap()
      ),
      ("/f/g", get(endpoint)),
      ("/f/g", Router::paths(paths!(("/h", get(endpoint)))).unwrap()),
      ("/i/j/k", get(endpoint)),
      (
        "/l",
        Router::paths(paths!(
          ("/m", get(endpoint)),
          ("/n", get(endpoint)),
          ("/o", Router::paths(paths!(("/p", get(endpoint)), ("/q", get(endpoint)))).unwrap())
        ))
        .unwrap()
      ),
    );
    let mut vec = Vector::new();
    paths.paths_indices(ArrayVector::new(), &mut vec).unwrap();
    assert_eq!(
      vec.as_slice(),
      &[
        ArrayVector::from_copyable_slice(&[("/a", 0)]).unwrap(),
        ArrayVector::from_copyable_slice(&[("/a", 1), ("/b", 0)]).unwrap(),
        ArrayVector::from_copyable_slice(&[("/a", 2), ("/c/d", 0)]).unwrap(),
        ArrayVector::from_copyable_slice(&[("/a", 3), ("/d", 0), ("/e", 0)]).unwrap(),
        ArrayVector::from_copyable_slice(&[("/f/g", 4)]).unwrap(),
        ArrayVector::from_copyable_slice(&[("/f/g", 5), ("/h", 0)]).unwrap(),
        ArrayVector::from_copyable_slice(&[("/i/j/k", 6)]).unwrap(),
        ArrayVector::from_copyable_slice(&[("/l", 7), ("/m", 0)]).unwrap(),
        ArrayVector::from_copyable_slice(&[("/l", 7), ("/n", 1)]).unwrap(),
        ArrayVector::from_copyable_slice(&[("/l", 7), ("/o", 2), ("/p", 0)]).unwrap(),
        ArrayVector::from_copyable_slice(&[("/l", 7), ("/o", 2), ("/q", 1)]).unwrap(),
      ]
    );
  }

  async fn endpoint(_: StateClean<'_, (), (), ReqResBuffer>) -> crate::Result<StatusCode> {
    Ok(StatusCode::Ok)
  }
}
