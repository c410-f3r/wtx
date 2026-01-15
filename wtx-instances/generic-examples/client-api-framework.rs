//! Illustrates how the `client-api-framework` feature facilitates the management and utilization
//! of large API endpoints for both HTTP and WebSocket requests.
//!
//! Contains one API called `GenericThrottlingApi` and its two endpoints: an HTTP JSON-RPC
//! `genericHttpRequest` and an WebSocket `genericWebSocketSubscription`.
//!
//! Everything that is not inside `main` should be constructed only once in your program.

extern crate serde;
extern crate tokio;
extern crate wtx;

use core::time::Duration;
use tokio::net::TcpStream;
use wtx::{
  client_api_framework::{
    Api,
    misc::{Pair, RequestCounter, RequestLimit},
    network::{HttpParams, WsParams, transport::SendingReceivingTransport},
  },
  de::format::SerdeJson,
  http::client_pool::{ClientPoolBuilder, ClientPoolTokio},
  misc::Uri,
  rng::{ChaCha20, SeedableRng as _, Xorshift64},
  tls::{TlsBuffer, TlsConfig, TlsConnector, TlsModeVerifyFull, TlsStream},
  web_socket::{WebSocket, WebSocketBuffer, WebSocketConnector},
};

wtx::create_packages_aux_wrapper!();

#[derive(Debug)]
#[wtx::api(error(wtx::Error), pkgs_aux(PkgsAux), transport(http, ws))]
pub struct GenericThrottlingApi {
  pub rc: RequestCounter,
}

impl Api for GenericThrottlingApi {
  type Error = wtx::Error;
  type Id = GenericThrottlingApiId;

  async fn before_sending(&mut self) -> Result<(), Self::Error> {
    self.rc.update_params().await?;
    Ok(())
  }
}

#[wtx::pkg(
  data_format(json_rpc("genericHttpRequest")),
  id(crate::GenericThrottlingApiId),
  transport(http)
)]
mod generic_http_request {
  #[pkg::aux]
  impl<A, DRSR> crate::HttpPkgsAux<A, DRSR> {}

  #[derive(Debug, serde::Serialize)]
  #[pkg::req_data]
  pub struct GenericHttpRequestReq(#[pkg::field(name = "generic_number")] i32);

  #[pkg::res_data]
  pub type GenericHttpRequestRes = (u8, u16, u32);
}

#[wtx::pkg(
  data_format(json_rpc("genericWebSocketSubscription")),
  id(crate::GenericThrottlingApiId),
  transport(ws)
)]
mod generic_web_socket_subscription {
  #[pkg::aux]
  impl<A, DRSR> crate::WsPkgsAux<A, DRSR> {}

  #[derive(Debug, serde::Serialize)]
  #[pkg::req_data]
  pub struct GenericWebSocketSubscriptionReq<'str> {
    generic_string: &'str str,
    #[serde(skip_serializing_if = "Option::is_none")]
    generic_number: Option<i32>,
  }

  #[pkg::res_data]
  pub type GenericWebSocketSubscriptionRes = u64;
}

fn http_pair() -> Pair<
  PkgsAux<GenericThrottlingApi, SerdeJson, HttpParams>,
  ClientPoolTokio<(), fn(&()) -> TlsModeVerifyFull>,
> {
  fn fun(_: &()) -> TlsModeVerifyFull {
    TlsModeVerifyFull
  }
  let fun: fn(&()) -> TlsModeVerifyFull = fun;
  Pair::new(
    PkgsAux::from_minimum(
      GenericThrottlingApi {
        rc: RequestCounter::new(RequestLimit::new(5, Duration::from_secs(1))),
      },
      SerdeJson,
      HttpParams::from_uri("ws://generic_web_socket_uri.com".into()),
    ),
    ClientPoolBuilder::tokio(1).aux((), fun).build(),
  )
}

async fn web_socket_pair() -> wtx::Result<
  Pair<
    PkgsAux<GenericThrottlingApi, SerdeJson, WsParams>,
    WebSocket<
      (),
      Xorshift64,
      TlsStream<TcpStream, TlsBuffer, TlsModeVerifyFull, true>,
      WebSocketBuffer,
      true,
    >,
  >,
> {
  let uri = Uri::new("ws://generic_web_socket_uri.com");
  let mut rng = ChaCha20::from_getrandom()?;
  let tls_stream = TlsConnector::default()
    .connect(
      &mut rng,
      TcpStream::connect(uri.hostname_with_implied_port()).await?,
      &TlsConfig::default(),
    )
    .await?;
  let web_socket = WebSocketConnector::default().connect(tls_stream, &uri).await?;
  Ok(Pair::new(
    PkgsAux::from_minimum(
      GenericThrottlingApi {
        rc: RequestCounter::new(RequestLimit::new(40, Duration::from_secs(2))),
      },
      SerdeJson,
      WsParams::default(),
    ),
    web_socket,
  ))
}

#[tokio::main]
async fn main() -> wtx::Result<()> {
  let mut hp = http_pair();
  let _http_response_tuple = hp
    .trans
    .send_pkg_recv_decode_contained(
      &mut hp.pkgs_aux.generic_http_request().data(123).build(),
      &mut hp.pkgs_aux,
    )
    .await?
    .result?;

  let mut wsp = web_socket_pair().await?;
  let _web_socket_subscription_id = wsp
    .trans
    .send_pkg_recv_decode_contained(
      &mut wsp.pkgs_aux.generic_web_socket_subscription().data("Hello", None).build(),
      &mut wsp.pkgs_aux,
    )
    .await?
    .result?;
  Ok(())
}
