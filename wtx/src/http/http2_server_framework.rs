//! Tools and libraries that make it easier to write, maintain, and scale web applications.

#[macro_use]
mod macros;

mod cors_middleware;
mod dyn_params;
mod endpoint;
mod endpoint_node;
mod http2_server_framework_error;
mod http_router;
mod json_reply;
mod methods;
mod middleware;
mod path;
mod path_params;
mod redirect;
mod res_finalizer;
mod route_match;
mod state;
#[cfg(test)]
mod tests;
mod verbatim_params;

use crate::{
  collections::ArrayVectorCopy,
  executor::{Executor, Runtime as _, TcpListener as _, TcpStream as _},
  http::{
    AutoStream, HttpRecvParams, ManualStream, MsgBufferString, OperationMode, Request, Response,
    push_h2_alpn,
  },
  http2::{Http2, Http2Buffer, Http2ErrorCode, Http2RecvStatus, ServerStream},
  misc::{TcpParams, Uri},
  rng::{ChaCha20, CryptoRng, CryptoSeedableRng as _, SeedableRng as _, Xorshift64},
  stream::{Stream, StreamReader, StreamWriter},
  sync::Arc,
  tls::{TlsAcceptor, TlsConfig, TlsMode},
};
use core::{mem, net::IpAddr, num::NonZeroUsize};
pub use cors_middleware::{CorsMiddleware, OriginResponse};
pub use dyn_params::DynParams;
pub use endpoint::Endpoint;
pub use endpoint_node::EndpointNode;
pub use http_router::HttpRouter;
pub use http2_server_framework_error::Http2ServerFrameworkError;
pub use json_reply::*;
pub use methods::{
  delete::{Delete, delete},
  get::{Get, get},
  json::{Json, json},
  patch::{Patch, patch},
  post::{Post, post},
  put::{Put, put},
  web_socket::{WebSocket, web_socket},
};
pub use middleware::Middleware;
pub use path::Path;
pub use path_params::PathParams;
pub use redirect::Redirect;
pub use res_finalizer::ResFinalizer;
pub use route_match::RouteMatch;
pub use state::{State, StateClean, StateGeneric, StateTest};
pub use verbatim_params::VerbatimParams;

type LocalStream<EX, TM> =
  ServerStream<<<EX as Executor>::TcpStream as Stream>::WriteHalfOwned, TM>;

/// HTTP/2 Server Framework
#[derive(Debug)]
pub struct Http2ServerFramework<DA, EC, EX, RC, RNG, TM> {
  data: DA,
  error_cb: EC,
  executor: EX,
  hrc: HttpRecvParams,
  rng: RNG,
  runtime_cb: RC,
  runtimes: Option<NonZeroUsize>,
  tcp_params: TcpParams,
  tls_config: Arc<TlsConfig<TM>>,
}

impl<EX, TM>
  Http2ServerFramework<
    (),
    fn(crate::Error),
    EX,
    fn() -> crate::Result<<EX as Executor>::LocalRuntime>,
    ChaCha20,
    TM,
  >
where
  EX: Executor,
{
  /// Taking aside the provided parameters, everything else is set to default values.
  ///
  /// The "h2" ALPN will always be pushed into the TLS configuration.
  #[inline]
  pub fn new(executor: EX, mut tls_config: TlsConfig<TM>) -> crate::Result<Self> {
    push_h2_alpn(tls_config.alpn_mut())?;
    let error_cb: fn(_) = |_| {};
    let runtime_cb: fn() -> _ = || EX::LocalRuntime::optioned();
    Ok(Self {
      data: (),
      error_cb,
      executor,
      hrc: HttpRecvParams::with_optioned_params(),
      rng: ChaCha20::from_std_random()?,
      runtime_cb,
      runtimes: None,
      tcp_params: TcpParams::default(),
      tls_config: tls_config.into(),
    })
  }
}

impl<DA, EC, EX, RC, RNG, TM> Http2ServerFramework<DA, EC, EX, RC, RNG, TM> {
  /// Mutable Random Number Generator.
  #[inline]
  pub const fn rng_mut(&mut self) -> &mut RNG {
    &mut self.rng
  }

