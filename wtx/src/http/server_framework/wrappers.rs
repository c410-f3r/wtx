use crate::http::{
  server_framework::{PathFun, PathManagement},
  HttpError, KnownHeaderName, Method, Mime, ReqResData, Request, Response,
};

/// Requires a request of type `GET`.
#[derive(Debug)]
pub struct Get<PF>(
  /// Path Function
  pub PF,
);

impl<E, PF, RRD> PathManagement<E, RRD> for Get<PF>
where
  E: From<crate::Error>,
  PF: PathFun<E, RRD>,
{
  #[inline]
  async fn manage_path(
    &self,
    _: bool,
    matching_path: &'static str,
    req: Request<RRD>,
    req_path_indcs: [usize; 2],
  ) -> Result<Response<RRD>, E> {
    if req.method != Method::Get {
      return Err(E::from(crate::Error::from(HttpError::UnexpectedHttpMethod {
        expected: Method::Get,
      })));
    }
    self.0.call(matching_path, req, req_path_indcs).await
  }
}

/// Creates a new [`Get`] instance with type inference.
pub fn get<I, O>(f: fn(I) -> O) -> Get<fn(I) -> O> {
  Get(f)
}

/// Requires a request of type `POST` with json MIME.
#[derive(Debug)]
pub struct Json<PF>(
  /// Path Function
  pub PF,
);

impl<E, PF, RRD> PathManagement<E, RRD> for Json<PF>
where
  E: From<crate::Error>,
  PF: PathFun<E, RRD>,
  RRD: ReqResData,
{
  #[inline]
  async fn manage_path(
    &self,
    _: bool,
    matching_path: &'static str,
    req: Request<RRD>,
    req_path_indcs: [usize; 2],
  ) -> Result<Response<RRD>, E> {
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

/// Creates a new [`Json`] instance with type inference.
pub fn json<I, O>(f: fn(I) -> O) -> Json<fn(I) -> O> {
  Json(f)
}

/// Requires a request of type `POST`.
#[derive(Debug)]
pub struct Post<PF>(
  /// Path Function
  pub PF,
);

impl<E, PF, RRD> PathManagement<E, RRD> for Post<PF>
where
  E: From<crate::Error>,
  PF: PathFun<E, RRD>,
{
  #[inline]
  async fn manage_path(
    &self,
    _: bool,
    matching_path: &'static str,
    req: Request<RRD>,
    req_path_indcs: [usize; 2],
  ) -> Result<Response<RRD>, E> {
    if req.method != Method::Post {
      return Err(E::from(crate::Error::from(HttpError::UnexpectedHttpMethod {
        expected: Method::Post,
      })));
    }
    self.0.call(matching_path, req, req_path_indcs).await
  }
}

/// Creates a new [`Post`] instance with type inference.
pub fn post<I, O>(f: fn(I) -> O) -> Post<fn(I) -> O> {
  Post(f)
}
