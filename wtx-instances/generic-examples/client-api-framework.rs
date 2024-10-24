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
extern crate wtx_macros;

use core::time::Duration;
use tokio::net::TcpStream;
use wtx::{
  client_api_framework::{
    misc::{Pair, RequestLimit, RequestThrottling},
    network::{transport::Transport, HttpParams, WsParams},
    Api,
  },
  data_transformation::dnsn::SerdeJson,
  http::client_framework::ClientFrameworkTokio,
  misc::{simple_seed, Uri, Xorshift64},
  web_socket::{WebSocketBuffer, WebSocketClient},
};

wtx::create_packages_aux_wrapper!();

#[derive(Debug)]
#[wtx_macros::api_params(pkgs_aux(PkgsAux), transport(http, ws))]
pub struct GenericThrottlingApi {
  pub rt: RequestThrottling,
}

impl Api for GenericThrottlingApi {
  type Error = wtx::Error;

  async fn before_sending(&mut self) -> Result<(), Self::Error> {
    self.rt.rc.update_params(&self.rt.rl).await?;
    Ok(())
  }
}

#[wtx_macros::pkg(
  api(crate::GenericThrottlingApi),
  data_format(json_rpc("genericHttpRequest")),
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

#[wtx_macros::pkg(
  api(crate::GenericThrottlingApi),
  data_format(json_rpc("genericWebSocketSubscription")),
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

#[tokio::main]
async fn main() -> wtx::Result<()> {
  async fn http_pair(
  ) -> Pair<PkgsAux<GenericThrottlingApi, SerdeJson, HttpParams>, ClientFrameworkTokio> {
    Pair::new(
      PkgsAux::from_minimum(
        GenericThrottlingApi {
          rt: RequestThrottling::from_rl(RequestLimit::new(5, Duration::from_secs(1))),
        },
        SerdeJson,
        HttpParams::from_uri("ws://generic_web_socket_uri.com".into()),
      ),
      ClientFrameworkTokio::tokio(1).build(),
    )
  }

  async fn web_socket_pair() -> wtx::Result<
    Pair<
      PkgsAux<GenericThrottlingApi, SerdeJson, WsParams>,
      WebSocketClient<(), TcpStream, WebSocketBuffer>,
    >,
  > {
    let uri = Uri::new("ws://generic_web_socket_uri.com");
    let web_socket = WebSocketClient::connect(
      (),
      [],
      Xorshift64::from(simple_seed()),
      TcpStream::connect(uri.hostname_with_implied_port()).await?,
      &uri,
      WebSocketBuffer::default(),
      |_| wtx::Result::Ok(()),
    )
    .await?;
    Ok(Pair::new(
      PkgsAux::from_minimum(
        GenericThrottlingApi {
          rt: RequestThrottling::from_rl(RequestLimit::new(40, Duration::from_secs(2))),
        },
        SerdeJson,
        WsParams::default(),
      ),
      web_socket,
    ))
  }

  let mut hp = http_pair().await;
  let _http_response_tuple = hp
    .trans
    .send_recv_decode_contained(
      &mut hp.pkgs_aux.generic_http_request().data(123).build(),
      &mut hp.pkgs_aux,
    )
    .await?
    .result?;

  let mut wsp = web_socket_pair().await?;
  let _web_socket_subscription_id = wsp
    .trans
    .send_recv_decode_contained(
      &mut wsp.pkgs_aux.generic_web_socket_subscription().data("Hello", None).build(),
      &mut wsp.pkgs_aux,
    )
    .await?
    .result?;
  Ok(())
}
