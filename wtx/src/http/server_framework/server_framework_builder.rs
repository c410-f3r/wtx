use crate::{
  http::{
    HttpRecvParams,
    server_framework::{ConnAux, Router, ServerFramework, StreamAux},
  },
  sync::Arc,
};

/// Server
#[derive(Debug)]
pub struct ServerFrameworkBuilder<CA, E, EN, M, S, SA> {
  cp: HttpRecvParams,
  router: Arc<Router<CA, E, EN, M, S, SA>>,
}

impl<CA, E, EN, M, S, SA> ServerFrameworkBuilder<CA, E, EN, M, S, SA>
where
  CA: ConnAux,
  SA: StreamAux,
{
  /// New instance with default connection values.
  #[inline]
  pub fn new(cp: HttpRecvParams, router: Router<CA, E, EN, M, S, SA>) -> Self {
    Self { cp, router: Arc::new(router) }
  }

  /// Sets the initialization structures for both `CA` and `SA`.
  #[inline]
  pub fn with_aux<CACB, SACB>(
    self,
    ca_cb: CACB,
    sa_cb: SACB,
  ) -> ServerFramework<CA, CACB, E, EN, M, S, SA, SACB>
  where
    CACB: Fn() -> Result<CA::Init, E>,
    SACB: Fn(&mut CA) -> Result<SA::Init, E>,
  {
    ServerFramework { _ca_cb: ca_cb, _cp: self.cp, _sa_cb: sa_cb, _router: self.router }
  }

  /// Fills the initialization structures for all auxiliaries with default values.
  #[inline]
  pub fn with_dflt_aux(
    self,
  ) -> ServerFramework<CA, fn() -> CA::Init, E, EN, M, S, SA, fn(&mut CA) -> SA::Init>
  where
    CA::Init: Default,
    SA::Init: Default,
  {
    fn dflt_conn<T>() -> T
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
    ServerFramework { _ca_cb: dflt_conn, _cp: self.cp, _sa_cb: dflt_stream, _router: self.router }
  }
}

impl<E, EN, M, S> ServerFrameworkBuilder<(), E, EN, M, S, ()> {
  /// Build without state
  #[inline]
  pub fn without_aux(
    self,
  ) -> ServerFramework<(), fn() -> Result<(), E>, E, EN, M, S, (), fn(&mut ()) -> Result<(), E>> {
    ServerFramework {
      _ca_cb: nothing_conn,
      _cp: self.cp,
      _sa_cb: nothing_stream,
      _router: self.router,
    }
  }
}

impl<CA, E, EN, M, S> ServerFrameworkBuilder<CA, E, EN, M, S, ()>
where
  CA: ConnAux,
{
  /// Sets the initializing strut for `CA` and sets the request auxiliary to `()`.
  #[inline]
  pub fn with_conn_aux<CACB>(
    self,
    ca_cb: CACB,
  ) -> ServerFramework<CA, CACB, E, EN, M, S, (), fn(&mut CA) -> Result<(), E>>
  where
    CACB: Fn() -> Result<CA::Init, E>,
  {
    ServerFramework { _ca_cb: ca_cb, _cp: self.cp, _sa_cb: nothing_stream, _router: self.router }
  }
}

impl<E, EN, M, S, SA> ServerFrameworkBuilder<(), E, EN, M, S, SA>
where
  SA: StreamAux,
{
  /// Sets the initializing strut for `SA` and sets the connection auxiliary to `()`.
  #[inline]
  pub fn with_stream_aux<SACB>(
    self,
    ra_cb: SACB,
  ) -> ServerFramework<(), fn() -> Result<(), E>, E, EN, M, S, SA, SACB>
  where
    SACB: Fn(&mut ()) -> Result<SA::Init, E>,
  {
    ServerFramework { _ca_cb: nothing_conn, _cp: self.cp, _sa_cb: ra_cb, _router: self.router }
  }
}

fn nothing_conn<E>() -> Result<(), E> {
  Ok(())
}
const fn nothing_stream<CA, E>(_: &mut CA) -> Result<(), E> {
  Ok(())
}
