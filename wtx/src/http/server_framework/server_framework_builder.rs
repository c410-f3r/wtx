use crate::{
  http::{
    conn_params::ConnParams,
    server_framework::{ConnAux, Router, ServerFramework, StreamAux},
  },
  misc::Arc,
};

/// Server
#[derive(Debug)]
pub struct ServerFrameworkBuilder<CA, CBP, E, EN, M, S, SA> {
  cbp: CBP,
  cp: ConnParams,
  router: Arc<Router<CA, E, EN, M, S, SA>>,
}

impl<CA, CBP, E, EN, M, S, SA> ServerFrameworkBuilder<CA, CBP, E, EN, M, S, SA>
where
  CA: ConnAux,
  SA: StreamAux,
{
  /// New instance with default connection values.
  #[inline]
  pub fn new(cbp: CBP, router: Router<CA, E, EN, M, S, SA>) -> Self {
    Self { cbp, cp: ConnParams::default(), router: Arc::new(router) }
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
  pub fn with_aux<CACB, SACB>(
    self,
    ca_cb: CACB,
    ra_cb: SACB,
  ) -> ServerFramework<CA, CACB, CBP, E, EN, M, S, SA, SACB>
  where
    CACB: Fn(CBP) -> CA::Init,
    SACB: Fn(&mut CA) -> SA::Init,
  {
    ServerFramework {
      _ca_cb: ca_cb,
      _cbp: self.cbp,
      _cp: self.cp,
      _sa_cb: ra_cb,
      _router: self.router,
    }
  }

  /// Fills the initialization structures for all auxiliaries with default values.
  #[inline]
  pub fn with_dflt_aux(
    self,
  ) -> ServerFramework<CA, fn(CBP) -> CA::Init, CBP, E, EN, M, S, SA, fn(&mut CA) -> SA::Init>
  where
    CA::Init: Default,
    SA::Init: Default,
  {
    fn dflt_conn<CBP, T>(_: CBP) -> T
    where
      T: Default,
    {
      T::default()
    }
    fn dflt_stream<CA, T>(_: &mut CA) -> T
    where
      T: Default,
    {
      T::default()
    }
    ServerFramework {
      _ca_cb: dflt_conn,
      _cbp: self.cbp,
      _cp: self.cp,
      _sa_cb: dflt_stream,
      _router: self.router,
    }
  }

  _conn_params_methods!();
}

impl<CBP, E, EN, M, S> ServerFrameworkBuilder<(), CBP, E, EN, M, S, ()> {
  /// Build without state
  #[inline]
  pub fn without_aux(
    self,
  ) -> ServerFramework<(), fn(CBP) -> (), CBP, E, EN, M, S, (), fn(&mut ()) -> ()> {
    ServerFramework {
      _ca_cb: nothing_conn,
      _cbp: self.cbp,
      _cp: self.cp,
      _sa_cb: nothing_stream,
      _router: self.router,
    }
  }
}

impl<CA, CBP, E, EN, M, S> ServerFrameworkBuilder<CA, CBP, E, EN, M, S, ()>
where
  CA: ConnAux,
{
  /// Sets the initializing strut for `CA` and sets the request auxiliary to `()`.
  #[inline]
  pub fn with_conn_aux<CACB>(
    self,
    ca_cb: CACB,
  ) -> ServerFramework<CA, CACB, CBP, E, EN, M, S, (), fn(&mut CA) -> ()>
  where
    CACB: Fn(CBP) -> CA::Init,
  {
    ServerFramework {
      _ca_cb: ca_cb,
      _cbp: self.cbp,
      _cp: self.cp,
      _sa_cb: nothing_stream,
      _router: self.router,
    }
  }
}

impl<CBP, E, EN, M, S, SA> ServerFrameworkBuilder<(), CBP, E, EN, M, S, SA>
where
  SA: StreamAux,
{
  /// Sets the initializing strut for `SA` and sets the connection auxiliary to `()`.
  #[inline]
  pub fn with_stream_aux<SACB>(
    self,
    ra_cb: SACB,
  ) -> ServerFramework<(), fn(CBP) -> (), CBP, E, EN, M, S, SA, SACB>
  where
    SACB: Fn(&mut ()) -> SA::Init,
  {
    ServerFramework {
      _ca_cb: nothing_conn,
      _cbp: self.cbp,
      _cp: self.cp,
      _sa_cb: ra_cb,
      _router: self.router,
    }
  }
}

fn nothing_conn<CBP>(_: CBP) {}
fn nothing_stream<CA>(_: &mut CA) {}
