use crate::{
  collections::{ArrayVectorCopy, Vector},
  http::http2_server_framework::{Endpoint, RouteMatch},
};

/// Can be a terminal endpoint, a router, or a set of paths.
pub trait EndpointNode<D, ER, S>: Endpoint<D, ER, S>
where
  ER: From<crate::Error>,
{
  /// If the implementation is a router.
  const IS_ROUTER: bool;

  /// Paths indices
  fn paths_indices(
    &self,
    prev: ArrayVectorCopy<RouteMatch, 4>,
    vec: &mut Vector<ArrayVectorCopy<RouteMatch, 4>>,
  ) -> crate::Result<()>;
}

impl<D, ER, S, T> EndpointNode<D, ER, S> for &T
where
  ER: From<crate::Error>,
  T: EndpointNode<D, ER, S>,
{
  const IS_ROUTER: bool = T::IS_ROUTER;

  #[inline]
  fn paths_indices(
    &self,
    prev: ArrayVectorCopy<RouteMatch, 4>,
    vec: &mut Vector<ArrayVectorCopy<RouteMatch, 4>>,
  ) -> crate::Result<()> {
    (*self).paths_indices(prev, vec)
  }
}

#[cfg(test)]
mod tests {
  use crate::{
    collections::{ArrayVectorCopy, Vector},
    http::{
      ManualStream, OperationMode, StatusCode,
      http2_server_framework::{EndpointNode, HttpRouter, RouteMatch, StateClean, get},
    },
  };

  //     /a          /f/g          /i/j/k
  //   /  |  \       |
  //  /   |   \      |
  // /b  /c/d  /d    /h
  //           |
  //           |
  //           /e
  #[test]
  fn paths_indices() {
    let paths = paths!(
      ("/a", get(auto)),
      ("/a", HttpRouter::paths(paths!(("/b", get(auto)))).unwrap()),
      ("/a", HttpRouter::paths(paths!(("/c/d", get(auto)))).unwrap()),
      (
        "/a",
        HttpRouter::paths(paths!(("/d", HttpRouter::paths(paths!(("/e", get(auto)))).unwrap())))
          .unwrap()
      ),
      ("/f/g", get(auto)),
      ("/f/g", HttpRouter::paths(paths!(("/h", get(auto)))).unwrap()),
      ("/i/j/k", get(manual)),
      (
        "/l",
        HttpRouter::paths(paths!(
          ("/m", get(auto)),
          ("/n", get(auto)),
          ("/o", HttpRouter::paths(paths!(("/p", get(auto)), ("/q", get(auto)))).unwrap())
        ))
        .unwrap()
      ),
    );
    let mut vec = Vector::new();
    paths.paths_indices(ArrayVectorCopy::new(), &mut vec).unwrap();
    assert_eq!(
      vec.as_slice(),
      &[
        ArrayVectorCopy::from_copyable_slice(&[RouteMatch::new(
          0,
          OperationMode::Auto,
          "/a".try_into().unwrap()
        )])
        .unwrap(),
        ArrayVectorCopy::from_copyable_slice(&[
          RouteMatch::new(1, OperationMode::Auto, "/a".try_into().unwrap()),
          RouteMatch::new(0, OperationMode::Auto, "/b".try_into().unwrap())
        ])
        .unwrap(),
        ArrayVectorCopy::from_copyable_slice(&[
          RouteMatch::new(2, OperationMode::Auto, "/a".try_into().unwrap()),
          RouteMatch::new(0, OperationMode::Auto, "/c/d".try_into().unwrap())
        ])
        .unwrap(),
        ArrayVectorCopy::from_copyable_slice(&[
          RouteMatch::new(3, OperationMode::Auto, "/a".try_into().unwrap()),
          RouteMatch::new(0, OperationMode::Auto, "/d".try_into().unwrap()),
          RouteMatch::new(0, OperationMode::Auto, "/e".try_into().unwrap())
        ])
        .unwrap(),
        ArrayVectorCopy::from_copyable_slice(&[RouteMatch::new(
          4,
          OperationMode::Auto,
          "/f/g".try_into().unwrap()
        )])
        .unwrap(),
        ArrayVectorCopy::from_copyable_slice(&[
          RouteMatch::new(5, OperationMode::Auto, "/f/g".try_into().unwrap()),
          RouteMatch::new(0, OperationMode::Auto, "/h".try_into().unwrap())
        ])
        .unwrap(),
        ArrayVectorCopy::from_copyable_slice(&[RouteMatch::new(
          6,
          OperationMode::Manual,
          "/i/j/k".try_into().unwrap()
        )])
        .unwrap(),
        ArrayVectorCopy::from_copyable_slice(&[
          RouteMatch::new(7, OperationMode::Auto, "/l".try_into().unwrap()),
          RouteMatch::new(0, OperationMode::Auto, "/m".try_into().unwrap())
        ])
        .unwrap(),
        ArrayVectorCopy::from_copyable_slice(&[
          RouteMatch::new(7, OperationMode::Auto, "/l".try_into().unwrap()),
          RouteMatch::new(1, OperationMode::Auto, "/n".try_into().unwrap())
        ])
        .unwrap(),
        ArrayVectorCopy::from_copyable_slice(&[
          RouteMatch::new(7, OperationMode::Auto, "/l".try_into().unwrap()),
          RouteMatch::new(2, OperationMode::Auto, "/o".try_into().unwrap()),
          RouteMatch::new(0, OperationMode::Auto, "/p".try_into().unwrap())
        ])
        .unwrap(),
        ArrayVectorCopy::from_copyable_slice(&[
          RouteMatch::new(7, OperationMode::Auto, "/l".try_into().unwrap()),
          RouteMatch::new(2, OperationMode::Auto, "/o".try_into().unwrap()),
          RouteMatch::new(1, OperationMode::Auto, "/q".try_into().unwrap())
        ])
        .unwrap(),
      ]
    );
  }

  async fn auto(_: StateClean<'_, ()>) -> crate::Result<StatusCode> {
    Ok(StatusCode::Ok)
  }

  async fn manual(_: ManualStream<(), ()>) -> crate::Result<()> {
    Ok(())
  }
}
