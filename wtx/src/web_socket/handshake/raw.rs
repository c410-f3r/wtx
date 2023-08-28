use crate::{
    misc::AsyncBounds,
    web_socket::{
        handshake::{
            misc::{derived_key, gen_key, trim},
            HeadersBuffer, WebSocketAccept, WebSocketHandshake,
        },
        FrameBufferVec, WebSocketClient, WebSocketError, WebSocketServer,
    },
    ExpectedHeader, ReadBuffer, Stream, UriParts,
};
#[cfg(feature = "async-trait")]
use alloc::boxed::Box;
use core::borrow::BorrowMut;
use httparse::{Header, Status};

const MAX_READ_HEADER_LEN: usize = 64;
const MAX_READ_LEN: usize = 2 * 1024;

/// Marker used to implement [WebSocketAccept].
#[derive(Debug)]
pub struct WebSocketAcceptRaw<'any, RB, S> {
    /// Frame buffer
    pub fb: &'any mut FrameBufferVec,
    /// Headers buffer
    pub headers_buffer: &'any mut HeadersBuffer<'any, 3>,
    /// Key buffer
    pub key_buffer: &'any mut [u8; 30],
    /// Read buffer
    pub rb: RB,
    /// Stream
    pub stream: S,
}

#[cfg_attr(feature = "async-trait", async_trait::async_trait)]
impl<'any, RB, S> WebSocketAccept<RB> for WebSocketAcceptRaw<'any, RB, S>
where
    RB: AsyncBounds + BorrowMut<ReadBuffer>,
    S: AsyncBounds + Stream,
{
    type Response = crate::Response<'any, 'any>;
    type Stream = S;

    #[inline]
    async fn accept(
        mut self,
    ) -> crate::Result<(Self::Response, WebSocketServer<RB, Self::Stream>)> {
        self.fb.set_params_through_expansion(0, 0, MAX_READ_LEN);
        let mut read = 0;
        let (key, version) = loop {
            let read_buffer = self.fb.payload_mut().get_mut(read..).unwrap_or_default();
            let local_read = self.stream.read(read_buffer).await?;
            read = read.wrapping_add(local_read);
            if read > MAX_READ_LEN {
                return Err(crate::Error::VeryLargeHttp);
            }
            if local_read == 0 {
                return Err(crate::Error::UnexpectedEOF);
            }
            let working_buffer = self.fb.payload().get(..read).unwrap_or_default();
            let mut req_buffer = [httparse::EMPTY_HEADER; MAX_READ_HEADER_LEN];
            let mut req = httparse::Request::new(&mut req_buffer);
            match req.parse(working_buffer)? {
                Status::Complete(_) => {
                    if !req
                        .method
                        .map_or(false, |el| trim(el.as_bytes()).eq_ignore_ascii_case(b"get"))
                    {
                        return Err(crate::Error::UnexpectedHttpMethod);
                    }
                    verify_common_header(req.headers)?;
                    if !has_header_key_and_value(req.headers, "sec-websocket-version", b"13") {
                        return Err(crate::Error::MissingHeader {
                            expected: ExpectedHeader::SecWebSocketVersion_13,
                        });
                    };
                    let Some(key) = req.headers.iter().find_map(|el| {
                        (el.name.eq_ignore_ascii_case("sec-websocket-key")).then_some(el.value)
                    }) else {
                        return Err(crate::Error::MissingHeader {
                            expected: ExpectedHeader::SecWebSocketKey,
                        });
                    };
                    break (key, req.version);
                }
                Status::Partial => {}
            }
        };
        self.headers_buffer.headers[0] = Header {
            name: "Connection",
            value: b"Upgrade",
        };
        self.headers_buffer.headers[1] = Header {
            name: "Sec-WebSocket-Accept",
            value: derived_key(self.key_buffer, key).as_bytes(),
        };
        self.headers_buffer.headers[2] = Header {
            name: "Upgrade",
            value: b"websocket",
        };
        let mut httparse_res = httparse::Response::new(&mut self.headers_buffer.headers);
        httparse_res.code = Some(101);
        httparse_res.version = version;
        let res = crate::Response::new(&[], httparse_res);
        let res_bytes = build_101_res(self.fb, res.headers());
        self.stream.write_all(res_bytes).await?;
        Ok((res, WebSocketServer::new(self.rb, self.stream)))
    }
}

/// Marker used to implement [WebSocketHandshake].
#[derive(Debug)]
pub struct WebSocketHandshakeRaw<'any, RB, S> {
    /// Frame buffer
    pub fb: &'any mut FrameBufferVec,
    /// Headers buffer
    pub headers_buffer: &'any mut HeadersBuffer<'any, MAX_READ_HEADER_LEN>,
    /// Read buffer
    pub rb: RB,
    /// Stream
    pub stream: S,
    /// Uri
    pub uri: &'any str,
}

