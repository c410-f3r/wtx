//! WebSocket echo server.

mod common;

use hyper::{server::conn::Http, service::service_fn, Body, Request, Response};
use tokio::{net::TcpListener, task};
use wtx::{
    web_socket::{
        handshake::{WebSocketUpgrade, WebSocketUpgradeHyper},
        WebSocketServer,
    },
    ReadBuffer,
};

#[tokio::main]
async fn main() -> wtx::Result<()> {
    let listener = TcpListener::bind(common::_host_from_args()).await?;
    loop {
        let (stream, _) = listener.accept().await?;
        let _jh = tokio::spawn(async move {
            let service = service_fn(server_upgrade);
            if let Err(err) = Http::new()
                .serve_connection(stream, service)
                .with_upgrades()
                .await
            {
                println!("An error occurred: {err}");
            }
        });
    }
}

async fn server_upgrade(req: Request<Body>) -> wtx::Result<Response<Body>> {
    let (res, fut) = WebSocketUpgradeHyper { req }.upgrade()?;
    let _jh = task::spawn(async move {
        let fut = async move {
            common::_handle_frames(
                &mut <_>::default(),
                &mut WebSocketServer::new(ReadBuffer::default(), fut.await?),
            )
            .await
        };
        if let Err(err) = tokio::task::unconstrained(fut).await {
            eprintln!("Error in WebSocket connection: {err}");
        }
    });
    Ok(res)
}
