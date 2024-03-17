use crate::{
  http2::{FrameInit, StreamState},
  misc::BlocksQueue,
};

#[derive(Debug)]
pub(crate) struct StreamData<const IS_CLIENT: bool> {
  pub(crate) received_frames: BlocksQueue<u8, FrameInit>,
  pub(crate) state: StreamState,
}
