use crate::http2::{
  DataFrame, GoAwayFrame, HeadersFrame, PingFrame, ResetFrame, SettingsFrame, WindowUpdateFrame,
};

#[derive(Debug)]
pub enum Frame<'data, 'headers> {
  Continuation,
  Data(DataFrame<'data>),
  GoAway(GoAwayFrame<'data>),
  Headers(HeadersFrame<'data, 'headers>),
  Ping(PingFrame),
  Reset(ResetFrame),
  Settings(SettingsFrame),
  WindowUpdate(WindowUpdateFrame),
}
