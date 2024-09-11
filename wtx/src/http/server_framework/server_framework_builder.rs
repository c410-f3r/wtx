use crate::http::{
  server_framework::{ConnAux, ReqAux, Router, ServerFramework},
  ConnParams, ReqResBuffer,
};
use alloc::sync::Arc;

/// Server
#[derive(Debug)]
pub struct ServerFrameworkBuilder<CA, E, P, RA, REQM, RESM> {
  cp: ConnParams,
  router: Arc<Router<CA, E, P, RA, REQM, RESM, ReqResBuffer>>,
}

impl<CA, E, P, RA, REQM, RESM> ServerFrameworkBuilder<CA, E, P, RA, REQM, RESM>
where
  CA: ConnAux,
  RA: ReqAux,
{
  /// New instance with default connection values.
  #[inline]
  pub fn new(router: Router<CA, E, P, RA, REQM, RESM, ReqResBuffer>) -> Self {
    Self { cp: ConnParams::default(), router: Arc::new(router) }
  }

  /// Sets the initialization structures for both `CA` and `RA`.
  #[inline]
  pub fn with_aux<CAC, RAC>(
    self,
    ca_cb: CAC,
    ra_cb: RAC,
  ) -> ServerFramework<CA, CAC, E, P, RA, RAC, REQM, RESM>
  where
    CAC: Fn() -> CA::Init,
    RAC: Fn() -> RA::Init,
  {
    ServerFramework { ca_cb, cp: self.cp, ra_cb, router: self.router }
  }

  /// Fills the initialization structures for all auxiliaries with default values.
  #[inline]
  pub fn with_dflt_aux(
    self,
  ) -> ServerFramework<CA, fn() -> CA::Init, E, P, RA, fn() -> RA::Init, REQM, RESM>
  where
    CA::Init: Default,
    RA::Init: Default,
  {
    fn fun<T>() -> T
    where
      T: Default,
    {
      T::default()
    }
    ServerFramework { ca_cb: fun, cp: self.cp, ra_cb: fun, router: self.router }
  }
}

impl<E, P, REQM, RESM> ServerFrameworkBuilder<(), E, P, (), REQM, RESM> {
  /// Build without state
  #[inline]
  pub fn no_aux(self) -> ServerFramework<(), fn() -> (), E, P, (), fn() -> (), REQM, RESM> {
    ServerFramework { ca_cb: nothing, cp: self.cp, ra_cb: nothing, router: self.router }
  }
}

impl<CA, E, P, REQM, RESM> ServerFrameworkBuilder<CA, E, P, (), REQM, RESM>
where
  CA: ConnAux,
{
  /// Sets the initializing strut for `CAA` and sets the request auxiliary to `()`.
  #[inline]
  pub fn with_conn_aux<CAC>(
    self,
    ca_cb: CAC,
  ) -> ServerFramework<CA, CAC, E, P, (), fn() -> (), REQM, RESM>
  where
    CAC: Fn() -> CA::Init,
  {
    ServerFramework { ca_cb, cp: self.cp, ra_cb: nothing, router: self.router }
  }
}

impl<E, P, RA, REQM, RESM> ServerFrameworkBuilder<(), E, P, RA, REQM, RESM>
where
  RA: ReqAux,
{
  /// Sets the initializing strut for `RA` and sets the connection auxiliary to `()`.
  #[inline]
  pub fn with_req_aux<RAC>(
    self,
    ra_cb: RAC,
  ) -> ServerFramework<(), fn() -> (), E, P, RA, RAC, REQM, RESM>
  where
    RAC: Fn() -> RA::Init,
  {
    ServerFramework { ca_cb: nothing, cp: self.cp, ra_cb, router: self.router }
  }
}

fn nothing() {}