  /// Shared data that is cloned across connections and streams.
  #[inline]
  pub fn set_data<_DA>(self, value: _DA) -> Http2ServerFramework<_DA, EC, EX, RC, RNG, TM> {
    Http2ServerFramework {
      data: value,
      error_cb: self.error_cb,
      executor: self.executor,
      hrc: self.hrc,
      rng: self.rng,
      runtime_cb: self.runtime_cb,
      runtimes: self.runtimes,
      tcp_params: self.tcp_params,
      tls_config: self.tls_config,
    }
  }

  /// Sets the error callback function.
  #[inline]
  pub fn set_error_cb<_EC>(self, value: _EC) -> Http2ServerFramework<DA, _EC, EX, RC, RNG, TM> {
    Http2ServerFramework {
      data: self.data,
      error_cb: value,
      executor: self.executor,
      hrc: self.hrc,
      rng: self.rng,
      runtime_cb: self.runtime_cb,
      runtimes: self.runtimes,
      tcp_params: self.tcp_params,
      tls_config: self.tls_config,
    }
  }

  /// See [`HttpRecvParams`].
  #[inline]
  #[must_use]
  pub fn set_http_recv_params(mut self, value: HttpRecvParams) -> Self {
    self.hrc = value;
    self
  }

  /// Allows the tweaking of the chosen runtime.
  #[inline]
  pub fn set_runtime_cb<_RC>(self, value: _RC) -> Http2ServerFramework<DA, EC, EX, _RC, RNG, TM> {
    Http2ServerFramework {
      data: self.data,
      error_cb: self.error_cb,
      executor: self.executor,
      hrc: self.hrc,
      rng: self.rng,
      runtime_cb: value,
      runtimes: self.runtimes,
      tcp_params: self.tcp_params,
      tls_config: self.tls_config,
    }
  }

  /// See [`TcpParams`].
  #[inline]
  #[must_use]
  pub fn set_tcp_params(mut self, value: TcpParams) -> Self {
    self.tcp_params = value;
    self
  }

  /// The number of spawned runtimes. Shouldn't be greater than the number of available threads.
  ///
  /// Only works when calling [`Self::run_in_threads`]. If [`None`], defaults to the number of threads.
  #[inline]
  #[must_use]
  pub fn set_runtimes(mut self, value: Option<NonZeroUsize>) -> Self {
    self.runtimes = value;
    self
  }

  /// The "h2" ALPN will always be pushed into the TLS configuration.
  #[inline]
  pub fn set_tls_config(mut self, mut value: TlsConfig<TM>) -> crate::Result<Self> {
    push_h2_alpn(value.alpn_mut())?;
    self.tls_config = value.into();
    Ok(self)
  }
}

impl<DA, EC, EX, RC, RNG, TM> Http2ServerFramework<DA, EC, EX, RC, RNG, TM>
where
  DA: Clone + Send + Sync + 'static,
  EX: Clone + Executor + Send + 'static,
  RNG: CryptoRng,
  TM: TlsMode + Send + 'static,
{
  /// Starts the server listening on the specified address.
  #[inline]
  pub async fn run<EN, ER, M>(
    self,
    addr: &str,
    hr: HttpRouter<DA, EN, ER, M, LocalStream<EX, TM>>,
  ) -> Result<(), ER>
  where
    EC: Clone + Fn(ER) + Send + 'static,
    EN: EndpointNode<DA, ER, LocalStream<EX, TM>, auto(..): Send, manual(..): Send>
      + Send
      + Sync
      + 'static,
    ER: From<crate::Error> + Send + Sync + 'static,
    M: Middleware<DA, ER, req(..): Send, res(..): Send> + Send + Sync + 'static,
    M::Aux: Send,
    <EX as Executor>::SpawnFuture<()>: Send,
    <EX as Executor>::TcpStream: Send + 'static,
    <EX::TcpStream as Stream>::ReadHalfOwned: Send + 'static,
    <EX::TcpStream as Stream>::WriteHalfOwned: Send + 'static,
    <<EX::TcpStream as Stream>::ReadHalfOwned as StreamReader>::read(..): Send,
    <<EX::TcpStream as Stream>::ReadHalfOwned as StreamReader>::read_skip(..): Send,
    <<EX::TcpStream as Stream>::WriteHalfOwned as StreamWriter>::write_all(..): Send,
    <<EX::TcpStream as Stream>::WriteHalfOwned as StreamWriter>::write_all_vectored(..): Send,
  {
    do_run(addr, hr, self).await
  }

  /// Starts the server listening on the specified address across different runtimes
  #[inline]
  pub async fn run_in_threads<EN, ER, M>(
    self,
    addr: &str,
    hr: HttpRouter<DA, EN, ER, M, LocalStream<EX, TM>>,
  ) -> Result<(), ER>
  where
    EC: Clone + Fn(ER) + Send + 'static,
    EN: EndpointNode<DA, ER, LocalStream<EX, TM>, auto(..): Send, manual(..): Send>
      + Send
      + Sync
      + 'static,
    ER: From<crate::Error> + Send + Sync + 'static,
    M: Middleware<DA, ER, req(..): Send, res(..): Send> + Send + Sync + 'static,
    M::Aux: Send,
    <EX as Executor>::SpawnFuture<()>: Send,
    <EX as Executor>::TcpStream: Send + 'static,
    <EX::TcpStream as Stream>::ReadHalfOwned: Send + 'static,
    <EX::TcpStream as Stream>::WriteHalfOwned: Send + 'static,
    <<EX::TcpStream as Stream>::ReadHalfOwned as StreamReader>::read(..): Send,
    <<EX::TcpStream as Stream>::ReadHalfOwned as StreamReader>::read_skip(..): Send,
    <<EX::TcpStream as Stream>::WriteHalfOwned as StreamWriter>::write_all(..): Send,
    <<EX::TcpStream as Stream>::WriteHalfOwned as StreamWriter>::write_all_vectored(..): Send,
  {
    do_run(addr, hr, self).await
  }
}

