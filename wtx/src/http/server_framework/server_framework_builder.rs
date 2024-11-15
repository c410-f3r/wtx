use crate::{
  http::{
    conn_params::ConnParams,
    server_framework::{ConnAux, Router, ServerFramework, StreamAux},
  },
  misc::Arc,
};

/// Server
#[derive(Debug)]
pub struct ServerFrameworkBuilder<CA, E, EN, M, S, SA> {
  cp: ConnParams,
  router: Arc<Router<CA, E, EN, M, S, SA>>,
}

impl<CA, E, EN, M, S, SA> ServerFrameworkBuilder<CA, E, EN, M, S, SA>
where
  CA: ConnAux,
  SA: StreamAux,
{
  /// New instance with default connection values.
  #[inline]
  pub fn new(router: Router<CA, E, EN, M, S, SA>) -> Self {
    Self { cp: ConnParams::default(), router: Arc::new(router) }
  }

  /// Maximum number of active concurrent streams
  #[inline]
  #[must_use]
  pub fn enable_connect_protocol(mut self, elem: bool) -> Self {
    self.cp._enable_connect_protocol = elem;
    self
  }

  /// Sets the initialization structures for both `CA` and `SA`.
  #[inline]
  pub fn with_aux<CAC, SAC>(
    self,
    ca_cb: CAC,
    ra_cb: SAC,
  ) -> ServerFramework<CA, CAC, E, EN, M, S, SA, SAC>
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
  ) -> ServerFramework<CA, fn() -> CA::Init, E, EN, M, S, SA, fn() -> SA::Init>
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

  _conn_params_methods!();
}

impl<E, EN, M, S> ServerFrameworkBuilder<(), E, EN, M, S, ()> {
  /// Build without state
  #[inline]
  pub fn without_aux(self) -> ServerFramework<(), fn() -> (), E, EN, M, S, (), fn() -> ()> {
    ServerFramework { _ca_cb: nothing, _cp: self.cp, _sa_cb: nothing, _router: self.router }
  }
}

impl<CA, E, EN, M, S> ServerFrameworkBuilder<CA, E, EN, M, S, ()>
where
  CA: ConnAux,
{
  /// Sets the initializing strut for `CAA` and sets the request auxiliary to `()`.
  #[inline]
  pub fn with_conn_aux<CAC>(
    self,
    ca_cb: CAC,
  ) -> ServerFramework<CA, CAC, E, EN, M, S, (), fn() -> ()>
  where
    CAC: Fn() -> CA::Init,
  {
    ServerFramework { _ca_cb: ca_cb, _cp: self.cp, _sa_cb: nothing, _router: self.router }
  }
}

impl<E, EN, M, S, SA> ServerFrameworkBuilder<(), E, EN, M, S, SA>
where
  SA: StreamAux,
{
  /// Sets the initializing strut for `SA` and sets the connection auxiliary to `()`.
  #[inline]
  pub fn with_req_aux<SAC>(
    self,
    ra_cb: SAC,
  ) -> ServerFramework<(), fn() -> (), E, EN, M, S, SA, SAC>
  where
    SAC: Fn() -> SA::Init,
  {
    ServerFramework { _ca_cb: nothing, _cp: self.cp, _sa_cb: ra_cb, _router: self.router }
  }
}

fn nothing() {}
