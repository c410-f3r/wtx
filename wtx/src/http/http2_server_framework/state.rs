use crate::{
  collections::Vector,
  futures::{FnFut, FnFutWrapper},
  http::{
    AutoStream, Headers, MsgBufferString, Request, StatusCode,
    http2_server_framework::{Endpoint, ResFinalizer, RouteMatch},
  },
};

/// [`StateGeneric`] with original content
pub type State<'any, D> = StateGeneric<'any, D, false>;
/// [`StateGeneric`] with cleaned content
pub type StateClean<'any, D> = StateGeneric<'any, D, true>;

/// State of a connection
///
/// # If `CLEAN` is true
///
/// When used in an endpoint's argument, request data is automatically cleaned. When used as the return type of an
/// endpoint, response data is automatically cleaned.
//
// The use of generics without lifetimes make `Endpoint::auto` `!Send` when associated with an
// hypothetical `State<&mut D>`, for whatever the reason.
#[derive(Debug)]
pub struct StateGeneric<'any, D, const CLEAN: bool> {
  /// Auxiliary data
  pub data: &'any mut D,
  /// Request/Response Data
  pub req: &'any mut Request<MsgBufferString>,
}

impl<'any, D, const CLEAN: bool> StateGeneric<'any, D, CLEAN> {
  /// Creates an instance with erased data if `CLEAN` is true.
  #[inline]
  pub fn new(data: &'any mut D, req: &'any mut Request<MsgBufferString>) -> Self {
    if CLEAN {
      req.clear();
    }
    Self { data, req }
  }
}

impl<D, E, F, RES, S, const CLEAN: bool> Endpoint<D, E, S>
  for FnFutWrapper<(StateGeneric<'_, D, CLEAN>,), F>
where
  E: From<crate::Error>,
  F: for<'any> FnFut<(StateGeneric<'any, D, CLEAN>,), Result = RES>,
  RES: ResFinalizer<E>,
{
  #[inline]
  async fn auto(
    &self,
    auto_stream: &mut AutoStream<D>,
    _: (u8, &[RouteMatch]),
  ) -> Result<StatusCode, E> {
    self
      .0
      .call((StateGeneric::new(&mut auto_stream.data, &mut auto_stream.req),))
      .await
      .finalize_response(&mut auto_stream.req)
  }
}

impl<'any, D> From<State<'any, D>> for StateClean<'any, D> {
  #[inline]
  fn from(state: State<'any, D>) -> Self {
    Self::new(state.data, state.req)
  }
}

impl<'any, D, const CLEAN: bool> From<&'any mut (D, Request<MsgBufferString>)>
  for StateGeneric<'any, D, CLEAN>
{
  #[inline]
  fn from((data, req): &'any mut (D, Request<MsgBufferString>)) -> Self {
    Self::new(data, req)
  }
}

impl<'any, D, const CLEAN: bool> From<&'any mut (&'any mut D, Request<MsgBufferString>)>
  for StateGeneric<'any, D, CLEAN>
{
  #[inline]
  fn from((data, req): &'any mut (&'any mut D, Request<MsgBufferString>)) -> Self {
    Self::new(data, req)
  }
}

/// Owned state used in testing environments.
#[derive(Debug)]
pub struct StateTest<D> {
  /// Connection auxiliary
  pub data: D,
  /// Request/Response Data
  pub req: Request<MsgBufferString>,
}

impl<D> StateTest<D> {
  /// Mutable parts
  #[inline]
  pub const fn parts_mut(&mut self) -> (&mut D, &mut Request<MsgBufferString>) {
    (&mut self.data, &mut self.req)
  }

  /// Returns a new [`StateGeneric`].
  #[inline]
  pub const fn state<const CLEAR: bool>(&mut self) -> StateGeneric<'_, D, CLEAR> {
    StateGeneric { data: &mut self.data, req: &mut self.req }
  }
}

impl<D> StateTest<D> {
  /// Mutable parts with a modified request that only contains data and headers.
  #[inline]
  pub const fn parts_mut_with_body_and_headers(
    &mut self,
  ) -> (&mut D, Request<(&mut Vector<u8>, &mut Headers)>) {
    (
      &mut self.data,
      Request {
        method: self.req.method,
        msg_data: (&mut self.req.msg_data.body, &mut self.req.msg_data.headers),
      },
    )
  }
}