async fn do_run<DA, EC, EN, ER, EX, M, RC, RNG, TM>(
  addr: &str,
  hr: HttpRouter<DA, EN, ER, M, LocalStream<EX, TM>>,
  hsf: Http2ServerFramework<DA, EC, EX, RC, RNG, TM>,
) -> Result<(), ER>
where
  DA: Clone + Send + Sync + 'static,
  EC: Clone + Fn(ER) + Send + 'static,
  EN: EndpointNode<DA, ER, LocalStream<EX, TM>, auto(..): Send, manual(..): Send>
    + Send
    + Sync
    + 'static,
  ER: From<crate::Error> + Send + Sync + 'static,
  EX: Clone + Executor + Send + 'static,
  M: Middleware<DA, ER, req(..): Send, res(..): Send> + Send + Sync + 'static,
  M::Aux: Send,
  RNG: CryptoRng,
  TM: TlsMode + Send + 'static,
  <EX as Executor>::SpawnFuture<()>: Send,
  <EX as Executor>::TcpStream: Send + 'static,
  <EX::TcpStream as Stream>::ReadHalfOwned: Send + 'static,
  <EX::TcpStream as Stream>::WriteHalfOwned: Send + 'static,
  <<EX::TcpStream as Stream>::ReadHalfOwned as StreamReader>::read(..): Send,
  <<EX::TcpStream as Stream>::ReadHalfOwned as StreamReader>::read_skip(..): Send,
  <<EX::TcpStream as Stream>::WriteHalfOwned as StreamWriter>::write_all(..): Send,
  <<EX::TcpStream as Stream>::WriteHalfOwned as StreamWriter>::write_all_vectored(..): Send,
{
  let Http2ServerFramework {
    data,
    error_cb,
    executor,
    hrc,
    mut rng,
    runtime_cb: _,
    runtimes: _,
    tcp_params,
    tls_config,
  } = hsf;
  let http_router = Arc::new(hr);
  let uri = Uri::new(addr);
  let listener = EX::TcpListener::bind(uri.hostname_with_implied_port(), tcp_params).await?;
  let mut xorshift = Xorshift64::from_simple_seed()?;
  loop {
    let stream = listener.accept(tcp_params).await?.0;
    let tls_stream =
      TlsAcceptor::new(&*tls_config, &mut rng, stream).accept().await?.rslt()?.stream;
    let conn_data = data.clone();
    let conn_error_cb0 = error_cb.clone();
    let conn_error_cb1 = error_cb.clone();
    let conn_executor = executor.clone();
    let conn_hrc = hrc;
    let conn_http_router = http_router.clone();
    let conn_peer = tls_stream.stream.peer_addr()?.ip();
    let mut conn_xorshift = Xorshift64::from_rng(&mut xorshift)?;

    let _conn_jh = executor.spawn(async move {
      let conn_init_fut = async move {
        let hb = Http2Buffer::new(&mut conn_xorshift);
        let (frame_reader, http2) = Http2::accept(hb, conn_hrc, tls_stream.into_split()?).await?;
        Ok::<_, ER>((frame_reader, http2))
      };
      let (frame_reader, http2) = match conn_init_fut.await {
        Err(err) => {
          conn_error_cb0(err);
          return;
        }
        Ok(elem) => elem,
      };
      let another_http2 = http2.clone();
      let _frame_reader_jh = conn_executor.spawn(frame_reader);
      let conn_rest_fut = async move {
        loop {
          let stream_data = conn_data.clone();
          let Some((server_stream, rslt)) = http2
            .stream(|req, _| {
              let rslt = *conn_http_router.router.find(req.msg_data.uri.path())?.data();
              Ok::<_, ER>(match rslt.1 {
                OperationMode::Auto => (rslt.0, None),
                OperationMode::Manual => (rslt.0, Some(mem::take(req.msg_data))),
              })
            })
            .await?
          else {
            return Ok(());
          };
          let (headers_aux, opt) = rslt?;
          let stream_err_cb = conn_error_cb0.clone();
          let stream_http_router = conn_http_router.clone();
          let _stream_jh = conn_executor.spawn(stream_jh::<DA, EC, EN, ER, EX, M, TM>(
            conn_peer,
            headers_aux,
            opt,
            server_stream,
            stream_data,
            stream_err_cb,
            stream_http_router,
          ));
        }
      };
      if let Err(err) = conn_rest_fut.await {
        another_http2.send_go_away(Http2ErrorCode::NoError).await;
        conn_error_cb1(err);
      }
    });
  }
}

