use crate::{
  collections::ShortStrU8,
  http::{Method, OperationMode, http2_server_framework::Http2ServerFrameworkError},
};

/// An endpoint
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HttpRouterNode {
  pub(crate) idx: u8,
  pub(crate) method: Option<Method>,
  pub(crate) om: OperationMode,
  pub(crate) path: ShortStrU8<'static>,
}

impl HttpRouterNode {
  pub(crate) fn find_by_method<'el>(
    iter: impl IntoIterator<Item = &'el HttpRouterNode>,
    method: Method,
  ) -> crate::Result<&'el HttpRouterNode> {
    iter
      .into_iter()
      .find(|el| el.method == Some(method))
      .ok_or(Http2ServerFrameworkError::MethodNotFoundInPath.into())
  }

  pub(crate) const fn new(
    idx: u8,
    method: Option<Method>,
    om: OperationMode,
    path: ShortStrU8<'static>,
  ) -> Self {
    Self { idx, method, om, path }
  }
}
