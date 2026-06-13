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
  executor::{Executor, Runtime, TcpListener, TcpStream},
  http::{
    AutoStream, HttpRecvParams, ManualStream, MsgBufferString, OperationMode, Request, Response,
  },
  http2::{Http2, Http2Buffer, Http2ErrorCode, Http2RecvStatus, ServerStream},
  misc::{TcpParams, Uri},
  rng::{ChaCha20, CryptoSeedableRng as _, SeedableRng, Xorshift64},
  stream::{StreamReader, StreamWriter},
  sync::Arc,
  tls::{TlsConfig, TlsModeVerified},
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

type LocalStream<TS> = ServerStream<Http2Buffer, <TS as TcpStream>::WriteHalf>;

/// Server
#[derive(Debug)]
pub struct Http2ServerFramework<DA, EC, EX, RC, RNG, TM> {
  data: DA,
  error_cb: EC,
  executor: EX,
  hrc: HttpRecvParams,
  rng: RNG,
  runtime_cb: RC,
  tcp_params: TcpParams,
  threads: Option<NonZeroUsize>,
  tls_config: Option<Arc<TlsConfig<'static>>>,
  tls_mode: TM,
}

impl<EX>
  Http2ServerFramework<
    (),
    fn(crate::Error),
    EX,
    fn() -> crate::Result<<EX as Executor>::LocalRuntime>,
    ChaCha20,
    TlsModeVerified,
  >
where
  EX: Executor,
{
  #[inline]
  pub fn new(executor: EX) -> crate::Result<Self> {
    let error_cb: fn(_) = |_| {};
    let runtime_cb: fn() -> _ = || EX::LocalRuntime::optioned();
    Ok(Self {
      data: (),
      error_cb,
      executor,
      hrc: HttpRecvParams::with_optioned_params(),
      rng: ChaCha20::from_std_random()?,
      runtime_cb,
      tcp_params: TcpParams::default(),
      threads: None,
      tls_config: None,
      tls_mode: TlsModeVerified,
    })
  }
}

impl<DA, EC, EX, RC, RNG, TM> Http2ServerFramework<DA, EC, EX, RC, RNG, TM> {
  #[inline]
  pub const fn rng_mut(&mut self) -> &mut RNG {
    &mut self.rng
  }

  #[inline]
  pub fn set_data<_DA>(self, value: _DA) -> Http2ServerFramework<_DA, EC, EX, RC, RNG, TM> {
    Http2ServerFramework {
      data: value,
      error_cb: self.error_cb,
      executor: self.executor,
      hrc: self.hrc,
      rng: self.rng,
      runtime_cb: self.runtime_cb,
      tcp_params: self.tcp_params,
      threads: self.threads,
      tls_config: self.tls_config,
      tls_mode: self.tls_mode,
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
      tcp_params: self.tcp_params,
      threads: self.threads,
      tls_config: self.tls_config,
      tls_mode: self.tls_mode,
    }
  }

  #[inline]
  pub fn set_runtime_cb<_RC>(self, value: _RC) -> Http2ServerFramework<DA, EC, EX, _RC, RNG, TM> {
    Http2ServerFramework {
      data: self.data,
      error_cb: self.error_cb,
      executor: self.executor,
      hrc: self.hrc,
      rng: self.rng,
      runtime_cb: value,
      tcp_params: self.tcp_params,
      threads: self.threads,
      tls_config: self.tls_config,
      tls_mode: self.tls_mode,
    }
  }

  /// See [`TcpParams`].
  #[inline]
  pub fn set_tcp_params(mut self, value: TcpParams) -> Self {
    self.tcp_params = value;
    self
  }

  #[inline]
  pub fn set_threads(mut self, value: Option<NonZeroUsize>) -> Self {
    self.threads = value;
    self
  }

  #[inline]
  pub fn set_tls_config(mut self, value: TlsConfig<'static>) -> Self {
    self.tls_config = Some(Arc::new(value));
    self
  }

