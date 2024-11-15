use crate::{
  http::server_framework::{Endpoint, RouteMatch},
  misc::{ArrayVector, Vector},
};

/// Can be a terminal endpoint, a router, or a set of paths.
pub trait EndpointNode<CA, E, S, SA>: Endpoint<CA, E, S, SA>
where
  E: From<crate::Error>,
{
  /// If the implementation is a router.
  const IS_ROUTER: bool;

  /// Paths indices
  fn paths_indices(
    &self,
    prev: ArrayVector<RouteMatch, 4>,
    vec: &mut Vector<ArrayVector<RouteMatch, 4>>,
  ) -> crate::Result<()>;
}

impl<CA, E, S, SA, T> EndpointNode<CA, E, S, SA> for &T
where
  E: From<crate::Error>,
  T: EndpointNode<CA, E, S, SA>,
{
  const IS_ROUTER: bool = T::IS_ROUTER;

  #[inline]
  fn paths_indices(
    &self,
    prev: ArrayVector<RouteMatch, 4>,
    vec: &mut Vector<ArrayVector<RouteMatch, 4>>,
  ) -> crate::Result<()> {
    (*self).paths_indices(prev, vec)
  }
}

#[cfg(all(feature = "_async-tests", test))]
mod tests {
  use crate::{
    http::{
      server_framework::{get, EndpointNode, RouteMatch, Router, StateClean},
      ManualStream, OperationMode, ReqResBuffer, StatusCode,
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
      ("/a", get(auto)),
      ("/a", Router::paths(paths!(("/b", get(auto)))).unwrap()),
      ("/a", Router::paths(paths!(("/c/d", get(auto)))).unwrap()),
      (
        "/a",
        Router::paths(paths!(("/d", Router::paths(paths!(("/e", get(auto)))).unwrap()))).unwrap()
      ),
      ("/f/g", get(auto)),
      ("/f/g", Router::paths(paths!(("/h", get(auto)))).unwrap()),
      ("/i/j/k", get(manual)),
      (
        "/l",
        Router::paths(paths!(
          ("/m", get(auto)),
          ("/n", get(auto)),
          ("/o", Router::paths(paths!(("/p", get(auto)), ("/q", get(auto)))).unwrap())
        ))
        .unwrap()
      ),
    );
    let mut vec = Vector::new();
    paths.paths_indices(ArrayVector::new(), &mut vec).unwrap();
    assert_eq!(
      vec.as_slice(),
      &[
        ArrayVector::from_copyable_slice(&[RouteMatch::new(0, OperationMode::Auto, "/a")]).unwrap(),
        ArrayVector::from_copyable_slice(&[
          RouteMatch::new(1, OperationMode::Auto, "/a"),
          RouteMatch::new(0, OperationMode::Auto, "/b")
        ])
        .unwrap(),
        ArrayVector::from_copyable_slice(&[
          RouteMatch::new(2, OperationMode::Auto, "/a"),
          RouteMatch::new(0, OperationMode::Auto, "/c/d")
        ])
        .unwrap(),
        ArrayVector::from_copyable_slice(&[
          RouteMatch::new(3, OperationMode::Auto, "/a"),
          RouteMatch::new(0, OperationMode::Auto, "/d"),
          RouteMatch::new(0, OperationMode::Auto, "/e")
        ])
        .unwrap(),
        ArrayVector::from_copyable_slice(&[RouteMatch::new(4, OperationMode::Auto, "/f/g")])
          .unwrap(),
        ArrayVector::from_copyable_slice(&[
          RouteMatch::new(5, OperationMode::Auto, "/f/g"),
          RouteMatch::new(0, OperationMode::Auto, "/h")
        ])
        .unwrap(),
        ArrayVector::from_copyable_slice(&[RouteMatch::new(6, OperationMode::Manual, "/i/j/k")])
          .unwrap(),
        ArrayVector::from_copyable_slice(&[
          RouteMatch::new(7, OperationMode::Auto, "/l"),
          RouteMatch::new(0, OperationMode::Auto, "/m")
        ])
        .unwrap(),
        ArrayVector::from_copyable_slice(&[
          RouteMatch::new(7, OperationMode::Auto, "/l"),
          RouteMatch::new(1, OperationMode::Auto, "/n")
        ])
        .unwrap(),
        ArrayVector::from_copyable_slice(&[
          RouteMatch::new(7, OperationMode::Auto, "/l"),
          RouteMatch::new(2, OperationMode::Auto, "/o"),
          RouteMatch::new(0, OperationMode::Auto, "/p")
        ])
        .unwrap(),
        ArrayVector::from_copyable_slice(&[
          RouteMatch::new(7, OperationMode::Auto, "/l"),
          RouteMatch::new(2, OperationMode::Auto, "/o"),
          RouteMatch::new(1, OperationMode::Auto, "/q")
        ])
        .unwrap(),
      ]
    );
  }

  async fn auto(_: StateClean<'_, (), (), ReqResBuffer>) -> crate::Result<StatusCode> {
    Ok(StatusCode::Ok)
  }

  async fn manual(_: ManualStream<(), (), ()>) -> crate::Result<()> {
    Ok(())
  }
}
