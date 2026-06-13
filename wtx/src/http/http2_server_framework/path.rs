use crate::{
  http::{
    AutoStream, HttpError, StatusCode,
    http2_server_framework::{Endpoint, ResFinalizer, RouteMatch, StateGeneric},
  },
  misc::{FnFut, FnFutWrapper, UriString, bytes_split1},
};
use core::str::FromStr;

/// URI path converted into an owned type.
#[derive(Debug)]
pub struct Path<T>(
  /// Arbitrary type
  pub T,
);

impl<D, E, F, P, RES, S> Endpoint<D, E, S> for FnFutWrapper<(Path<P>,), F>
where
  E: From<crate::Error>,
  P: FromStr,
  P::Err: Into<crate::Error>,
  F: FnFut<(Path<P>,), Result = RES>,
  RES: ResFinalizer<E>,
{
  #[inline]
  async fn auto(
    &self,
    auto_stream: &mut AutoStream<D>,
    path_defs: (u8, &[RouteMatch]),
  ) -> Result<StatusCode, E> {
    let path = manage_path(path_defs, &auto_stream.req.msg_data.uri)?;
    let path_owned = Path(P::from_str(path).map_err(Into::into)?);
    auto_stream.req.clear();
    self.0.call((path_owned,)).await.finalize_response(&mut auto_stream.req)
  }
}

impl<D, E, F, P, RES, S, const CLEAN: bool> Endpoint<D, E, S>
  for FnFutWrapper<(StateGeneric<'_, D, CLEAN>, Path<P>), F>
where
  E: From<crate::Error>,
  P: FromStr,
  P::Err: Into<crate::Error>,
  F: for<'any> FnFut<(StateGeneric<'any, D, CLEAN>, Path<P>), Result = RES>,
  RES: ResFinalizer<E>,
{
  #[inline]
  async fn auto(
    &self,
    auto_stream: &mut AutoStream<D>,
    path_defs: (u8, &[RouteMatch]),
  ) -> Result<StatusCode, E> {
    let path = manage_path(path_defs, &auto_stream.req.msg_data.uri)?;
    let path_owned = Path(P::from_str(path).map_err(Into::into)?);
    self
      .0
      .call((StateGeneric::new(&mut auto_stream.data, &mut auto_stream.req), path_owned))
      .await
      .finalize_response(&mut auto_stream.req)
  }
}

fn manage_path<'uri>(
  path_defs: (u8, &[RouteMatch]),
  uri: &'uri UriString,
) -> crate::Result<&'uri str> {
  let fun = || {
    let path = uri.path();
    let mut prev_idx: usize = 0;
    let mut iter = path_defs.1.iter().map(|el| el.path.as_bytes());
    while let Some([b'/', sub_path_def @ ..]) = iter.next() {
      prev_idx = prev_idx.wrapping_add(1);
      let mut is_first = true;
      let has_placeholder = bytes_split1(sub_path_def, b'/').any(|elem| {
        if !is_first {
          prev_idx = prev_idx.wrapping_add(1);
        }
        is_first = false;
        if let [b'{', ..] = elem {
          true
        } else {
          prev_idx = prev_idx.wrapping_add(elem.len());
          false
        }
      });
      if !has_placeholder {
        continue;
      };
      return path.get(prev_idx..);
    }
    None
  };
  fun().ok_or_else(|| crate::Error::from(HttpError::MissingUriPlaceholder))
}