  /// Sets the TLS mode.
  #[inline]
  pub fn set_tls_mode<_TM>(self, value: _TM) -> Http2ServerFramework<DA, EC, EX, RC, RNG, _TM> {
    Http2ServerFramework {
      data: self.data,
      error_cb: self.error_cb,
      executor: self.executor,
      hrc: self.hrc,
      rng: self.rng,
      runtime_cb: self.runtime_cb,
      tcp_params: self.tcp_params,
      threads: self.threads,
      tls_config: self.tls_config,
      tls_mode: value,
    }
  }
}

impl<DA, EC, EX, RC, RNG, TM> Http2ServerFramework<DA, EC, EX, RC, RNG, TM>
where
  DA: Clone + Send + Sync + 'static,
  EX: Clone + Executor + Send + 'static,
{
  pub async fn run<EN, ER, M>(
    self,
    addr: &str,
    hr: HttpRouter<DA, EN, ER, M, LocalStream<EX::TcpStream>>,
  ) -> Result<(), ER>
  where
    EC: Clone + Fn(ER) + Send + 'static,
    EN: EndpointNode<DA, ER, LocalStream<EX::TcpStream>, auto(..): Send, manual(..): Send>
      + Send
      + Sync
      + 'static,
    ER: From<crate::Error> + Send + Sync + 'static,
    M: Middleware<DA, ER, req(..): Send, res(..): Send> + Send + Sync + 'static,
    M::Aux: Send,
    <EX as Executor>::SpawnFuture<()>: Send,
    <EX as Executor>::TcpStream: Send + 'static,
    <EX::TcpStream as TcpStream>::ReadHalf: Send + 'static,
    <EX::TcpStream as TcpStream>::WriteHalf: Send + 'static,
    <<EX::TcpStream as TcpStream>::ReadHalf as StreamReader>::read(..): Send,
    <<EX::TcpStream as TcpStream>::ReadHalf as StreamReader>::read_skip(..): Send,
    <<EX::TcpStream as TcpStream>::WriteHalf as StreamWriter>::write_all(..): Send,
    <<EX::TcpStream as TcpStream>::WriteHalf as StreamWriter>::write_all_vectored(..): Send,
  {
    do_run(addr, hr, self).await
  }

  pub async fn run_in_threads<EN, ER, M>(
    self,
    addr: &str,
    hr: HttpRouter<DA, EN, ER, M, LocalStream<EX::TcpStream>>,
  ) -> Result<(), ER>
  where
    EC: Clone + Fn(ER) + Send + 'static,
    EN: EndpointNode<DA, ER, LocalStream<EX::TcpStream>, auto(..): Send, manual(..): Send>
      + Send
      + Sync
      + 'static,
    ER: From<crate::Error> + Send + Sync + 'static,
    M: Middleware<DA, ER, req(..): Send, res(..): Send> + Send + Sync + 'static,
    M::Aux: Send,
    <EX as Executor>::SpawnFuture<()>: Send,
    <EX as Executor>::TcpStream: Send + 'static,
    <EX::TcpStream as TcpStream>::ReadHalf: Send + 'static,
    <EX::TcpStream as TcpStream>::WriteHalf: Send + 'static,
    <<EX::TcpStream as TcpStream>::ReadHalf as StreamReader>::read(..): Send,
    <<EX::TcpStream as TcpStream>::ReadHalf as StreamReader>::read_skip(..): Send,
    <<EX::TcpStream as TcpStream>::WriteHalf as StreamWriter>::write_all(..): Send,
    <<EX::TcpStream as TcpStream>::WriteHalf as StreamWriter>::write_all_vectored(..): Send,
  {
    do_run(addr, hr, self).await
  }
}