fn log_req(_peer: &IpAddr, _req: &Request<MsgBufferString>) {
  let _method = _req.method.strings().custom[0];
  let _path = _req.msg_data.uri.path();
  _debug!(r#"{_peer} "{_method} {_path}""#,);
}

async fn stream_jh<DA, EC, EN, ER, EX, M, TM>(
  conn_peer: IpAddr,
  headers_aux: ArrayVectorCopy<RouteMatch, 4>,
  opt: Option<MsgBufferString>,
  mut server_stream: LocalStream<EX, TM>,
  stream_data: DA,
  stream_err_cb: EC,
  stream_http_router: Arc<HttpRouter<DA, EN, ER, M, LocalStream<EX, TM>>>,
) where
  EC: Fn(ER),
  EN: EndpointNode<DA, ER, LocalStream<EX, TM>, auto(..): Send, manual(..): Send>,
  ER: From<crate::Error>,
  EX: Executor,
  M: Middleware<DA, ER, req(..): Send, res(..): Send>,
{
  let stream_fun = async {
    if let Some(local_rrb) = opt {
      let req = Request::http2(server_stream.method(), local_rrb);
      log_req(&conn_peer, &req);
      let manual_stream = ManualStream {
        data: stream_data,
        peer: conn_peer,
        protocol: server_stream.protocol(),
        req,
        stream: server_stream.clone(),
      };
      stream_http_router.en.manual(manual_stream, (0, &headers_aux)).await?;
      return Ok(());
    }
    let (hrs, local_rrb) = server_stream.recv_req().await?;
    if let Http2RecvStatus::ClosedConnection | Http2RecvStatus::ClosedStream(_) = hrs {
      return Ok(());
    }
    let req = local_rrb.into_http2_request(server_stream.method());
    log_req(&conn_peer, &req);
    let mut auto_stream = AutoStream::new(stream_data, conn_peer, server_stream.protocol(), req);
    let status = stream_http_router.auto(&mut auto_stream, (0, &headers_aux)).await?;
    let enc_buffer = mem::take(&mut auto_stream.req.msg_data.uri).into_inner();
    let _ = server_stream
      .send_res(
        &mut enc_buffer.into_bytes().into(),
        Response::new(auto_stream.req.msg_data, status),
      )
      .await?;
    Ok::<_, ER>(())
  };
  let stream_fun_rslt = stream_fun.await;
  let _rslt = server_stream.common().clear().await;
  if let Err(err) = stream_fun_rslt {
    server_stream.common().send_go_away(Http2ErrorCode::InternalError).await;
    stream_err_cb(err);
  }
}
