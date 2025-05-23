use crate::{
  collection::{ArrayVector, Vector},
  http::{
    AutoStream, Headers, HttpError, KnownHeaderName, ManualStream, Method, Mime, OperationMode,
    StatusCode,
    server_framework::{Endpoint, EndpointNode, RouteMatch},
  },
  misc::FnFut,
};

/// Requires a JSON request as well as an associated method.
#[derive(Debug)]
pub struct Json<T>(
  /// Function
  pub T,
  /// Required method
  pub Method,
);

/// Creates a new [`Json`] instance.
#[inline]
pub fn json<A, T>(method: Method, ty: T) -> Json<T::Wrapper>
where
  T: FnFut<A>,
{
  Json(ty.into_wrapper(), method)
}

impl<CA, E, S, SA, T> Endpoint<CA, E, S, SA> for Json<T>
where
  E: From<crate::Error>,
  T: Endpoint<CA, E, S, SA>,
{
  const OM: OperationMode = T::OM;

  #[inline]
  async fn auto(
    &self,
    auto_stream: &mut AutoStream<CA, SA>,
    path_defs: (u8, &[RouteMatch]),
  ) -> Result<StatusCode, E> {
    check_json(&auto_stream.req.rrd.headers, auto_stream.req.method, self.1)?;
    self.0.auto(auto_stream, path_defs).await
  }

  #[inline]
  async fn manual(
    &self,
    manual_stream: ManualStream<CA, S, SA>,
    path_defs: (u8, &[RouteMatch]),
  ) -> Result<(), E> {
    check_json(&manual_stream.req.rrd.headers, manual_stream.req.method, self.1)?;
    self.0.manual(manual_stream, path_defs).await
  }
}

impl<CA, E, S, SA, T> EndpointNode<CA, E, S, SA> for Json<T>
where
  E: From<crate::Error>,
  T: Endpoint<CA, E, S, SA>,
{
  const IS_ROUTER: bool = false;

  #[inline]
  fn paths_indices(
    &self,
    _: ArrayVector<RouteMatch, 4>,
    _: &mut Vector<ArrayVector<RouteMatch, 4>>,
  ) -> crate::Result<()> {
    Ok(())
  }
}

fn check_json<E>(req_headers: &Headers, req_method: Method, user_method: Method) -> Result<(), E>
where
  E: From<crate::Error>,
{
  let header = req_headers
    .get_by_name(KnownHeaderName::ContentType.into())
    .ok_or(crate::Error::from(HttpError::MissingHeader(KnownHeaderName::ContentType)))?;
  if !header.value.starts_with(Mime::ApplicationJson.as_str()) {
    return Err(E::from(crate::Error::from(HttpError::UnexpectedContentType)));
  }
  if req_method != user_method {
    return Err(E::from(crate::Error::from(HttpError::UnexpectedHttpMethod {
      expected: user_method,
    })));
  }
  Ok(())
}
