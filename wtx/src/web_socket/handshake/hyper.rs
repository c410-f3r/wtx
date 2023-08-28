use crate::{
    misc::AsyncBounds,
    web_socket::{
        handshake::{
            misc::{derived_key, gen_key, trim},
            WebSocketHandshake, WebSocketUpgrade,
        },
        WebSocketClient, WebSocketError,
    },
    Error::MissingHeader,
    ExpectedHeader, ReadBuffer,
};
#[cfg(feature = "async-trait")]
use alloc::boxed::Box;
use core::{
    borrow::BorrowMut,
    future::Future,
    pin::{pin, Pin},
    task::{ready, Context, Poll},
};
use hyper::{
    client::conn::{self, Connection},
    header::{CONNECTION, HOST, UPGRADE},
    http::{HeaderMap, HeaderValue},
    rt::Executor,
    upgrade::{self, OnUpgrade, Upgraded},
    Body, Request, Response, StatusCode,
};
use tokio::io::{AsyncRead, AsyncWrite};

/// A future that resolves to a WebSocket stream when the associated HTTP upgrade completes.
#[derive(Debug)]
pub struct UpgradeFutHyper {
    inner: OnUpgrade,
}

impl Future for UpgradeFutHyper {
    type Output = crate::Result<Upgraded>;

    #[inline]
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let stream = ready!(pin!(&mut self.inner).poll(cx))?;
        Poll::Ready(Ok(stream))
    }
}

/// Marker used to implement [WebSocketHandshake].
#[derive(Debug)]
pub struct WebSocketHandshakeHyper<'executor, E, RB, S> {
    /// Executor
    pub executor: &'executor E,
    /// Read buffer
    pub rb: RB,
    /// Request
    pub req: Request<Body>,
    /// Stream
    pub stream: S,
}

#[cfg_attr(feature = "async-trait", async_trait::async_trait)]
impl<'executor, E, RB, S> WebSocketHandshake<RB> for WebSocketHandshakeHyper<'executor, E, RB, S>
where
    E: AsyncBounds + Executor<Connection<S, Body>> + 'executor,
    RB: AsyncBounds + BorrowMut<ReadBuffer>,
    S: AsyncRead + AsyncWrite + Send + Unpin + 'static,
{
    type Response = Response<Body>;
    type Stream = Upgraded;

    #[inline]
    async fn handshake(
        mut self,
    ) -> crate::Result<(Self::Response, WebSocketClient<RB, Self::Stream>)> {
        let fun = || {
            let authority = self.req.uri().authority().map(|el| el.as_str())?;
            let mut iter = authority.split('@');
            let before_at = iter.next()?;
            Some(iter.next().unwrap_or(before_at))
        };
        let host = fun().ok_or(crate::Error::MissingHost)?.parse()?;
        drop(
            self.req
                .headers_mut()
                .insert(CONNECTION, HeaderValue::from_static("upgrade")),
        );
        drop(self.req.headers_mut().insert(HOST, host));
        drop(
            self.req
                .headers_mut()
                .insert("Sec-WebSocket-Key", gen_key(&mut <_>::default()).parse()?),
        );
        drop(
            self.req
                .headers_mut()
                .insert("Sec-WebSocket-Version", HeaderValue::from_static("13")),
        );
        drop(
            self.req
                .headers_mut()
                .insert(UPGRADE, HeaderValue::from_static("websocket")),
        );
        let (mut sender, conn) = conn::handshake(self.stream).await?;
        self.executor.execute(conn);
        let mut res = sender.send_request(self.req).await?;
        verify_res(&res)?;
        match upgrade::on(&mut res).await {
            Err(err) => Err(err.into()),
            Ok(elem) => Ok((res, WebSocketClient::new(self.rb, elem))),
        }
    }
}

/// Structured used to implement [WebSocketUpgrade].
#[derive(Debug)]
pub struct WebSocketUpgradeHyper<T> {
    /// Request
    pub req: Request<T>,
}

#[cfg_attr(feature = "async-trait", async_trait::async_trait)]
impl<T> WebSocketUpgrade for WebSocketUpgradeHyper<T>
where
    T: AsyncBounds,
{
    type Response = Response<Body>;
    type Stream = Upgraded;
    type Upgrade = UpgradeFutHyper;

    #[inline]
    fn upgrade(self) -> crate::Result<(Self::Response, Self::Upgrade)> {
        verify_headers(self.req.headers())?;
        let sws_opt = self.req.headers().get("Sec-WebSocket-Key");
        let swk = sws_opt.ok_or(MissingHeader {
            expected: ExpectedHeader::SecWebSocketKey,
        })?;
        if self
            .req
            .headers()
            .get("Sec-WebSocket-Version")
            .map(HeaderValue::as_bytes)
            != Some(b"13")
        {
            return Err(MissingHeader {
                expected: ExpectedHeader::SecWebSocketVersion_13,
            });
        }
        let res = Response::builder()
            .status(StatusCode::SWITCHING_PROTOCOLS)
            .header(CONNECTION, "upgrade")
            .header(UPGRADE, "websocket")
            .header(
                "Sec-WebSocket-Accept",
                derived_key(&mut <_>::default(), swk.as_bytes()),
            )
            .body(Body::from("switching to websocket protocol"))?;
        let stream = UpgradeFutHyper {
            inner: upgrade::on(self.req),
        };
        Ok((res, stream))
    }
}

fn verify_headers(hm: &HeaderMap) -> crate::Result<()> {
    if !hm
        .get("Upgrade")
        .map(|h| h.as_bytes())
        .map_or(false, |h| trim(h).eq_ignore_ascii_case(b"websocket"))
    {
        return Err(MissingHeader {
            expected: ExpectedHeader::Upgrade_WebSocket,
        });
    }
    if !hm
        .get("Connection")
        .map(|h| h.as_bytes())
        .map_or(false, |h| trim(h).eq_ignore_ascii_case(b"upgrade"))
    {
        return Err(MissingHeader {
            expected: ExpectedHeader::Connection_Upgrade,
        });
    }
    Ok(())
}

fn verify_res(res: &Response<Body>) -> crate::Result<()> {
    if res.status() != StatusCode::SWITCHING_PROTOCOLS {
        return Err(WebSocketError::MissingSwitchingProtocols.into());
    }
    verify_headers(res.headers())?;
    Ok(())
}