async fn do_run<DA, EC, EN, ER, EX, M, RC, RNG, TM>(
  addr: &str,
  hr: HttpRouter<DA, EN, ER, M, LocalStream<EX::TcpStream>>,
  hsf: Http2ServerFramework<DA, EC, EX, RC, RNG, TM>,
) -> Result<(), ER>
where
  DA: Clone + Send + Sync + 'static,
  EC: Clone + Fn(ER) + Send + 'static,
  EN: EndpointNode<DA, ER, LocalStream<EX::TcpStream>, auto(..): Send, manual(..): Send>
    + Send
    + Sync
    + 'static,
  ER: From<crate::Error> + Send + Sync + 'static,
  EX: Clone + Executor + Send + 'static,
  M: Middleware<DA, ER, req(..): Send, res(..): Send> + Send + Sync + 'static,
  M::Aux: Send,
  <EX as Executor>::SpawnFuture<()>: Send,
  <EX as Executor>::TcpStream: Send + 'static,
  <EX::TcpStream as TcpStream>::ReadHalf: Send + 'static,
  <EX::TcpStream as TcpStream>::WriteHalf: Send + 'static,
  <<EX::TcpStream as TcpStream>::ReadHalf as StreamReader>::read(..): Send,
  <<EX::TcpStream as TcpStream>::ReadHalf as StreamReader>::read_skip(..): Send,
  <<EX::TcpStream as TcpStream>::WriteHalf as StreamWriter>::write_all(..): Send,
  <<EX::TcpStream as TcpStream>::WriteHalf as StreamWriter>::write_all_vectored(..): Send,
{
  let Http2ServerFramework {
    data,
    error_cb,
    executor,
    hrc,
    rng,
    runtime_cb,
    tcp_params,
    threads: _,
    tls_config,
    tls_mode,
  } = hsf;
  let http_router = Arc::new(hr);
  let uri = Uri::new(addr);
  let listener = EX::TcpListener::bind(uri.hostname_with_implied_port(), tcp_params).await?;
  let mut xorshift = Xorshift64::from_simple_seed()?;
  loop {
    let stream = listener.accept().await?.0;
    let conn_data = data.clone();
    let conn_error_cb0 = error_cb.clone();
    let conn_error_cb1 = error_cb.clone();
    let conn_executor = executor.clone();
    let conn_hrc = hrc.clone();
    let conn_http_router = http_router.clone();
    let conn_peer = stream.peer_addr().map_err(crate::Error::from)?.ip();
    let mut conn_xorshift = Xorshift64::from_rng(&mut xorshift)?;

    let _conn_jh = executor.spawn(async move {
      let conn_init_fut = async move {
        let hb = Http2Buffer::new(&mut conn_xorshift);
        let (frame_reader, http2) = Http2::accept(hb, conn_hrc, stream.into_split()?).await?;
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
          let (mut stream, rslt) = match http2
            .stream(|req, _| {
              let rslt = conn_http_router.router.find(req.msg_data.uri.path())?.data().clone();
              Ok::<_, ER>(match rslt.1 {
                OperationMode::Auto => (rslt.0, None),
                OperationMode::Manual => (rslt.0, Some(mem::take(req.msg_data))),
              })
            })
            .await?
          {
            None => return Ok(()),
            Some(elem) => elem,
          };
          let (headers_aux, opt) = rslt?;
          let stream_err_cb = conn_error_cb0.clone();
          let stream_http_router = conn_http_router.clone();
          let _stream_jh = conn_executor.spawn(async move {
            let stream_fun = async {
              if let Some(local_rrb) = opt {
                let req = Request::http2(stream.method(), local_rrb);
                log_req(&conn_peer, &req);
                let manual_stream = ManualStream {
                  data: stream_data,
                  peer: conn_peer,
                  protocol: stream.protocol(),
                  req,
                  stream: stream.clone(),
                };
                stream_http_router.en.manual(manual_stream, (0, &headers_aux)).await?;
                return Ok(());
              }
              let (hrs, local_rrb) = stream.recv_req().await?;
              if let Http2RecvStatus::ClosedConnection | Http2RecvStatus::ClosedStream(_) = hrs {
                return Ok(());
              }
              let req = local_rrb.into_http2_request(stream.method());
              log_req(&conn_peer, &req);
              let mut auto_stream =
                AutoStream { data: stream_data, peer: conn_peer, protocol: stream.protocol(), req };
              let status_code =
                stream_http_router.auto(&mut auto_stream, (0, &headers_aux)).await?;
              let _ = stream
                .send_res(Response { msg_data: auto_stream.req.msg_data, status_code })
                .await?;
              Ok::<_, ER>(())
            };
            let stream_fun_rslt = stream_fun.await;
            let _rslt = stream.common().clear().await;
            if let Err(err) = stream_fun_rslt {
              stream.common().send_go_away(Http2ErrorCode::InternalError).await;
              stream_err_cb(err);
            }
          });
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
