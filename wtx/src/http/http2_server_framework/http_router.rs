use crate::{
  collections::{ArrayVectorCopy, DynVectorU7, ShortBoxStrU8, VectorU16},
  http::{
    AutoStream, ManualStream, Method, OperationMode, Response, Router, StatusCode,
    http2_server_framework::{
      Endpoint, EndpointNode, Http2ServerFrameworkError, HttpRouterChain, HttpRouterNode,
      Middleware,
    },
  },
};
use core::{
  fmt::{Debug, Formatter},
  marker::PhantomData,
  ops::ControlFlow,
};

/// Redirects requests to specific asynchronous functions based on the set of inner URIs.
pub struct HttpRouter<D, EN, ER, M, S> {
  endpoint_node: EN,
  middlewares: M,
  phantom: PhantomData<fn() -> (D, ER, S)>,
  router: Router<HttpRouterChain>,
}

impl<D, ER, S> HttpRouter<D, (), ER, (), S> {
  /// Creates an empty router.
  #[inline]
  pub const fn empty() -> Self {
    Self { endpoint_node: (), middlewares: (), phantom: PhantomData, router: Router::new() }
  }
}

impl<D, EN, ER, S> HttpRouter<D, EN, ER, (), S>
where
  EN: EndpointNode<D, ER, S>,
  ER: From<crate::Error>,
{
  /// Creates a new instance with automatic paths and no middlewares.
  #[inline]
  pub fn from_paths(endpoint_node: EN) -> crate::Result<Self> {
    HttpRouter::new(endpoint_node, ())
  }
}

impl<D, EN, ER, M, S> HttpRouter<D, EN, ER, M, S> {
  pub(crate) fn router(&self) -> &Router<HttpRouterChain> {
    &self.router
  }
}

impl<D, EN, ER, M, S> HttpRouter<D, EN, ER, M, S>
where
  EN: EndpointNode<D, ER, S>,
  ER: From<crate::Error>,
{
  /// Creates a new instance with explicit middlewares.
  #[inline]
  pub fn new(endpoint_node: EN, middlewares: M) -> crate::Result<Self> {
    let router = Self::matcher(&endpoint_node)?;
    Ok(HttpRouter { endpoint_node, middlewares, phantom: PhantomData, router })
  }

  // Only called once at start-up, as such, all temporary inner allocations and scans problably
  // aren't a problem.
  fn matcher(endpoint_node: &EN) -> crate::Result<Router<HttpRouterChain>> {
    let mut paths_indices = VectorU16::new();
    endpoint_node.paths_indices(&mut paths_indices, ArrayVectorCopy::new())?;
    let mut groups = VectorU16::<(ShortBoxStrU8, HttpRouterChain)>::new();
    for array in paths_indices {
      let [initials @ .., last] = array.as_slice() else {
        continue;
      };
      let mut key_string = alloc::string::String::new();
      for elem in initials {
        key_string.push_str(&elem.path);
      }
      key_string.push_str(&last.path);
      let key_box_str: ShortBoxStrU8 = key_string.try_into()?;
      if let Some((_, hrn)) = groups.iter_mut().find(|(elem, _)| *elem == key_box_str) {
        if hrn.steps().as_slice() != initials {
          return Err(Http2ServerFrameworkError::PathWithConflictingNesting.into());
        }
        if hrn.leafs().iter().any(|el| el.method == last.method) {
          return Err(Http2ServerFrameworkError::PathWithDuplicatedMethod.into());
        }
        hrn.leafs_mut().push(*last)?;
      } else {
        let steps = ArrayVectorCopy::from_copyable_slice(initials)?;
        let mut leafs = DynVectorU7::new();
        leafs.push(*last)?;
        groups.push((key_box_str, HttpRouterChain::new(leafs, steps)))?;
      }
    }
    let mut matcher = Router::new();
    let mut builder = matcher.builder();
    for (key, hrn) in groups {
      let _ = builder.add(&key, hrn)?;
    }
    drop(builder);
    Ok(matcher)
  }
}

impl<D, EN, ER, M, S> Endpoint<D, ER, S> for HttpRouter<D, EN, ER, M, S>
where
  M: Middleware<D, ER>,
  EN: EndpointNode<D, ER, S>,
  ER: From<crate::Error>,
{
  const OM: OperationMode = OperationMode::Auto;

  #[inline]
  async fn auto(
    &self,
    auto_stream: &mut AutoStream<D>,
    path_leafs: (u8, &[HttpRouterNode]),
  ) -> Result<StatusCode, ER> {
    let mw_aux = &mut self.middlewares.aux();
    let cf = self.middlewares.req(&mut auto_stream.data, mw_aux, &mut auto_stream.req).await?;
    let status_code = if let ControlFlow::Break(el) = cf {
      el
    } else {
      self.endpoint_node.auto(auto_stream, path_leafs).await?
    };
    if let ControlFlow::Break(el) = self
      .middlewares
      .res(
        &mut auto_stream.data,
        mw_aux,
        Response { msg_data: &mut auto_stream.req.msg_data, status_code },
      )
      .await?
    {
      return Ok(el);
    }
    Ok(status_code)
  }

  #[inline]
  async fn manual(
    &self,
    mut manual_stream: ManualStream<D, S>,
    path_leafs: (u8, &[HttpRouterNode]),
  ) -> Result<(), ER> {
    let mw_aux = &mut self.middlewares.aux();
    if let ControlFlow::Break(_) =
      self.middlewares.req(&mut manual_stream.data, mw_aux, &mut manual_stream.req).await?
    {
      return Ok(());
    }
    self.endpoint_node.manual(manual_stream, path_leafs).await
  }
}

impl<D, EN, ER, M, S> EndpointNode<D, ER, S> for HttpRouter<D, EN, ER, M, S>
where
  M: Middleware<D, ER>,
  EN: EndpointNode<D, ER, S>,
  ER: From<crate::Error>,
{
  const IS_ROUTER: bool = true;

  #[inline]
  fn method(&self) -> Option<Method> {
    None
  }

  #[inline]
  fn paths_indices(
    &self,
    buffer: &mut VectorU16<ArrayVectorCopy<HttpRouterNode, 4>>,
    prev: ArrayVectorCopy<HttpRouterNode, 4>,
  ) -> crate::Result<()> {
    self.endpoint_node.paths_indices(buffer, prev)
  }
}

impl<D, EN, ER, M, S> Clone for HttpRouter<D, EN, ER, M, S>
where
  EN: Clone,
  M: Clone,
{
  #[inline]
  fn clone(&self) -> Self {
    Self {
      endpoint_node: self.endpoint_node.clone(),
      middlewares: self.middlewares.clone(),
      phantom: self.phantom,
      router: self.router.clone(),
    }
  }
}

impl<D, EN, ER, M, S> Debug for HttpRouter<D, EN, ER, M, S>
where
  EN: Debug,
  M: Debug,
{
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    f.debug_struct("HttpRouter")
      .field("endpoint_node", &self.endpoint_node)
      .field("middlewares", &self.middlewares)
      .field("router", &self.router)
      .finish()
  }
}