#[cfg_attr(feature = "async-trait", async_trait::async_trait)]
impl<'any, RB, S> WebSocketHandshake<RB> for WebSocketHandshakeRaw<'any, RB, S>
where
    RB: AsyncBounds + BorrowMut<ReadBuffer>,
    S: AsyncBounds + Stream,
{
    type Response = crate::Response<'any, 'any>;
    type Stream = S;

    #[inline]
    async fn handshake(
        mut self,
    ) -> crate::Result<(Self::Response, WebSocketClient<RB, Self::Stream>)> {
        self.fb.set_params_through_expansion(0, 0, MAX_READ_LEN);
        let key_buffer = &mut <_>::default();
        let (key, req) = build_upgrade_req(self.fb, key_buffer, self.uri);
        self.stream.write_all(req).await?;
        let mut read = 0;
        let res_len = loop {
            let read_buffer = self.fb.payload_mut().get_mut(read..).unwrap_or_default();
            let local_read = self.stream.read(read_buffer).await?;
            read = read.wrapping_add(local_read);
            if read > MAX_READ_LEN {
                return Err(crate::Error::VeryLargeHttp);
            }
            if local_read == 0 {
                return Err(crate::Error::UnexpectedEOF);
            }
            let mut headers = [httparse::EMPTY_HEADER; MAX_READ_HEADER_LEN];
            let mut httparse_res = httparse::Response::new(&mut headers);
            match httparse_res.parse(self.fb.payload().get(..read).unwrap_or_default())? {
                Status::Complete(el) => break el,
                Status::Partial => {}
            }
        };
        let mut httparse_res = httparse::Response::new(&mut self.headers_buffer.headers);
        let _rslt = httparse_res.parse(self.fb.payload().get(..res_len).unwrap_or_default())?;
        let res = crate::Response::new(&[], httparse_res);
        if res.code() != Some(101) {
            return Err(WebSocketError::MissingSwitchingProtocols.into());
        }
        verify_common_header(res.headers())?;
        if !has_header_key_and_value(
            res.headers(),
            "sec-websocket-accept",
            derived_key(&mut <_>::default(), key.as_bytes()).as_bytes(),
        ) {
            return Err(crate::Error::MissingHeader {
                expected: crate::ExpectedHeader::SecWebSocketKey,
            });
        }
        let idx = read.wrapping_sub(res_len);
        self.rb
            .borrow_mut()
            .set_indices_through_expansion(0, 0, idx);
        self.rb
            .borrow_mut()
            .after_current_mut()
            .get_mut(..idx)
            .unwrap_or_default()
            .copy_from_slice(self.fb.payload().get(res_len..read).unwrap_or_default());
        Ok((res, WebSocketClient::new(self.rb, self.stream)))
    }
}

fn build_upgrade_req<'fb, 'kb>(
    fb: &'fb mut FrameBufferVec,
    key_buffer: &'kb mut [u8; 26],
    uri: &str,
) -> (&'kb str, &'fb [u8]) {
    let uri_parts = UriParts::from(uri);
    let key = gen_key(key_buffer);

    let idx = fb.buffer().len();
    fb.buffer_mut().extend(b"GET ");
    fb.buffer_mut().extend(uri_parts.href.as_bytes());
    fb.buffer_mut().extend(b" HTTP/1.1\r\n");

    fb.buffer_mut().extend(b"Connection: Upgrade\r\n");
    fb.buffer_mut().extend(b"Host: ");
    fb.buffer_mut().extend(uri_parts.host.as_bytes());
    fb.buffer_mut().extend(b"\r\n");
    fb.buffer_mut().extend(b"Sec-WebSocket-Key: ");
    fb.buffer_mut().extend(key.as_bytes());
    fb.buffer_mut().extend(b"\r\n");
    fb.buffer_mut().extend(b"Sec-WebSocket-Version: 13\r\n");
    fb.buffer_mut().extend(b"Upgrade: websocket\r\n");

    fb.buffer_mut().extend(b"\r\n");

    (key, fb.buffer().get(idx..).unwrap_or_default())
}

fn build_101_res<'fb>(fb: &'fb mut FrameBufferVec, headers: &[Header<'_>]) -> &'fb [u8] {
    let idx = fb.buffer().len();
    fb.buffer_mut()
        .extend(b"HTTP/1.1 101 Switching Protocols\r\n");
    for header in headers {
        fb.buffer_mut().extend(header.name.as_bytes());
        fb.buffer_mut().extend(b": ");
        fb.buffer_mut().extend(header.value);
        fb.buffer_mut().extend(b"\r\n");
    }
    fb.buffer_mut().extend(b"\r\n");
    fb.buffer().get(idx..).unwrap_or_default()
}

fn has_header_key_and_value(buffer: &[Header<'_>], key: &str, value: &[u8]) -> bool {
    buffer
        .iter()
        .find_map(|h| {
            let has_key = trim(h.name.as_bytes()).eq_ignore_ascii_case(key.as_bytes());
            let has_value = h
                .value
                .split(|el| el == &b',')
                .any(|el| trim(el).eq_ignore_ascii_case(value));
            (has_key && has_value).then_some(true)
        })
        .unwrap_or(false)
}

fn verify_common_header(buffer: &[Header<'_>]) -> crate::Result<()> {
    if !has_header_key_and_value(buffer, "connection", b"upgrade") {
        return Err(crate::Error::MissingHeader {
            expected: ExpectedHeader::Connection_Upgrade,
        });
    }
    if !has_header_key_and_value(buffer, "upgrade", b"websocket") {
        return Err(crate::Error::MissingHeader {
            expected: ExpectedHeader::Upgrade_WebSocket,
        });
    }
    Ok(())
}
