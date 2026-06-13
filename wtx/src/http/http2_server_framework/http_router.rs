use crate::{
  collection::{ArrayVectorU8, Vector},
  http::{
    AutoStream, ManualStream, OperationMode, Response, Router, StatusCode,
    http2_server_framework::{Endpoint, EndpointNode, Middleware, RouteMatch},
  },
};
use core::{marker::PhantomData, ops::ControlFlow};

/// Redirects requests to specific asynchronous functions based on the set of inner URIs.
#[derive(Debug)]
pub struct HttpRouter<D, EN, ER, M, S> {
  pub(crate) en: EN,
  pub(crate) middlewares: M,
  pub(crate) phantom: PhantomData<(D, ER, S)>,
  pub(crate) router: Router<(ArrayVectorU8<RouteMatch, 4>, OperationMode)>,
}

impl<D, EN, ER, M, S> HttpRouter<D, EN, ER, M, S>
where
  EN: EndpointNode<D, ER, S>,
  ER: From<crate::Error>,
{
  /// Creates a new instance with generic paths and middlewares.
  #[inline]
  pub fn new(en: EN, middlewares: M) -> crate::Result<Self> {
    let router = Self::matcher(&en)?;
    Ok(Self { middlewares, en, phantom: PhantomData, router })
  }

  fn matcher(en: &EN) -> crate::Result<Router<(ArrayVectorU8<RouteMatch, 4>, OperationMode)>> {
    let mut vec = Vector::new();
    en.paths_indices(ArrayVectorU8::new(), &mut vec)?;
    let mut matcher = Router::new();
    let mut builder = matcher.builder();
    for array in vec {
      let [initials @ .., last] = array.as_slice() else {
        continue;
      };
      let mut key = alloc::string::String::new();
      for elem in initials {
        key.push_str(&elem.path);
      }
      key.push_str(&last.path);
      let om = last.om;
      let _ = builder.add(&key.try_into()?, (array, om))?;
    }
    drop(builder);
    Ok(matcher)
  }
}

impl<D, EN, ER, S> HttpRouter<D, EN, ER, (), S>
where
  EN: EndpointNode<D, ER, S>,
  ER: From<crate::Error>,
{
  /// Creates a new instance with automatic paths and middlewares.
  #[inline]
  pub fn paths(en: EN) -> crate::Result<Self> {
    let router = Self::matcher(&en)?;
    Ok(Self { en, middlewares: (), phantom: PhantomData, router })
  }
}

impl<D, EN, ER, M, S> Endpoint<D, ER, S> for HttpRouter<D, EN, ER, M, S>
where
  EN: EndpointNode<D, ER, S>,
  ER: From<crate::Error>,
  M: Middleware<D, ER>,
{
  const OM: OperationMode = OperationMode::Auto;

  #[inline]
  async fn auto(
    &self,
    auto_stream: &mut AutoStream<D>,
    path_defs: (u8, &[RouteMatch]),
  ) -> Result<StatusCode, ER> {
    let mw_aux = &mut self.middlewares.aux();
    if let ControlFlow::Break(el) =
      self.middlewares.req(&mut auto_stream.data, mw_aux, &mut auto_stream.req).await?
    {
      return Ok(el);
    }
    let status_code = self.en.auto(auto_stream, path_defs).await?;
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
    path_defs: (u8, &[RouteMatch]),
  ) -> Result<(), ER> {
    let mw_aux = &mut self.middlewares.aux();
    if let ControlFlow::Break(_) =
      self.middlewares.req(&mut manual_stream.data, mw_aux, &mut manual_stream.req).await?
    {
      return Ok(());
    }
    self.en.manual(manual_stream, path_defs).await?;
    Ok(())
  }
}

impl<D, EN, ER, M, S> EndpointNode<D, ER, S> for HttpRouter<D, EN, ER, M, S>
where
  EN: EndpointNode<D, ER, S>,
  ER: From<crate::Error>,
  M: Middleware<D, ER>,
{
  const IS_ROUTER: bool = true;

  #[inline]
  fn paths_indices(
    &self,
    prev: ArrayVectorU8<RouteMatch, 4>,
    vec: &mut Vector<ArrayVectorU8<RouteMatch, 4>>,
  ) -> crate::Result<()> {
    self.en.paths_indices(prev, vec)
  }
}

impl<D, EN, ER, M, S> Clone for HttpRouter<D, EN, ER, M, S>
where
  EN: Clone,
  M: Clone,
{
  fn clone(&self) -> Self {
    Self {
      en: self.en.clone(),
      middlewares: self.middlewares.clone(),
      phantom: self.phantom,
      router: self.router.clone(),
    }
  }
}
