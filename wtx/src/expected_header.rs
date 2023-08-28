/// Expected HTTP headers
#[allow(non_camel_case_types)]
#[derive(Debug)]
pub enum ExpectedHeader {
    /// `connection` key with `upgrade` value.
    Connection_Upgrade,
    /// `sec-websocket-key` key.
    SecWebSocketKey,
    /// `sec-websocket-version` key with `13` value.
    SecWebSocketVersion_13,
    /// `upgrade` key with `websocket` value.
    Upgrade_WebSocket,
}
