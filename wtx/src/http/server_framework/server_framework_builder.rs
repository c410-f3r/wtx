use crate::http::{
  conn_params::ConnParams,
  server_framework::{ConnAux, Router, ServerFramework, StreamAux},
};
use alloc::sync::Arc;

/// Server
#[derive(Debug)]
pub struct ServerFrameworkBuilder<CA, E, P, REQM, RESM, SA> {
  cp: ConnParams,
  router: Arc<Router<CA, E, P, REQM, RESM, SA>>,
}

impl<CA, E, P, REQM, RESM, SA> ServerFrameworkBuilder<CA, E, P, REQM, RESM, SA>
where
  CA: ConnAux,
  SA: StreamAux,
{
  /// New instance with default connection values.
  #[inline]
  pub fn new(router: Router<CA, E, P, REQM, RESM, SA>) -> Self {
    Self { cp: ConnParams::default(), router: Arc::new(router) }
  }

  /// Sets the initialization structures for both `CA` and `SA`.
  #[inline]
  pub fn with_aux<CAC, SAC>(
    self,
    ca_cb: CAC,
    ra_cb: SAC,
  ) -> ServerFramework<CA, CAC, E, P, REQM, RESM, SA, SAC>
  where
    CAC: Fn() -> CA::Init,
    SAC: Fn() -> SA::Init,
  {
    ServerFramework { _ca_cb: ca_cb, _cp: self.cp, _sa_cb: ra_cb, _router: self.router }
  }

  /// Fills the initialization structures for all auxiliaries with default values.
  #[inline]
  pub fn with_dflt_aux(
    self,
  ) -> ServerFramework<CA, fn() -> CA::Init, E, P, REQM, RESM, SA, fn() -> SA::Init>
  where
    CA::Init: Default,
    SA::Init: Default,
  {
    fn fun<T>() -> T
    where
      T: Default,
    {
      T::default()
    }
    ServerFramework { _ca_cb: fun, _cp: self.cp, _sa_cb: fun, _router: self.router }
  }
}

impl<E, P, REQM, RESM> ServerFrameworkBuilder<(), E, P, REQM, RESM, ()> {
  /// Build without state
  #[inline]
  pub fn without_aux(self) -> ServerFramework<(), fn() -> (), E, P, REQM, RESM, (), fn() -> ()> {
    ServerFramework { _ca_cb: nothing, _cp: self.cp, _sa_cb: nothing, _router: self.router }
  }
}

impl<CA, E, P, REQM, RESM> ServerFrameworkBuilder<CA, E, P, REQM, RESM, ()>
where
  CA: ConnAux,
{
  /// Sets the initializing strut for `CAA` and sets the request auxiliary to `()`.
  #[inline]
  pub fn with_conn_aux<CAC>(
    self,
    ca_cb: CAC,
  ) -> ServerFramework<CA, CAC, E, P, REQM, RESM, (), fn() -> ()>
  where
    CAC: Fn() -> CA::Init,
  {
    ServerFramework { _ca_cb: ca_cb, _cp: self.cp, _sa_cb: nothing, _router: self.router }
  }
}

impl<E, P, REQM, RESM, SA> ServerFrameworkBuilder<(), E, P, REQM, RESM, SA>
where
  SA: StreamAux,
{
  /// Sets the initializing strut for `SA` and sets the connection auxiliary to `()`.
  #[inline]
  pub fn with_req_aux<SAC>(
    self,
    ra_cb: SAC,
  ) -> ServerFramework<(), fn() -> (), E, P, REQM, RESM, SA, SAC>
  where
    SAC: Fn() -> SA::Init,
  {
    ServerFramework { _ca_cb: nothing, _cp: self.cp, _sa_cb: ra_cb, _router: self.router }
  }
}

fn nothing() {}
