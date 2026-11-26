use crate::{
  collections::{ArrayVectorCopy, VectorU16},
  http::{
    Method,
    http2_server_framework::{Endpoint, HttpRouterNode},
  },
};

/// Can be a terminal endpoint, a router, or a set of paths.
pub trait EndpointNode<D, ER, S>: Endpoint<D, ER, S>
where
  ER: From<crate::Error>,
{
  /// If the implementation is a router.
  const IS_ROUTER: bool;

  /// HTTP method required by this endpoint, or `None` if it accepts any method.
  fn method(&self) -> Option<Method>;

  /// Paths indices
  fn paths_indices(
    &self,
    buffer: &mut VectorU16<ArrayVectorCopy<HttpRouterNode, 4>>,
    prev: ArrayVectorCopy<HttpRouterNode, 4>,
  ) -> crate::Result<()>;
}

impl<D, ER, S, T> EndpointNode<D, ER, S> for &T
where
  ER: From<crate::Error>,
  T: EndpointNode<D, ER, S>,
{
  const IS_ROUTER: bool = T::IS_ROUTER;

  #[inline]
  fn method(&self) -> Option<Method> {
    T::method(*self)
  }

  #[inline]
  fn paths_indices(
    &self,
    buffer: &mut VectorU16<ArrayVectorCopy<HttpRouterNode, 4>>,
    prev: ArrayVectorCopy<HttpRouterNode, 4>,
  ) -> crate::Result<()> {
    (*self).paths_indices(buffer, prev)
  }
}

#[cfg(test)]
mod tests {
  use crate::{
    collections::{ArrayVectorCopy, VectorU16},
    http::{
      ManualStream, Method, OperationMode, StatusCode,
      http2_server_framework::{EndpointNode, HttpRouter, HttpRouterNode, StateClean, get},
    },
  };

  //     /a          /f/g    /i/j/k
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
      ("/a", HttpRouter::from_paths(paths!(("/b", get(auto)))).unwrap()),
      ("/a", HttpRouter::from_paths(paths!(("/c/d", get(auto)))).unwrap()),
      (
        "/a",
        HttpRouter::from_paths(paths!((
          "/d",
          HttpRouter::from_paths(paths!(("/e", get(auto)))).unwrap()
        )))
        .unwrap()
      ),
      ("/f/g", get(auto)),
      ("/f/g", HttpRouter::from_paths(paths!(("/h", get(auto)))).unwrap()),
      ("/i/j/k", get(manual)),
      (
        "/l",
        HttpRouter::from_paths(paths!(
          ("/m", get(auto)),
          ("/n", get(auto)),
          ("/o", HttpRouter::from_paths(paths!(("/p", get(auto)), ("/q", get(auto)))).unwrap())
        ))
        .unwrap()
      ),
    );
    let mut vec = VectorU16::new();
    paths.paths_indices(&mut vec, ArrayVectorCopy::new()).unwrap();
    assert_eq!(
      vec.as_slice(),
      &[
        ArrayVectorCopy::from_copyable_slice(&[HttpRouterNode::new(
          0,
          Some(Method::Get),
          OperationMode::Auto,
          "/a".try_into().unwrap()
        )])
        .unwrap(),
        ArrayVectorCopy::from_copyable_slice(&[
          HttpRouterNode::new(1, None, OperationMode::Auto, "/a".try_into().unwrap()),
          HttpRouterNode::new(0, Some(Method::Get), OperationMode::Auto, "/b".try_into().unwrap())
        ])
        .unwrap(),
        ArrayVectorCopy::from_copyable_slice(&[
          HttpRouterNode::new(2, None, OperationMode::Auto, "/a".try_into().unwrap()),
          HttpRouterNode::new(
            0,
            Some(Method::Get),
            OperationMode::Auto,
            "/c/d".try_into().unwrap()
          )
        ])
        .unwrap(),
        ArrayVectorCopy::from_copyable_slice(&[
          HttpRouterNode::new(3, None, OperationMode::Auto, "/a".try_into().unwrap()),
          HttpRouterNode::new(0, None, OperationMode::Auto, "/d".try_into().unwrap()),
          HttpRouterNode::new(0, Some(Method::Get), OperationMode::Auto, "/e".try_into().unwrap())
        ])
        .unwrap(),
        ArrayVectorCopy::from_copyable_slice(&[HttpRouterNode::new(
          4,
          Some(Method::Get),
          OperationMode::Auto,
          "/f/g".try_into().unwrap()
        )])
        .unwrap(),
        ArrayVectorCopy::from_copyable_slice(&[
          HttpRouterNode::new(5, None, OperationMode::Auto, "/f/g".try_into().unwrap()),
          HttpRouterNode::new(0, Some(Method::Get), OperationMode::Auto, "/h".try_into().unwrap())
        ])
        .unwrap(),
        ArrayVectorCopy::from_copyable_slice(&[HttpRouterNode::new(
          6,
          Some(Method::Get),
          OperationMode::Manual,
          "/i/j/k".try_into().unwrap()
        )])
        .unwrap(),
        ArrayVectorCopy::from_copyable_slice(&[
          HttpRouterNode::new(7, None, OperationMode::Auto, "/l".try_into().unwrap()),
          HttpRouterNode::new(0, Some(Method::Get), OperationMode::Auto, "/m".try_into().unwrap())
        ])
        .unwrap(),
        ArrayVectorCopy::from_copyable_slice(&[
          HttpRouterNode::new(7, None, OperationMode::Auto, "/l".try_into().unwrap()),
          HttpRouterNode::new(1, Some(Method::Get), OperationMode::Auto, "/n".try_into().unwrap())
        ])
        .unwrap(),
        ArrayVectorCopy::from_copyable_slice(&[
          HttpRouterNode::new(7, None, OperationMode::Auto, "/l".try_into().unwrap()),
          HttpRouterNode::new(2, None, OperationMode::Auto, "/o".try_into().unwrap()),
          HttpRouterNode::new(0, Some(Method::Get), OperationMode::Auto, "/p".try_into().unwrap())
        ])
        .unwrap(),
        ArrayVectorCopy::from_copyable_slice(&[
          HttpRouterNode::new(7, None, OperationMode::Auto, "/l".try_into().unwrap()),
          HttpRouterNode::new(2, None, OperationMode::Auto, "/o".try_into().unwrap()),
          HttpRouterNode::new(1, Some(Method::Get), OperationMode::Auto, "/q".try_into().unwrap())
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
