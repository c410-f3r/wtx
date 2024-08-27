use crate::{
  http::{
    server_framework::{Endpoint, PathOwned, PathStr, ResponseFinalizer},
    HttpError, ReqResData, ReqResDataMut, Request, StatusCode,
  },
  misc::{bytes_pos1, str_split1, FnFut1, FnFut2, Vector},
};
use core::str::FromStr;

impl<E, F, P, RES, RRD> Endpoint<PathOwned<P>, E, RRD> for F
where
  E: From<crate::Error>,
  P: FromStr,
  P::Err: Into<crate::Error>,
  F: for<'any> FnFut1<PathOwned<P>, Result = RES>,
  RES: ResponseFinalizer<E, RRD>,
  RRD: ReqResData,
{
  #[inline]
  async fn call(
    &self,
    matching_path: &'static str,
    req: &mut Request<RRD>,
    [begin, _]: [usize; 2],
  ) -> Result<StatusCode, E> {
    let uri = req.rrd.uri();
    let path = manage_path(begin, matching_path, uri.as_str()).map_err(From::from)?;
    let path_owned = PathOwned(P::from_str(path).map_err(Into::into)?);
    (self)(path_owned).await.finalize_response(req)
  }
}

impl<E, F, P, RES, RRD> Endpoint<(&mut Vector<u8>, PathOwned<P>), E, RRD> for F
where
  E: From<crate::Error>,
  P: FromStr,
  P::Err: Into<crate::Error>,
  F: for<'any> FnFut2<&'any mut Vector<u8>, PathOwned<P>, Result = RES>,
  RES: ResponseFinalizer<E, RRD>,
  RRD: ReqResDataMut<Body = Vector<u8>>,
{
  #[inline]
  async fn call(
    &self,
    matching_path: &'static str,
    req: &mut Request<RRD>,
    [begin, _]: [usize; 2],
  ) -> Result<StatusCode, E> {
    let (body, _, uri) = req.rrd.parts_mut();
    let path = manage_path(begin, matching_path, uri.as_str()).map_err(From::from)?;
    let path_owned = PathOwned(P::from_str(path).map_err(Into::into)?);
    (self)(body, path_owned).await.finalize_response(req)
  }
}

impl<'uri, E, F, RES, RRD> Endpoint<(&mut Vector<u8>, PathStr<'uri>), E, RRD> for F
where
  E: From<crate::Error>,
  F: for<'any> FnFut2<&'any mut Vector<u8>, PathStr<'any>, Result = RES>,
  RES: ResponseFinalizer<E, RRD>,
  RRD: ReqResDataMut<Body = Vector<u8>>,
{
  #[inline]
  async fn call(
    &self,
    matching_path: &'static str,
    req: &mut Request<RRD>,
    [begin, _]: [usize; 2],
  ) -> Result<StatusCode, E> {
    let (body, _, uri) = req.rrd.parts_mut();
    let path = manage_path(begin, matching_path, uri.as_str()).map_err(From::from)?;
    (self)(body, PathStr(path)).await.finalize_response(req)
  }
}

#[inline]
fn manage_path<'uri>(
  begin: usize,
  matching_path: &'static str,
  uri: &'uri str,
) -> crate::Result<&'uri str> {
  let req_path = uri.get(begin..).unwrap_or_default();
  bytes_pos1(matching_path.as_bytes(), b':')
    .and_then(|colon_begin| {
      let (mp_before, _) = matching_path.split_at_checked(colon_begin)?;
      let (rp_before, rp_after) = req_path.split_at_checked(colon_begin)?;
      if mp_before != rp_before {
        return None;
      }
      str_split1(rp_after, b'/').next()
    })
    .ok_or_else(|| crate::Error::from(HttpError::UriMismatch))
}
