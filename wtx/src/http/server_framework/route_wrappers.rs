use crate::{
  http::{
    server_framework::{Endpoint, PathManagement},
    HttpError, KnownHeaderName, Method, Mime, ReqResBuffer, ReqResData, Request, StatusCode,
  },
  misc::{ArrayVector, FnFut, Vector},
};

/// Requires a request of type `GET`.
#[derive(Debug)]
pub struct Get<T>(
  /// Arbitrary type
  pub T,
);

/// Creates a new [`Get`] instance.
#[inline]
pub fn get<A, T>(ty: T) -> Get<T::Wrapper>
where
  T: FnFut<A>,
{
  Get(ty.into_wrapper())
}

impl<CA, E, RA, T> PathManagement<CA, E, RA> for Get<T>
where
  E: From<crate::Error>,
  T: Endpoint<CA, E, RA>,
{
  const IS_ROUTER: bool = false;

  #[inline]
  async fn manage_path(
    &self,
    ca: &mut CA,
    path_defs: (u8, &[(&'static str, u8)]),
    ra: &mut RA,
    req: &mut Request<ReqResBuffer>,
  ) -> Result<StatusCode, E> {
    if req.method != Method::Get {
      return Err(E::from(crate::Error::from(HttpError::UnexpectedHttpMethod {
        expected: Method::Get,
      })));
    }
    self.0.call(ca, path_defs, ra, req).await
  }

  #[inline]
  fn paths_indices(
    &self,
    _: ArrayVector<(&'static str, u8), 8>,
    _: &mut Vector<ArrayVector<(&'static str, u8), 8>>,
  ) -> crate::Result<()> {
    Ok(())
  }
}

/// Requires a request of type `POST` with json MIME.
#[derive(Debug)]
pub struct Json<T>(
  /// Arbitrary type
  pub T,
);

/// Creates a new [`Json`] instance.
#[inline]
pub fn json<A, T>(ty: T) -> Json<T::Wrapper>
where
  T: FnFut<A>,
{
  Json(ty.into_wrapper())
}

impl<CA, E, T, RA> PathManagement<CA, E, RA> for Json<T>
where
  E: From<crate::Error>,
  T: Endpoint<CA, E, RA>,
{
  const IS_ROUTER: bool = false;

  #[inline]
  async fn manage_path(
    &self,
    ca: &mut CA,
    path_defs: (u8, &[(&'static str, u8)]),
    ra: &mut RA,
    req: &mut Request<ReqResBuffer>,
  ) -> Result<StatusCode, E> {
    if req
      .rrd
      .headers()
      .get_by_name(KnownHeaderName::ContentType.into())
      .map_or(true, |el| el.value == Mime::Json.as_str().as_bytes())
    {
      return Err(E::from(crate::Error::from(HttpError::UnexpectedContentType)));
    }
    if req.method != Method::Post {
      return Err(E::from(crate::Error::from(HttpError::UnexpectedHttpMethod {
        expected: Method::Post,
      })));
    }
    self.0.call(ca, path_defs, ra, req).await
  }

  #[inline]
  fn paths_indices(
    &self,
    _: ArrayVector<(&'static str, u8), 8>,
    _: &mut Vector<ArrayVector<(&'static str, u8), 8>>,
  ) -> crate::Result<()> {
    Ok(())
  }
}

/// Requires a request of type `POST`.
#[derive(Debug)]
pub struct Post<T>(
  /// Arbitrary type
  pub T,
);

/// Creates a new [`Json`] instance.
#[inline]
pub fn post<A, T>(ty: T) -> Post<T::Wrapper>
where
  T: FnFut<A>,
{
  Post(ty.into_wrapper())
}

impl<CA, E, T, RA> PathManagement<CA, E, RA> for Post<T>
where
  E: From<crate::Error>,
  T: Endpoint<CA, E, RA>,
{
  const IS_ROUTER: bool = false;

  #[inline]
  async fn manage_path(
    &self,
    ca: &mut CA,
    path_defs: (u8, &[(&'static str, u8)]),
    ra: &mut RA,
    req: &mut Request<ReqResBuffer>,
  ) -> Result<StatusCode, E> {
    if req.method != Method::Post {
      return Err(E::from(crate::Error::from(HttpError::UnexpectedHttpMethod {
        expected: Method::Post,
      })));
    }
    self.0.call(ca, path_defs, ra, req).await
  }

  #[inline]
  fn paths_indices(
    &self,
    _: ArrayVector<(&'static str, u8), 8>,
    _: &mut Vector<ArrayVector<(&'static str, u8), 8>>,
  ) -> crate::Result<()> {
    Ok(())
  }
}
