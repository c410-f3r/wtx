use crate::{
  http::{HttpError, ReqResData, Request, Response},
  misc::{atoi, bytes_pos1, bytes_split1},
};

/// Path function
pub trait PathFun<E, RRD> {
  /// Generates a response.
  fn call(
    &self,
    matching_path: &'static str,
    req: Request<RRD>,
    req_path_indcs: [usize; 2],
  ) -> impl Future<Output = Result<Response<RRD>, E>>;
}

impl<E, RRD, T> PathFun<E, RRD> for &T
where
  T: PathFun<E, RRD>,
{
  #[inline]
  async fn call(
    &self,
    matching_path: &'static str,
    req: Request<RRD>,
    req_path_indcs: [usize; 2],
  ) -> Result<Response<RRD>, E> {
    (*self).call(matching_path, req, req_path_indcs).await
  }
}

impl<E, FUT, RRD> PathFun<E, RRD> for fn(Request<RRD>) -> FUT
where
  FUT: Future<Output = Result<Response<RRD>, E>>,
{
  #[inline]
  async fn call(
    &self,
    _: &'static str,
    req: Request<RRD>,
    _: [usize; 2],
  ) -> Result<Response<RRD>, E> {
    (self)(req).await
  }
}

impl<E, FUT, RRD> PathFun<E, RRD> for fn((u32, Request<RRD>)) -> FUT
where
  E: From<crate::Error>,
  FUT: Future<Output = Result<Response<RRD>, E>>,
  RRD: ReqResData,
{
  #[inline]
  async fn call(
    &self,
    matching_path: &'static str,
    req: Request<RRD>,
    [begin, _]: [usize; 2],
  ) -> Result<Response<RRD>, E> {
    let uri = req.rrd.uri();
    let req_path = uri.as_str().get(begin..).unwrap_or_default().as_bytes();
    let elem: &[u8] = bytes_pos1(matching_path.as_bytes(), b':')
      .and_then(|colon_begin| {
        let (mp_before, _) = matching_path.as_bytes().split_at_checked(colon_begin)?;
        let (rp_before, rp_after) = req_path.split_at_checked(colon_begin)?;
        if mp_before != rp_before {
          return None;
        }
        bytes_split1(rp_after, b'/').next()
      })
      .ok_or_else(|| E::from(HttpError::UriMismatch.into()))?;
    (self)((atoi(elem).map_err(Into::into)?, req)).await
  }
}

#[cfg(feature = "grpc")]
impl<DRSR, E, FUT, RRD> PathFun<E, RRD> for fn((crate::grpc::ServerData<DRSR>, Request<RRD>)) -> FUT
where
  DRSR: Default,
  E: From<crate::Error>,
  FUT: Future<Output = Result<Response<RRD>, E>>,
  RRD: ReqResData,
{
  #[inline]
  async fn call(
    &self,
    _: &'static str,
    req: Request<RRD>,
    _: [usize; 2],
  ) -> Result<Response<RRD>, E> {
    (self)((crate::grpc::ServerData::new(DRSR::default()), req)).await
  }
}
