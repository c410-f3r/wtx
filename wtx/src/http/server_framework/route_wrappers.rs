#![expect(clippy::partial_pub_fields, reason = "necessary due to type checker")]

use crate::http::{
  server_framework::{Endpoint, PathManagement},
  HttpError, KnownHeaderName, Method, Mime, ReqResData, Request, StatusCode,
};
use core::marker::PhantomData;

/// Requires a request of type `GET`.
#[derive(Debug)]
pub struct Get<A, T>(
  /// Arbitrary type
  pub T,
  PhantomData<A>,
);

/// Creates a new [`Get`] instance.
#[inline]
pub fn get<A, T>(ty: T) -> Get<A, T> {
  Get(ty, PhantomData)
}

impl<A, E, RRD, T> PathManagement<E, RRD> for Get<A, T>
where
  E: From<crate::Error>,
  T: Endpoint<A, E, RRD>,
{
  #[inline]
  async fn manage_path(
    &self,
    _: bool,
    matching_path: &'static str,
    req: &mut Request<RRD>,
    req_path_indcs: [usize; 2],
  ) -> Result<StatusCode, E> {
    if req.method != Method::Get {
      return Err(E::from(crate::Error::from(HttpError::UnexpectedHttpMethod {
        expected: Method::Get,
      })));
    }
    self.0.call(matching_path, req, req_path_indcs).await
  }
}

/// Requires a request of type `POST` with json MIME.
#[derive(Debug)]
pub struct Json<A, T>(
  /// Arbitrary type
  pub T,
  PhantomData<A>,
);

/// Creates a new [`Json`] instance.
#[inline]
pub fn json<A, T>(ty: T) -> Json<A, T> {
  Json(ty, PhantomData)
}

impl<A, E, T, RRD> PathManagement<E, RRD> for Json<A, T>
where
  E: From<crate::Error>,
  T: Endpoint<A, E, RRD>,
  RRD: ReqResData,
{
  #[inline]
  async fn manage_path(
    &self,
    _: bool,
    matching_path: &'static str,
    req: &mut Request<RRD>,
    req_path_indcs: [usize; 2],
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
    self.0.call(matching_path, req, req_path_indcs).await
  }
}

/// Requires a request of type `POST`.
#[derive(Debug)]
pub struct Post<A, T>(
  /// Arbitrary type
  pub T,
  PhantomData<A>,
);

/// Creates a new [`Post`] instance.
#[inline]
pub fn post<A, T>(ty: T) -> Post<A, T> {
  Post(ty, PhantomData)
}

impl<A, E, T, RRD> PathManagement<E, RRD> for Post<A, T>
where
  E: From<crate::Error>,
  T: Endpoint<A, E, RRD>,
{
  #[inline]
  async fn manage_path(
    &self,
    _: bool,
    matching_path: &'static str,
    req: &mut Request<RRD>,
    req_path_indcs: [usize; 2],
  ) -> Result<StatusCode, E> {
    if req.method != Method::Post {
      return Err(E::from(crate::Error::from(HttpError::UnexpectedHttpMethod {
        expected: Method::Post,
      })));
    }
    self.0.call(matching_path, req, req_path_indcs).await
  }
}
